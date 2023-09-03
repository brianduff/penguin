use actix_web::{App, HttpServer};
use api::api_service;

mod api;
mod list;
mod errors;
mod model;
//mod generate;
mod restlist;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(api_service())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}