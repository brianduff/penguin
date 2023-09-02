use actix_web::web::{scope, Json};
use actix_web::Scope;
use actix_web::{delete, get, put, Responder,post,  web};
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::config::{Config, Identifiable, ManagedList};
use crate::errors::Result;
use crate::errors::MyError;
use crate::model::Client;

static CONFIG_DIR: &str = "config";

pub fn api_service() -> Scope {
  scope("/api/v1")
    .service(clients::service())
    .service(domains::service())
}

mod clients {

  use super::*;

  pub(super) fn service() -> Scope {
    scope("/client")
        .service(get_all)
        .service(get)
        .service(put)
        .service(post)
        .service(delete)
  }

  #[get("")]
  async fn get_all() -> Result<impl Responder> {
    let mut config = Config::load(CONFIG_DIR)?;
    ManagedListRest::new(&mut config.clients).get_all()
  }

  #[get("/{id}")]
  async fn get(params: web::Path<(u32,)>) -> Result<impl Responder> {
    ManagedListRest::new(&mut Config::load(CONFIG_DIR)?.clients).get(params.0)
  }

  #[put("/{id}")]
  async fn put(params: web::Path<(u32,)>, client: web::Json<Client>) -> Result<impl Responder> {
    let mut config = Config::load(CONFIG_DIR)?;
    let result = ManagedListRest::new(&mut config.clients).put(params.0, client);
    config.save()?;

    result
  }

  #[delete("/{id}")]
  async fn delete(params: web::Path<(u32,)>) -> Result<impl Responder> {
    let mut config = Config::load(CONFIG_DIR)?;
    let result = ManagedListRest::new(&mut config.clients).delete(params.0);
    config.save()?;

    result
  }

  #[post("")]
  async fn post(client: web::Json<Client>) -> Result<impl Responder> {
    let mut config = Config::load(CONFIG_DIR)?;
    let result = ManagedListRest::new(&mut config.clients).post(client);
    config.save()?;

    result
  }

}

mod domains {
  use super::*;

  pub(super) fn service() -> Scope {
    scope("/domain")
  }
}



struct ManagedListRest<'a, T: Identifiable + Clone + Serialize + DeserializeOwned> {
  list: &'a mut ManagedList<T>
}

impl<'a, T: Identifiable + Clone + Serialize + DeserializeOwned> ManagedListRest<'a, T> {
  fn new(list: &'a mut ManagedList<T>) -> Self {
    Self {
      list
    }
  }

  fn get_all(&self) -> Result<impl Responder> {
    Ok(web::Json(self.list.items.clone()))
  }

  fn get(&self, id: u32) -> Result<impl Responder> {
    match self.list.items.iter().find(|c| c.id() == Some(id)) {
      Some(c) => {
        Ok(web::Json(c.clone()))
      },
      None => {
        Err(MyError::NotFound)
      }
    }
  }

  fn put(&mut self, id: u32, item: Json<T>) -> Result<impl Responder> {
    match self.list.update(id, item.into_inner()) {
      Some(v) => Ok(web::Json(v.clone())),
      None => Err(MyError::NotFound)
    }
  }

  fn delete(&mut self, id: u32) -> Result<impl Responder> {
    match self.list.delete(id) {
      Some(v) => Ok(web::Json(v)),
      None => Err(MyError::NotFound)
    }
  }

  fn post(&mut self, item: Json<T>) -> Result<impl Responder> {
    Ok(web::Json(self.list.add(item.into_inner()).clone()))
  }
}
