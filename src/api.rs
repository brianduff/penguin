use actix_web::web::scope;
use actix_web::Scope;
use actix_web::{get, put, Responder, web};
use crate::config::Config;
use crate::errors::Result;
use crate::errors::MyError;

static CONFIG_DIR: &str = "config";

pub fn api_service() -> Scope {
  scope("/api/v1")
    .service(clients::service())
    .service(domains::service())
}

mod clients {

use crate::model::Client;

use super::*;

  pub(super) fn service() -> Scope {
    scope("/client")
        .service(get_all)
        .service(get)
        .service(put)
  }

  #[get("")]
  async fn get_all() -> Result<impl Responder> {
    Ok(web::Json(Config::load(CONFIG_DIR)?.clients))
  }

  #[get("/{id}")]
  async fn get(params: web::Path<(u32,)>) -> Result<impl Responder> {
    let c = Config::load(CONFIG_DIR)?.clients;
    match c.into_iter().find(|c| c.id == Some(params.0)) {
      Some(c) => {
        Ok(web::Json(c))
      },
      None => {
        Err(MyError::NotFound)
      }
    }
  }

  #[put("")]
  async fn put(client: web::Json<Client>) -> Result<impl Responder> {
    let mut config = Config::load(CONFIG_DIR)?;
    config.add_client(&client);
    config.save()?;

    Ok(client)
  }

}

mod domains {
  use super::*;

  pub(super) fn service() -> Scope {
    scope("/domain")
  }
}

