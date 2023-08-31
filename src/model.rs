use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Client {
  pub id: Option<u32>,
  pub ip: String,
  pub name: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub rules: Vec<Rule>,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub leases: Vec<Lease>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Rule {
  pub kind: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub domainlists: Vec<u32>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Lease {
  pub end_date: NaiveDateTime,
  pub rule: Rule
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DomainList {
  pub id: Option<u32>,
  pub name: String,
  #[serde(skip_serializing_if = "Vec::is_empty", default)]
  pub domains: Vec<String>
}
