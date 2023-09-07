use api::{api_routes, DOMAINS_JSON};
use axum::{Router, routing::get};
use model::DomainList;
use restlist::JsonRestList;

use crate::{model::Client, api::CLIENTS_JSON, generate::generate_squid_config};

static SQUID_DIR: &str = "squid";

mod api;
mod file;
mod list;
mod errors;
mod model;
mod generate;
mod restlist;

async fn gen() -> errors::Result<String> {
    let domains = JsonRestList::<DomainList>::load(DOMAINS_JSON)?;
    let clients = JsonRestList::<Client>::load(CLIENTS_JSON)?;

    generate_squid_config(SQUID_DIR, &clients.list, &domains.list)?;

    Ok("Done".to_owned())
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest("/api", api_routes())
        .route("/generate", get(gen));
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}