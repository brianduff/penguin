use crate::errors::MyError;
use crate::model::Client;
use crate::restlist::JsonRestList;
use crate::{errors::Result, AppState};
use axum::extract::State;
use axum::Router;
use axum::{
  extract::{self, Path},
  routing, Json,
};

pub fn api_routes() -> Router<AppState> {
  Router::new()
    .nest("/v1/client", clients::routes())
    .nest("/v1/domainlist", domains::routes())
    .nest("/v1/netaccess", netaccess::routes())
}

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

  fn check<F, S: Into<String>>(test: F, message: S) -> Result<()>
  where
    F: FnOnce() -> bool,
  {
    if test() {
      return Err(MyError::BadRequest(message.into()));
    }

    Ok(())
  }

  fn other_clients<'a>(clients: &'a JsonRestList<Client>, client: &'a Client) -> Vec<&'a Client> {
    clients
      .list
      .items
      .iter()
      .filter(|c| c.id != client.id)
      .collect()
  }

  fn validate(clients: &JsonRestList<Client>, client: &Client) -> Result<()> {
    check(
      || client.name.trim().is_empty(),
      "Client name must not be empty",
    )?;
    check(
      || {
        other_clients(clients, client)
          .iter()
          .map(|c| &c.ip)
          .any(|v| v == &client.ip)
      },
      format!("A client with ip address '{}' already exists.", client.ip),
    )?;
    check(
      || {
        other_clients(clients, client)
          .iter()
          .map(|c| &c.name)
          .any(|v| v == &client.name)
      },
      format!("A client with name '{}' already exists.", client.name),
    )?;

    Ok(())
  }

  fn load(state: &AppState) -> anyhow::Result<JsonRestList<Client>> {
    JsonRestList::<Client>::load(state.app_config.clients_json())
  }

  async fn get_all(State(state): State<AppState>) -> Result<Json<Vec<Client>>> {
    load(&state)?.get_all()
  }

  async fn get(State(state): State<AppState>, Path(id): Path<u32>) -> Result<Json<Client>> {
    load(&state)?.get(id)
  }

  async fn put(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    extract::Json(client): extract::Json<Client>,
  ) -> Result<Json<Client>> {
    let clients = load(&state)?;

    validate(&clients, &client)?;
    let result = load(&state)?.put(id, client.clone());
    state.regenerate().await;

    result
  }

  async fn delete(State(state): State<AppState>, Path(id): Path<u32>) -> Result<Json<Client>> {
    let result = load(&state)?.delete(id);
    state.regenerate().await;

    result
  }

  async fn post(
    State(state): State<AppState>,
    extract::Json(client): extract::Json<Client>,
  ) -> Result<Json<Client>> {
    let mut clients = load(&state)?;
    validate(&clients, &client)?;
    let result = clients.add(client.clone());
    state.regenerate().await;

    result
  }
}

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

  fn load(state: &AppState) -> anyhow::Result<JsonRestList<DomainList>> {
    JsonRestList::<DomainList>::load(state.app_config.domains_json())
  }

  async fn get_all(State(state): State<AppState>) -> Result<Json<Vec<DomainList>>> {
    load(&state)?.get_all()
  }

  async fn get(State(state): State<AppState>, Path(id): Path<u32>) -> Result<Json<DomainList>> {
    load(&state)?.get(id)
  }

  async fn put(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    extract::Json(client): extract::Json<DomainList>,
  ) -> Result<Json<DomainList>> {
    load(&state)?.put(id, client)
  }

  async fn delete(State(state): State<AppState>, Path(id): Path<u32>) -> Result<Json<DomainList>> {
    load(&state)?.delete(id)
  }

  async fn post(
    State(state): State<AppState>,
    extract::Json(client): extract::Json<DomainList>,
  ) -> Result<Json<DomainList>> {
    let mut clients = load(&state)?;

    clients.add(client)
  }
}

mod netaccess {
  use std::collections::HashMap;

  use crate::model::NetAccess;

  use super::*;
  use anyhow::anyhow;

  pub(super) fn routes() -> Router<AppState> {
    Router::new()
      .route("/", routing::get(get_all))
      // .route("/", routing::post(post))
      // .route("/:mac", routing::get(get))
      // .route("/:mac", routing::put(put))
      // .route("/:mac", routing::delete(delete))
  }

  async fn get_all_netaccess(state: AppState) -> anyhow::Result<Vec<NetAccess>> {
    let mut unifi_client = state.unifi_client.lock().await;
    let unifi_client = unifi_client.as_mut();
    match unifi_client {
      None => Err(anyhow!("No unifi client")),
      Some(client) => {
        // let rules = client.get_traffic_rules().await.or_else(|r| Err(MyError::Failed(r)));
        let rules = client.get_traffic_rules().await?;

        let mut mac_to_state = HashMap::new();
        for rule in rules {
          if rule.action == "BLOCK" && rule.matching_target == "INTERNET" {
            for device in rule.target_devices {
              mac_to_state.insert(device.client_mac, rule.enabled);
            }
          }
        }

        let mut result = Vec::new();
        for (mac, enabled) in mac_to_state.iter() {
          result.push(NetAccess {
            mac_address: Some(mac.to_owned()),
            enabled: !*enabled
          })
        }
        Ok(result)
      }
    }
  }

  async fn get_all(State(state): State<AppState>) -> Result<Json<Vec<NetAccess>>> {
    Ok(Json(get_all_netaccess(state).await?))
  }
}