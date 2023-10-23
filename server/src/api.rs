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
use axum::middleware;

use crate::auth::auth;

pub fn api_routes() -> Router<AppState> {
  Router::new()
    .nest("/v1/client", clients::routes())
    .nest("/v1/domainlist", domains::routes())
    .nest("/v1/netaccess", netaccess::routes())
    .nest("/v1/logs/proxy", logs::proxy::routes())
}

mod clients {


use super::*;

  pub(super) fn routes() -> Router<AppState> {
    Router::new()
      .route("/", routing::get(get_all).route_layer(middleware::from_fn(auth)))
      .route("/", routing::post(post).route_layer(middleware::from_fn(auth)))
      .route("/:id", routing::get(get).route_layer(middleware::from_fn(auth)))
      .route("/:id", routing::put(put).route_layer(middleware::from_fn(auth)))
      .route("/:id", routing::delete(delete).route_layer(middleware::from_fn(auth)))
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
      .route("/", routing::get(get_all).route_layer(middleware::from_fn(auth)))
      .route("/", routing::post(post).route_layer(middleware::from_fn(auth)))
      .route("/:id", routing::get(get).route_layer(middleware::from_fn(auth)))
      .route("/:id", routing::put(put))
      .route("/:id", routing::delete(delete).route_layer(middleware::from_fn(auth)))
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

  use crate::{model::NetAccess, unifi::{TrafficRule, TargetDevice}};

  use super::*;
  use anyhow::anyhow;

  pub(super) fn routes() -> Router<AppState> {
    Router::new()
      .route("/", routing::get(get_all).route_layer(middleware::from_fn(auth)))
      .route("/:mac", routing::get(get).route_layer(middleware::from_fn(auth)))
      .route("/", routing::post(post).route_layer(middleware::from_fn(auth)))
      .route("/:mac", routing::put(put).route_layer(middleware::from_fn(auth)))
  }

  async fn get_all_netaccess(state: AppState) -> anyhow::Result<Vec<NetAccess>> {
    let mut unifi_client = state.unifi_client.lock().await;
    let unifi_client = unifi_client.as_mut();
    match unifi_client {
      None => Err(anyhow!("No unifi client")),
      Some(client) => {
        let rules = client.get_traffic_rules().await?;

        let mut mac_to_is_blocked = HashMap::new();
        for rule in rules {
          if rule.action == "BLOCK" && rule.matching_target == "INTERNET" {
            for device in rule.target_devices {
              mac_to_is_blocked.insert(device.client_mac, rule.enabled);
            }
          }
        }

        // Load the local configuration so we can get auto_disable_at values.
        let netaccess_config = state.app_config.load_netaccess_config()?;

        let mut result = Vec::new();
        for (mac, blocked) in mac_to_is_blocked.iter() {
          result.push(NetAccess {
            mac_address: mac.to_owned(),
            enabled: !*blocked,  // Access to the internet is ENABLED if the block internet rule is DISABLED
            auto_disable_at: netaccess_config.get(mac).map(|v| v.auto_disable_at)
          })
        }

        Ok(result)
      }
    }
  }

  async fn get_all(State(state): State<AppState>) -> Result<Json<Vec<NetAccess>>> {
    Ok(Json(get_all_netaccess(state).await?))
  }

  async fn get(State(state): State<AppState>, Path(mac): Path<String>) -> Result<Json<NetAccess>> {
    let all = get_all_netaccess(state).await?;

    for access in all {
      if access.mac_address == mac.clone() {
        return Ok(Json(access));
      }
    }

    Err(MyError::NotFound)
  }

  async fn post(State(state): State<AppState>, extract::Json(access): extract::Json<NetAccess>) -> Result<Json<NetAccess>> {
    let all = get_all_netaccess(state.clone()).await?;
    for a in all {
      if a.mac_address == access.mac_address {
        return Err(MyError::BadRequest("Access already exists".to_owned()))
      }
    }

    let new_rule = create_block_rule(&access.mac_address, access.enabled);

    let mut unifi_client = state.unifi_client.lock().await;
    let unifi_client = unifi_client.as_mut();

    match unifi_client {
      None => Err(MyError::BadRequest("No unifi client".to_owned())),
      Some(client) => {
        client.create_traffic_rule(&new_rule).await?;

        Ok(Json(access))
      }
    }
  }

  fn create_block_rule(mac_address: &str, enabled: bool) -> TrafficRule {
    let mut new_rule = TrafficRule::block_internet();
    new_rule.enabled = enabled;
    new_rule.description = "Traffic rule created by Penguin".to_owned();
    new_rule.target_devices = Vec::new();
    new_rule.target_devices.push(TargetDevice::for_client_mac(mac_address));

    new_rule
  }

  async fn put(State(state): State<AppState>, Path(mac): Path<String>, extract::Json(access): extract::Json<NetAccess>) -> Result<Json<NetAccess>> {
    if access.mac_address != mac {
      return Err(MyError::BadRequest("Mac address doesn't match".to_owned()));
    }

    let mut unifi_client = state.unifi_client.lock().await;
    let unifi_client = unifi_client.as_mut();
    match unifi_client {
      None => Err(MyError::BadRequest("No unifi client".to_owned())),
      Some(client) => {
        let mut rules = client.get_traffic_rules().await?;

        for rule in rules.iter_mut().filter(|r| r.action == "BLOCK" && r.matching_target == "INTERNET") {
          // If this rule has a single target device and it matches, just make sure the enabled flag
          // is correct.
          if rule.target_devices.len() == 1 && rule.target_devices.first().unwrap().client_mac == mac {
            if rule.enabled != access.enabled {
              rule.enabled = access.enabled;
              client.update_traffic_rule(rule).await?;
            }
          } else if let Some(pos) = rule.target_devices.iter().position(|d| d.client_mac == mac) {
            if rule.enabled != access.enabled {
              // We need to pull this mac address out of this rule into its own separate rule. First, create the new rule.
              let new_rule = create_block_rule(&mac, access.enabled);
              client.create_traffic_rule(&new_rule).await?;

              // Now remove the target device from its existing rule.
              rule.target_devices.remove(pos);
              client.update_traffic_rule(rule).await?;
            }
          }
        }

        Ok(Json(access))
      }
    }
  }

}


mod logs {
  use super::*;

  pub(super) mod proxy {
    use axum::extract::Query;
    use serde::Deserialize;
    use serde_with::{NoneAsEmptyString, serde_as};
    use crate::squid::{LogEntry, get_all_logs};

    use super::*;

    pub fn routes() -> Router<AppState> {
      Router::new()
        .route("/", routing::get(get_all))
    }

    #[serde_as]
    #[derive(Deserialize)]
    struct LogQuery {
      #[serde_as(as = "NoneAsEmptyString")]
      client_id: Option<u32>
    }

    async fn get_all(State(state): State<AppState>, query: Option<Query<LogQuery>>) -> Result<Json<Vec<LogEntry>>> {
      let mut logs = get_all_logs()?;

      if let Some(query) = query {
        if let Some(client_id) = query.client_id {
          // Look up the client's ip address.
          let clients = JsonRestList::<Client>::load(state.app_config.clients_json())?;
          let ip = clients.list.items.iter().find_map(|c| if c.id.unwrap() == client_id { Some(c.ip.to_owned()) } else { None } );
          if let Some(ip) = ip {
            logs.retain(|e| e.client_ip == ip);
          } else {
            logs.clear();
          }
        }
      }

      Ok(Json(logs))
    }
  }
}