use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Client {
  pub id: i32,
  pub ip: String,
  pub name: String,
  pub rules: Vec<Rule>,
  pub leases: Vec<Lease>
}

#[derive(Serialize, Deserialize)]
pub struct Rule {
  pub kind: String,
  pub domainlists: Vec<i32>
}

#[derive(Serialize, Deserialize)]
pub struct Lease {
  pub end_date: NaiveDateTime,
  pub rule: Rule
}

#[derive(Serialize, Deserialize)]
pub struct DomainList {
  pub id: i32,
  pub name: String,
  pub domains: Vec<String>
}
