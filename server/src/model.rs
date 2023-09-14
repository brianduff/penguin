use std::path::PathBuf;

use chrono::NaiveDateTime;
use confique::Config;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct Client {
  pub id: Option<u32>,
  pub ip: String,
  pub name: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub rules: Vec<Rule>,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub leases: Vec<Lease>
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct Rule {
  pub kind: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub domainlists: Vec<u32>
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct Lease {
  pub end_date: NaiveDateTime,
  pub rule: Rule
}

#[derive(Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct DomainList {
  pub id: Option<u32>,
  pub name: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub domains: Vec<String>
}

// App wide configuration
#[derive(Config, Clone)]
pub struct Conf {
  #[config(default = "config")]
  pub config_dir: String,
  #[config(default = "squid")]
  pub squid_config_dir: String
}

impl Conf {
  pub fn load() -> anyhow::Result<Conf> {
    Ok(Conf::builder()
        .env()
        .file("/opt/penguin/penguin.toml")
        .file("penguin.toml")
        .load()?)
  }

  pub fn config_path(&self) -> PathBuf {
    PathBuf::from(&self.config_dir)
  }

  pub fn clients_json(&self) -> PathBuf {
    self.config_path().join("clients.json")
  }

  pub fn domains_json(&self) -> PathBuf {
    self.config_path().join("domains.json")
  }
}