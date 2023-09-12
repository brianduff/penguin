use std::sync::{Arc, atomic::{AtomicU32, Ordering}, Mutex};

use api::{api_routes, DOMAINS_JSON};
use axum::{Router, routing::get, extract::State};
use chrono::Utc;
use model::DomainList;
use restlist::JsonRestList;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_schedule::{every, Job};

use crate::{model::Client, api::CLIENTS_JSON, generate::generate_squid_config};

static SQUID_DIR: &str = "squid";

mod api;
mod file;
mod list;
mod errors;
mod model;
mod generate;
mod restlist;


#[derive(Clone, Copy)]
pub enum Event {
    GenerateConfiguration
}

async fn regenerate_config_handler(State(state): State<AppState>) -> errors::Result<String> {
    regenerate_config(state).await
}

async fn regenerate_config(state: AppState) -> errors::Result<String> {
    let mut guard = state.gen_config_lock.lock().unwrap();
    let domains = JsonRestList::<DomainList>::load(DOMAINS_JSON)?;
    let clients = JsonRestList::<Client>::load(CLIENTS_JSON)?;

    generate_squid_config(SQUID_DIR, &clients.list, &domains.list)?;
    *guard += 1;
    println!("Wrote squid configuration. Generation={}", *guard);

    Ok("Done".to_owned())
}

async fn possibly_regenerate_config(state: AppState) -> errors::Result<String> {
    // TODO: check lastmod time of the files and skip loading if not changed.
    let mut clients = JsonRestList::<Client>::load(CLIENTS_JSON)?;

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
        println!("Regenerating due to expired leases");
        return regenerate_config(state).await;
    }

    Ok("Done".to_owned())
}

async fn listen_for_events(state: AppState, mut rx: Receiver<Event>) {
    while let Some(event) = rx.recv().await {
        match event {
            Event::GenerateConfiguration => {
                regenerate_config(state.clone()).await.unwrap();
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
    config_lock: Arc<Mutex<u32>>
}

impl AppState {
    pub async fn regenerate(&self) {
        self.events.send(Event::GenerateConfiguration).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    // Set up a configuration change receiver.
    let (tx, rx) = mpsc::channel::<Event>(10);

    let state = AppState {
        events: tx,
        gen_config_lock: Arc::new(Mutex::new(0)),
        config_lock: Arc::new(Mutex::new(0))
    };

    let state_for_listen = state.clone();
    tokio::spawn(async move {
        listen_for_events(state_for_listen, rx).await
    });

    // On startup, regenerate squid configuration in case it changed while the
    // server was down.
    let state_for_startup = state.clone();
    tokio::spawn(async {
        regenerate_config(state_for_startup).await.unwrap()
    });

    // Every 30s regenerate squid configuration for expired leases, and remove
    // expired leases from our own configuration.
    let state_for_cron = state.clone();
    tokio::spawn(async move {
        every(2).seconds().perform(|| async {
            possibly_regenerate_config(state_for_cron.clone()).await.unwrap();
        }).await;
    });


    let app = Router::new()
        .route("/generate", get(regenerate_config_handler))
        .nest("/api", api_routes())
        .with_state(state);

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}