use axum::Router;
use crate::{errors::Result, AppState};
use crate::model::Client;
use crate::restlist::JsonRestList;
use axum::{routing, extract::{Path, self}, Json};
use axum::extract::State;

pub fn api_routes() -> Router<AppState> {
  Router::new()
    .nest("/v1/client", clients::routes())
    .nest("/v1/domainlist", domains::routes())
}

// Nice, but still somewhat repetitive. It'd be cool if we could avoid
// all this boilerplate.
pub static CLIENTS_JSON: &str = "config/clients.json";

mod clients {
  use super::*;

  pub(super) fn routes() -> Router<AppState> {
    Router::new()
      .route("/", routing::get(get_all))
      .route("/", routing::post(post))
      .route("/:id", routing::get(get))
      .route("/:id", routing::put(put))
      .route("/:id", routing::delete(delete))

    }

  fn load() -> anyhow::Result<JsonRestList<Client>> {
    JsonRestList::<Client>::load(CLIENTS_JSON)
  }

  async fn get_all() -> Result<Json<Vec<Client>>> {
    load()?.get_all()
  }

  async fn get(Path(id): Path<u32>) -> Result<Json<Client>> {
    load()?.get(id)
  }

  async fn put(State(state): State<AppState>, Path(id): Path<u32>,
      extract::Json(client): extract::Json<Client>) -> Result<Json<Client>> {
    let result = load()?.put(id, client.clone());
    state.regenerate().await;

    result
  }

  async fn delete(State(state): State<AppState>, Path(id): Path<u32>) -> Result<Json<Client>> {
    let result = load()?.delete(id);
    state.regenerate().await;

    result
  }

  async fn post(State(state): State<AppState>, extract::Json(client): extract::Json<Client>) -> Result<Json<Client>> {
    let result = load()?.add(client.clone());
    state.regenerate().await;

    result
  }

}

pub static DOMAINS_JSON: &str = "config/domains.json";

mod domains {
  use crate::model::DomainList;

  use super::*;

  pub(super) fn routes() -> Router<AppState> {
    Router::new()
      .route("/", routing::get(get_all))
      .route("/", routing::post(post))
      .route("/:id", routing::get(get))
      .route("/:id", routing::put(put))
      .route("/:id", routing::delete(delete))
  }

  fn load() -> anyhow::Result<JsonRestList<DomainList>> {
    JsonRestList::<DomainList>::load(DOMAINS_JSON)
  }

  async fn get_all() -> Result<Json<Vec<DomainList>>> {
    load()?.get_all()
  }

  async fn get(Path(id): Path<u32>) -> Result<Json<DomainList>> {
    load()?.get(id)
  }

  async fn put(Path(id): Path<u32>, extract::Json(client): extract::Json<DomainList>) -> Result<Json<DomainList>> {
    load()?.put(id, client)
  }

  async fn delete(Path(id): Path<u32>) -> Result<Json<DomainList>> {
    load()?.delete(id)
  }

  async fn post(extract::Json(client): extract::Json<DomainList>) -> Result<Json<DomainList>> {
    load()?.add(client)
  }
}
