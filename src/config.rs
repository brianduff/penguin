use std::{path::{Path, PathBuf}, fs::File, io::{BufReader, BufWriter}};
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

use crate::model::{Client, DomainList};

pub struct Config {
  paths: ConfigPaths,
  pub clients: Vec<Client>,
  pub domains: Vec<DomainList>
}

impl Config {
  pub fn load<P: AsRef<Path>>(dir: P) -> Result<Config> {
    let paths = ConfigPaths::new(dir);

    Ok(Config {
      clients: load_file(&paths.clients)?,
      domains: load_file(&paths.domains)?,
      paths,
    })
  }

  pub fn save(&self) -> Result<()> {
    write_file(&self.paths.clients, &self.clients)?;
    write_file(&self.paths.domains, &self.domains)?;

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
  serde_json::to_writer(&mut writer, &value)?;

  Ok(())
}