use std::{path::{Path, PathBuf}, fs::File, io::{BufReader, BufWriter}, mem};
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

use crate::model::{Client, DomainList};

pub trait Identifiable {
  fn id(&self) -> Option<u32>;
  fn set_id(&mut self, id: u32);
}

pub struct ManagedList<T: Identifiable + Clone> {
  pub items: Vec<T>
}

impl<T: Identifiable + Clone> ManagedList<T> {
  pub fn new(items: Vec<T>) -> Self {
    Self {
      items
    }
  }

  pub fn add(&mut self, item: T) -> &T {
    let mut owned = item.to_owned();

    let max_id = self.items.iter().map(|c| c.id()).fold(0, |a, b| a.max(b.unwrap()));
    owned.set_id(max_id + 1);
    self.items.push(owned);

    self.items.last().unwrap()
  }

  pub fn update(&mut self, id: u32, mut updated: T) -> Option<&T> {
    updated.set_id(id);
    match self.items.iter().position(|c| c.id() == Some(id)) {
      Some(pos) => {
        let _ = mem::replace(&mut self.items[pos], updated);
        self.items.get(pos)
      },
      None => None
    }
  }

  pub fn delete(&mut self, id: u32) -> Option<T> {
    if let Some(pos) = self.items.iter().position(|c| c.id() == Some(id)) {
      Some(self.items.remove(pos))
    } else {
      None
    }
  }

}


impl Identifiable for Client {
    fn id(&self) -> Option<u32> {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = Some(id)
    }
}

impl Identifiable for DomainList {
  fn id(&self) -> Option<u32> {
    self.id
  }

  fn set_id(&mut self, id: u32) {
      self.id = Some(id)
  }
}

pub struct Config {
  paths: ConfigPaths,
  pub clients: ManagedList<Client>,
  pub domains: ManagedList<DomainList>
}

impl Config {
  pub fn load<P: AsRef<Path>>(dir: P) -> Result<Config> {
    let paths = ConfigPaths::new(dir);

    Ok(Config {
      clients: ManagedList::new(load_file(&paths.clients)?),
      domains: ManagedList::new(load_file(&paths.domains)?),
      paths,
    })
  }

  pub fn save(&self) -> Result<()> {
    if self.paths.clients.parent().is_some_and(|p| !p.exists()) {
      std::fs::create_dir_all(self.paths.clients.parent().unwrap())?;
    }

    write_file(&self.paths.clients, &self.clients.items)?;
    write_file(&self.paths.domains, &self.domains.items)?;

    Ok(())
  }
}


struct ConfigPaths {
  clients: PathBuf,
  domains: PathBuf
}

impl ConfigPaths {
  pub fn new<P: AsRef<Path>>(dir: P) -> Self {
    Self {
      clients: dir.as_ref().join("clients.json"),
      domains: dir.as_ref().join("domains.json")
    }
  }
}

fn load_file<T: DeserializeOwned, P: AsRef<Path>>(path: &P) -> Result<Vec<T>> {
  if path.as_ref().exists() {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    Ok(serde_json::from_reader(reader)?)
  } else {
    Ok(Vec::new())
  }
}

fn write_file<T: Serialize, P: AsRef<Path>>(path: &P, value: &Vec<T>) -> Result<()> {
  let file = File::create(path)?;
  let mut writer = BufWriter::new(file);
  serde_json::to_writer_pretty(&mut writer, &value)?;

  Ok(())
}