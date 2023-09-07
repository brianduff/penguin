use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use axum::Json;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::file::create_file;
use crate::list::{Identifiable, IdentifiedList};
use crate::errors::Result;
use crate::errors::MyError;

pub struct JsonRestList<T: Identifiable + Clone + Serialize + DeserializeOwned> {
  pub list: IdentifiedList<T>,
  path: PathBuf
}

impl<T: Identifiable + Clone + Serialize + DeserializeOwned> JsonRestList<T> {
  pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
    let items = if path.as_ref().exists() {
      let file = File::open(&path)?;
      let reader = BufReader::new(file);

      let data: Vec<T> = serde_json::from_reader(reader)?;
      data
    } else {
      Vec::new()
    };
    Ok(Self {
      list: IdentifiedList::new(items),
      path: path.as_ref().to_owned()
    })
  }

  fn save(&self) -> anyhow::Result<()> {
    let file = create_file(&self.path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &self.list.items)?;

    Ok(())
  }

  pub fn get_all(&self) -> Result<Json<Vec<T>>> {
    Ok(axum::Json(self.list.items.clone()))
  }

  pub fn get(&self, id: u32) -> Result<Json<T>> {
    match self.list.items.iter().find(|c| c.id() == Some(id)) {
      Some(c) => {
        Ok(axum::Json(c.clone()))
      },
      None => {
        Err(MyError::NotFound)
      }
    }
  }

  pub fn put(&mut self, id: u32, item: T) -> Result<Json<T>> {
    let result = self.list.update(id, item);
    if let Some(result) = result {
      let rv = result.clone();
      self.save()?;
      Ok(Json(rv))
    } else {
      Err(MyError::NotFound)
    }
  }

  pub fn delete(&mut self, id: u32) -> Result<Json<T>> {
    let result = self.list.delete(id);
    if let Some(result) = result {
      self.save()?;
      Ok(Json(result))
    } else {
      Err(MyError::NotFound)
    }
  }

  pub fn add(&mut self, item: T) -> Result<Json<T>> {
    let result = self.list.add(item).clone();
    self.save()?;
    Ok(Json(result))
  }
}
