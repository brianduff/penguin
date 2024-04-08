use anyhow::Result;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use chrono::serde::{ts_milliseconds, ts_milliseconds_option};
use chrono::{DateTime, NaiveDateTime, Utc};
use confique::{Config, Builder};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Serialize, Deserialize, Clone, TS)]
//#[ts(export)]
pub struct Client {
  pub id: Option<u32>,
  pub ip: String,
  pub name: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub rules: Vec<Rule>,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub leases: Vec<Lease>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mac_address: Option<String>
}

#[derive(Copy, Clone, TS, Serialize, Deserialize, PartialEq)]
pub enum RuleKind {
  #[serde(rename = "allow_http_access")]
  AllowHttpAccess,
  #[serde(rename = "deny_http_access")]
  DenyHttpAccess
}

#[derive(Serialize, Deserialize, Clone, TS)]
//#[ts(export)]
pub struct Rule {
  pub kind: RuleKind,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub domainlists: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, TS)]
//#[ts(export)]
pub struct Lease {
  #[serde(with = "ts_milliseconds_option")]
  pub end_date_utc: Option<DateTime<Utc>>,
  #[deprecated(note = "Use end_date_utc")]
  pub end_date: Option<NaiveDateTime>,
  pub rule: Rule,
}

#[derive(Serialize, Deserialize, Clone, TS)]
//#[ts(export)]
pub struct DomainList {
  pub id: Option<u32>,
  pub name: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub domains: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, TS, Debug)]
pub struct NetAccess {
  pub mac_address: String,
  /// Is internet access enabled for this mac address?
  pub enabled: bool,

  /// At around the given time, access will automatically be disabled.
  #[serde(with = "ts_milliseconds_option")]
  pub auto_disable_at: Option<DateTime<Utc>>
}

// A configuration value that's persisted to disk in a map of
// mac address -> NetAccessSettings.
#[derive(Serialize, Deserialize, Clone, TS, Debug)]
pub struct NetAccessConfig {
  #[serde(with = "ts_milliseconds")]
  pub auto_disable_at: DateTime<Utc>
}

#[derive(Config, Clone, Debug)]
pub struct UnifiConfig {
  #[config(default = false)]
  pub enabled: bool,
  #[config(default = "https://192.168.1.1/")]
  pub url: String,
  pub username: Option<String>,
  pub password: Option<String>,
}

impl Default for UnifiConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      url: "https://192.168.1.1/".to_owned(),
      username: None,
      password: None
    }
  }
}

// App wide configuration
#[derive(Config, Clone, Debug)]
pub struct Conf {
  #[config(default = "config")]
  pub config_dir: String,
  #[config(default = "squid")]
  pub squid_config_dir: String,
  #[config(default = "logs")]
  pub squid_log_dir: String,
  #[config(default = false)]
  pub hup_squid_daemon: bool,

  #[config(default = false)]
  pub require_auth: bool,

  #[config(default = [])]
  pub authorized_users: Vec<String>,

  #[config(nested)]
  pub unifi: UnifiConfig,
}

impl Conf {

  fn load_from_dir<P: Into<PathBuf>>(mut builder: Builder<Conf>, path: P) -> anyhow::Result<Builder<Conf>> {
    let path : PathBuf = path.into();
    if path.is_dir() {
      for dir in path.read_dir()? {
        let entry = dir?.path();
        if entry.extension() == Some(OsStr::new("toml")) {
          builder = builder.file(entry);
        }
      }
    }

    Ok(builder)
  }

  pub fn load() -> anyhow::Result<Conf> {
    let mut builder = Conf::builder().env();

    builder = Self::load_from_dir(builder, "/opt/penguin/conf.d")?;
    builder = Self::load_from_dir(builder, "dev.conf.d")?;

    Ok(
      builder
        .file("/opt/penguin/penguin.toml")
        .file("penguin.toml")
        .load()?,
    )
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

  pub fn netaccess_json(&self) -> PathBuf {
    self.config_path().join("netaccess.json")
  }

  pub fn load_netaccess_config(&self) -> Result<HashMap<String, NetAccessConfig>> {
    let path = self.netaccess_json();
    let items = if path.exists() {
      let file = File::open(&path)?;
      let reader = BufReader::new(file);

      serde_json::from_reader(reader)?
    } else {
      HashMap::new()
    };

    Ok(items)
  }

  // pub fn save_netaccess_config(&self, config: HashMap<String, NetAccessConfig>) -> Result<()> {
  //   let file = create_file(self.netaccess_json())?;
  //   let mut writer = BufWriter::new(file);
  //   serde_json::to_writer_pretty(&mut writer, &config)?;

  //   Ok(())
  // }

}
