use actix_web::{App, HttpServer, Responder, get};
use api::{api_service, DOMAINS_JSON};
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

#[get("/generate")]
async fn gen() -> errors::Result<impl Responder> {
    let domains = JsonRestList::<DomainList>::load(DOMAINS_JSON)?;
    let clients = JsonRestList::<Client>::load(CLIENTS_JSON)?;

    generate_squid_config(SQUID_DIR, &clients.list, &domains.list)?;

    Ok("Done")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(api_service())
            .service(gen)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}