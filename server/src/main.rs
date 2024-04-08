use std::{
  path::Path,
  sync::{Arc, Mutex},
  time::Duration,
};

use api::api_routes;
use axum::{extract::State, routing::get, Router};
use chrono::{Local, NaiveDateTime, Timelike, Utc};
use model::{Conf, DomainList};
use restlist::JsonRestList;
use serde_json::Value;
use squid::ActiveState;
use tempdir::TempDir;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_schedule::{every, Job};
use tower_http::{
  cors::{Any, CorsLayer},
  trace::{self, TraceLayer},
};
use tracing::{error, warn, Level, info};
use unifi::UnifiClient;

use crate::{
  file::{get_parent_or_die, read_json_value, write_json_value},
  generate::generate_squid_config,
  model::Client,
};

mod api;
mod auth;
mod errors;
mod file;
mod generate;
mod list;
mod model;
mod restlist;
mod squid;
mod unifi;

const PORT: u32 = 8080;

#[derive(Clone, Copy)]
pub enum Event {
  GenerateConfiguration,
}

async fn status() -> errors::Result<String> {
  Ok("OK".to_owned())
}

async fn regenerate_config_handler(State(state): State<AppState>) -> errors::Result<String> {
  regenerate_config(state).await?;

  Ok("Done".to_owned())
}

async fn regenerate_config(state: AppState) -> anyhow::Result<String> {
  let mut guard = state.gen_config_lock.lock().unwrap();
  let domains = JsonRestList::<DomainList>::load(state.app_config.domains_json())?;
  let clients = JsonRestList::<Client>::load(state.app_config.clients_json())?;

  let temp_dir = TempDir::new("penguin-squid")?;
  std::fs::create_dir_all(&temp_dir)?;
  generate_squid_config(&temp_dir, &clients.list, &domains.list)?;

  std::fs::create_dir_all(&state.app_config.squid_config_dir)?;
  let dest_dir = std::fs::canonicalize(Path::new(&state.app_config.squid_config_dir))?;
  let old_dir = get_parent_or_die(&dest_dir)?.join("squid_old");

  tracing::info!("Regenerating squid config to {:?}", dest_dir);
  if old_dir.exists() {
    std::fs::remove_dir_all(&old_dir)?;
  }
  if dest_dir.exists() {
    std::fs::rename(&dest_dir, old_dir)?;
  }

  std::fs::rename(temp_dir, &dest_dir)?;
  *guard += 1;
  tracing::info!("Wrote squid configuration. Generation={}", *guard);

  if state.app_config.hup_squid_daemon {
    squid::reload_config();
  }

  Ok("Done".to_owned())
}

async fn possibly_regenerate_config(state: AppState) -> anyhow::Result<String> {
  // TODO: check lastmod time of the files and skip loading if not changed.
  let mut clients = JsonRestList::<Client>::load(state.app_config.clients_json())?;

  let now = Utc::now();
  let mut lease_found: bool = false;
  for client in clients.list.items.iter_mut() {
    let old_len = client.leases.len();
    client.leases.retain(|l| now <= l.end_date_utc.unwrap());
    let new_len = client.leases.len();
    if new_len != old_len {
      lease_found = true;
    }
  }

  if lease_found {
    // Save the clients config back out with the expired leases removed.
    clients.save()?;
    tracing::info!("Regenerating due to expired leases");
    return regenerate_config(state).await;
  }

  Ok("Done".to_owned())
}

async fn listen_for_events(state: AppState, mut rx: Receiver<Event>) {
  while let Some(event) = rx.recv().await {
    match event {
      Event::GenerateConfiguration => {
        let result = regenerate_config(state.clone()).await;
        if let Err(err) = result {
          tracing::error!("Failed to generate config: {:?}", err);
        }
      }
    }
  }
}

#[derive(Clone)]
pub struct AppState {
  events: Sender<Event>,
  // A mutex on the generated squid configuration
  gen_config_lock: Arc<Mutex<u32>>,
  // A mutex on the local configuration json files
  //    config_lock: Arc<Mutex<u32>>,

  // App config
  app_config: Conf,

  unifi_client: Arc<tokio::sync::Mutex<Option<UnifiClient>>>,
}

impl AppState {
  pub async fn regenerate(&self) {
    if let Err(e) = self.events.send(Event::GenerateConfiguration).await {
      tracing::error!("Failed to send regenerate event: {:?}", e);
    }
  }
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt()
    .with_max_level(tracing::Level::INFO)
    .init();

  tracing::info!("Starting");

  // Set up a configuration change receiver.
  let (tx, rx) = mpsc::channel::<Event>(10);

  let state = AppState {
    events: tx,
    gen_config_lock: Arc::new(Mutex::new(0)),
    //       config_lock: Arc::new(Mutex::new(0)),
    app_config: Conf::load().unwrap(),
    unifi_client: Arc::new(tokio::sync::Mutex::new(None))
  };

  if let Err(e) = repair_client_json(&state).await {
    error!("Failed to repair client json: {:?}", e);
  }

  let state_for_listen = state.clone();
  tokio::spawn(async move { listen_for_events(state_for_listen, rx).await });

  // On startup, regenerate squid configuration in case it changed while the
  // server was down.
  let state_for_startup = state.clone();
  tokio::spawn(async {
    if let Err(err) = regenerate_config(state_for_startup).await {
      tracing::error!("Failed to generate config: {:?}", err);
    }
  });

  // Every 30s regenerate squid configuration for expired leases, and remove
  // expired leases from our own configuration.
  let state_for_cron = state.clone();
  tokio::spawn(async move {
    every(30)
      .seconds()
      .perform(|| async {
        if let Err(err) = possibly_regenerate_config(state_for_cron.clone()).await {
          tracing::error!("Failed to generate config: {:?}", err);
        }
      })
      .await;
  });

  if state.app_config.unifi.enabled {
    if state.app_config.unifi.username.is_none() || state.app_config.unifi.password.is_none() {
      error!("Support for Unifi is enabled in config, but username and password are not specified. Ignoring.");
      return;
    }

    let state_for_unifi = state.clone();
    tokio::spawn(async move {
      let username = state_for_unifi.app_config.unifi.username.unwrap();
      let password = state_for_unifi.app_config.unifi.password.unwrap();

      let mut client = UnifiClient::new(&username, &password); // TODO: pass url
      match client.login().await {
        Ok(()) => {
          let mut mutex = state_for_unifi.unifi_client.lock_owned().await;
          *mutex = Some(client);
          info!("Successfully connected to UniFi at {}", state_for_unifi.app_config.unifi.url);
        },
        Err(e) => {
          error!("Support for unifi is enabled in config, but failed to log in: {:?}", e);
        }
      }
    });
  }

  // A permissive cors policy because we're expecting to be behind a firewall.
  let cors = CorsLayer::new()
    .allow_methods(Any)
    .allow_origin(Any)
    .allow_headers(Any);

  let app = Router::new()
    .route("/statusz", get(status))
    .route("/generate", get(regenerate_config_handler))
    .nest("/api", api_routes())
    .with_state(state)
    .layer(cors)
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

  // Spawn a statusz poller. This pings statusz a few times to make sure it's up
  // and sends notify a ready state when it is.
  tokio::spawn(async {
    poll_statusz().await;
  });

  tokio::spawn(async move {
    proxy_schedule().await;
    every(10).minutes().perform(proxy_schedule).await;
  });

  axum::Server::bind(&format!("0.0.0.0:{}", PORT).parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
}

static TRIES: u8 = 50;

/// Keeps the proxy running only at certain times of the day.
/// This should be generalized and not hardcoded here.
async fn proxy_schedule() {
  let now = Local::now();

  // The proxy is up between 7am and 9:30pm.
  let proxy_up = now.hour() >= 7 && (now.hour() < 21 || (now.hour() == 21 && now.minute() < 30));

  // Check the current status.
  let status = squid::get_status();
  let currently_up = matches!(status.active, ActiveState::Active);

  if proxy_up != currently_up {
    if proxy_up {
      info!("Proxy should be up at this time but isn't. Starting now.");
    } else {
      info!("Proxy should be down at this time but isn't. Stopping now.")
    }

    info!("Hour={}. Minute={}", now.hour(), now.minute());

    let result = squid::set_running(proxy_up);
    if let Err(result) = result {
      warn!("Failed to enable or disable proxy: {:?}. Will try again in 10 minutes", result);
    }
  }
}

async fn poll_statusz() {
  let mut tries = 0;
  while tries < TRIES {
    tries += 1;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let resp = reqwest::get(format!("http://0.0.0.0:{}/statusz", PORT)).await;
    if let Ok(resp) = resp {
      if let Ok(text) = resp.text().await {
        if "OK" == text {
          tracing::info!("Server listening on port {}", PORT);
          #[cfg(target_os = "linux")]
          {
            libsystemd::daemon::notify(false, &[libsystemd::daemon::NotifyState::Ready]);
          }
          return;
        }
      }
    }
    tracing::debug!("Still waiting for server to start.. (attempt {})", tries);
  }
}

async fn repair_client_json(state: &AppState) -> anyhow::Result<()> {
  if state.app_config.clients_json().exists() {
    let mut repaired = false;

    // Patch the json to fix compatibility issues. TODO: generalize this.
    let mut client_value: Value = read_json_value(&state.app_config.clients_json())?;
    if let Some(clients) = client_value.as_array_mut() {
      for client in clients.iter_mut() {
        if let Some(client) = client.as_object_mut() {
          if let Some(leases) = client.get_mut("leases") {
            if let Some(leases) = leases.as_array_mut() {
              for lease in leases.iter_mut() {
                if let Some(lease) = lease.as_object_mut() {
                  if let Some(v) = lease.get("end_date") {
                    if lease.get("end_date_utc").is_none() {
                      warn!(
                        "Repairing non-UTC date in {:?}",
                        &state.app_config.clients_json()
                      );
                      repaired = true;
                      let date: NaiveDateTime = serde_json::from_value(v.clone())?;
                      lease.insert(
                        "end_date_utc".to_string(),
                        Value::Number(date.timestamp_millis().into()),
                      );
                    }
                  }
                }
              }
            }
          }
        }
      }
    }

    if repaired {
      warn!("Writing repaired {:?}", &state.app_config.clients_json());
      write_json_value(&state.app_config.clients_json(), &client_value)?;
    }
  }

  Ok(())
}
