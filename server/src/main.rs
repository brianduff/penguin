use std::{sync::{Arc, Mutex}, path::Path, time::Duration};

use api::api_routes;
use axum::{Router, routing::get, extract::State};
use chrono::Utc;
use model::{DomainList, Conf};
use restlist::JsonRestList;
use tempdir::TempDir;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_schedule::{every, Job};
use tower_http::{trace::{TraceLayer, self}, cors::{CorsLayer, Any}};
use tracing::Level;

use crate::{model::Client, generate::generate_squid_config, file::get_parent_or_die};

mod api;
mod file;
mod list;
mod errors;
mod model;
mod generate;
mod restlist;
mod squid;

const PORT: u32 = 8080;

#[derive(Clone, Copy)]
pub enum Event {
    GenerateConfiguration
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

    squid::reload_config();

    Ok("Done".to_owned())
}

async fn possibly_regenerate_config(state: AppState) -> anyhow::Result<String> {
    // TODO: check lastmod time of the files and skip loading if not changed.
    let mut clients = JsonRestList::<Client>::load(state.app_config.clients_json())?;

    let now = Utc::now().naive_local();
    let mut lease_found: bool = false;
    for client in clients.list.items.iter_mut() {
        let old_len = client.leases.len();
        client.leases.retain(|l| now <= l.end_date);
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
    app_config: Conf
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
        app_config: Conf::load().unwrap()
    };

    let state_for_listen = state.clone();
    tokio::spawn(async move {
        listen_for_events(state_for_listen, rx).await
    });

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
        every(2).seconds().perform(|| async {
            if let Err(err) = possibly_regenerate_config(state_for_cron.clone()).await {
                tracing::error!("Failed to generate config: {:?}", err);
            }
        }).await;
    });

    // A permissive cors policy because we're expecting to be behind a firewall.
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/statusz", get(status))
        .route("/generate", get(regenerate_config_handler))
        .nest("/api", api_routes())
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)));

    // Spawn a statusz poller. This pings statusz a few times to make sure it's up
    // and sends notify a ready state when it is.
    tokio::spawn(async {
        poll_statusz().await;
    });

    axum::Server::bind(&format!("0.0.0.0:{}", PORT).parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

}

static TRIES: u8 = 50;

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