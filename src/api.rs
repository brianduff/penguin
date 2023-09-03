use actix_web::web::scope;
use actix_web::Scope;
use actix_web::{delete, get, put, Responder,post,  web};
use crate::errors::Result;
use crate::model::Client;
use crate::restlist::JsonRestList;

pub fn api_service() -> Scope {
  scope("/api/v1")
    .service(clients::service())
    .service(domains::service())
}

// Nice, but still somewhat repetitive. It'd be cool if we could avoid
// all this boilerplate.

mod clients {
  use super::*;
  static CLIENTS_JSON: &str = "config/clients.json";

  pub(super) fn service() -> Scope {
    scope("/client")
        .service(get_all)
        .service(get)
        .service(put)
        .service(post)
        .service(delete)
  }

  fn load() -> anyhow::Result<JsonRestList<Client>> {
    JsonRestList::<Client>::load(CLIENTS_JSON)
  }

  #[get("")]
  async fn get_all() -> Result<impl Responder> {
    load()?.get_all()
  }

  #[get("/{id}")]
  async fn get(params: web::Path<(u32,)>) -> Result<impl Responder> {
    load()?.get(params.0)
  }

  #[put("/{id}")]
  async fn put(params: web::Path<(u32,)>, client: web::Json<Client>) -> Result<impl Responder> {
    load()?.put(params.0, client.into_inner())
  }

  #[delete("/{id}")]
  async fn delete(params: web::Path<(u32,)>) -> Result<impl Responder> {
    load()?.delete(params.0)
  }

  #[post("")]
  async fn post(client: web::Json<Client>) -> Result<impl Responder> {
    load()?.add(client.into_inner())
  }

}

mod domains {
  use crate::model::DomainList;

  use super::*;

  static DOMAINS_JSON: &str = "config/domains.json";

  pub(super) fn service() -> Scope {
    scope("/domainlist")
        .service(get_all)
        .service(get)
        .service(put)
        .service(post)
        .service(delete)
  }

  fn load() -> anyhow::Result<JsonRestList<DomainList>> {
    JsonRestList::<DomainList>::load(DOMAINS_JSON)
  }

  #[get("")]
  async fn get_all() -> Result<impl Responder> {
    load()?.get_all()
  }

  #[get("/{id}")]
  async fn get(params: web::Path<(u32,)>) -> Result<impl Responder> {
    load()?.get(params.0)
  }

  #[put("/{id}")]
  async fn put(params: web::Path<(u32,)>, dl: web::Json<DomainList>) -> Result<impl Responder> {
    load()?.put(params.0, dl.into_inner())
  }

  #[delete("/{id}")]
  async fn delete(params: web::Path<(u32,)>) -> Result<impl Responder> {
    load()?.delete(params.0)
  }

  #[post("")]
  async fn post(dl: web::Json<DomainList>) -> Result<impl Responder> {
    load()?.add(dl.into_inner())
  }
}
