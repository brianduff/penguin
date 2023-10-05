use reqwest::{Response, Client, RequestBuilder, Method};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use serde_json::json;
use anyhow::{anyhow, Result};

pub struct UnifiClient {
  device_url: String,
  cookie: Option<String>,
  csrf_token: Option<String>,
  username: String,
  password: String
}

impl UnifiClient {
  pub fn new(username: &str, password: &str) -> Self {
    UnifiClient { username: username.to_owned(), password: password.to_owned(), ..Default::default() }
  }

  fn url_path(&self, path: &str) -> String {
    format!("{}{}", self.device_url, path)
  }

  fn update_cookie(&mut self, response: &Response) -> Result<()> {
    if let Some(cookie) = response.headers().get("set-cookie") {
      self.cookie = Some(cookie.to_str()?.to_owned());
    }

    if let Some(token) = response.headers().get("x-csrf-token") {
      self.csrf_token = Some(token.to_str()?.to_owned());
    }

    Ok(())
  }

  fn attach_cookie(&mut self, mut req: RequestBuilder) -> Result<RequestBuilder> {
    let cookie = self.cookie.as_ref().ok_or(anyhow!("Not logged in"))?;
    let cookie_parts = cookie.split("; ");
    for part in cookie_parts {
      let mut kv = part.split('=');
      let key = kv.next();
      let value = kv.next();

      if matches!(key, Some("TOKEN")) {
        if let Some(jwt) = value {
          req = req.header("Cookie", format!("TOKEN={}", jwt));

          if let Some(token) = self.csrf_token.as_ref() {
            return Ok(req.header("X-Csrf-Token", token));
          }
        }
      }
    }

    self.cookie = None;
    self.csrf_token = None;
    Err(anyhow!("Invalid cookie. Try login() again"))
  }

  fn reqwest_client() -> Result<Client> {
    Ok(reqwest::Client::builder().danger_accept_invalid_certs(true).build()?)
  }

  fn json_request(&mut self, method: Method, path: &str) -> Result<RequestBuilder> {
    let req = Self::reqwest_client()?.request(method, self.url_path(path));
    Ok(self.attach_cookie(req)?.header("Content-Type", "application/json"))
  }

  async fn handle_response<T: DeserializeOwned>(&mut self, response: Response) -> Result<T> {
    self.update_cookie(&response)?;
    response.error_for_status_ref()?;
    Ok(response.json().await?)
  }

  async fn req<RQ: Serialize, RS: DeserializeOwned>(&mut self, method: Method, path: &str, body: &RQ) -> Result<RS> {
    let response =
      self.json_request(method, path)
      ?.body(serde_json::to_string_pretty(body)?)
      .send()
      .await?;

    self.handle_response(response).await
  }

  async fn get<RS: DeserializeOwned>(&mut self, path: &str) -> Result<RS> {
    let response =
      self.json_request(Method::GET, path)?
      .send()
      .await?;

    self.handle_response(response).await
  }

  pub async fn get_traffic_rules(&mut self) -> Result<Vec<TrafficRule>> {
    self.get("proxy/network/v2/api/site/default/trafficrules").await
 }

  pub async fn create_traffic_rule(&mut self, rule: &TrafficRule) -> Result<TrafficRule> {
    self.req(Method::POST, "proxy/network/v2/api/site/default/trafficrules", rule).await
  }

  pub async fn update_traffic_rule(&mut self, rule: &TrafficRule) -> Result<TrafficRule> {
    let id = rule.id.as_ref().ok_or(anyhow!("Rule doesn't have id: {:?}", rule))?;
    self.req(Method::PUT, &format!("proxy/network/v2/api/site/default/trafficrules/{}", id), rule).await
  }

  pub async fn login(&mut self) -> Result<()> {
    let response = Self::reqwest_client()?.post(self.url_path("api/auth/login"))
      .header("Content-Type", "application/json")
      .body(json!({
        "username": self.username,
        "password": self.password,
        "rememberMe": false,
        "token": ""
      }).to_string())
      .send()
      .await?;

    response.error_for_status_ref()?;
    self.update_cookie(&response)?;

    Ok(())
  }
}

impl Default for UnifiClient {
  fn default() -> Self {
    Self {
      device_url: "https://192.168.1.1/".to_owned(),
      cookie: None,
      csrf_token: None,
      username: "".to_owned(),
      password: "".to_owned()
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TrafficRule {
  #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  pub action: String,
  pub app_category_ids: Vec<String>,
  pub app_ids: Vec<String>,
  pub bandwidth_limit: BandwidthLimit,
  pub description: String,
  pub domains: Vec<String>,
  pub enabled: bool,
  pub ip_addresses: Vec<String>,
  pub ip_ranges: Vec<String>,
  pub matching_target: String,
  pub network_ids: Vec<String>,
  pub regions: Vec<String>,
  pub schedule: Schedule,
  pub target_devices: Vec<TargetDevice>
}

impl Default for TrafficRule {
  fn default() -> Self {
    Self {
      id: None,
      action: "BLOCK".to_owned(),
      app_category_ids: Default::default(),
      app_ids: Default::default(),
      bandwidth_limit: BandwidthLimit::disabled(),
      description: "".to_owned(),
      domains: Default::default(),
      enabled: false,
      ip_addresses: Default::default(),
      ip_ranges: Default::default(),
      matching_target: "APP".to_owned(),
      network_ids: Default::default(),
      regions: Default::default(),
      schedule: Schedule::always(),
      target_devices: Default::default()
    }
  }
}

impl TrafficRule {
  pub fn block_internet() -> Self {
    TrafficRule {
      matching_target: "INTERNET".to_owned(),
      ..Default::default()
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BandwidthLimit {
  pub download_limit_kbps: u32,
  pub enabled: bool,
  pub upload_limit_kbps: u32,
}

impl BandwidthLimit {
  pub fn disabled() -> Self {
    BandwidthLimit::default()
  }
}

impl Default for BandwidthLimit {
  fn default() -> Self {
    Self { download_limit_kbps: 1024, enabled: false, upload_limit_kbps: 1024 }
  }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Schedule {
  pub mode: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date_start: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub date_end: Option<String>,
  pub repeat_on_days: Vec<u32>,
  pub time_all_day: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub time_range_start: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub time_range_end: Option<String>
}

impl Schedule {
  pub fn always() -> Self {
    Schedule {
      mode: "ALWAYS".to_string(),
      ..Default::default()
    }
  }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TargetDevice {
  pub client_mac: String,
  #[serde(rename="type")]
  pub typ: String
}

impl TargetDevice {
  pub fn for_client_mac(mac: &str) -> Self {
    TargetDevice {
      client_mac: mac.to_owned(),
      ..Default::default()
    }
  }
}

impl Default for TargetDevice {
  fn default() -> Self {
    Self {
      client_mac: Default::default(),
      typ: "CLIENT".to_owned()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[ignore]  // Don't actually run this test except locally, since it'll try to connect and do stuff.
  #[tokio::test]
  async fn test_unifi() {
    let mut unifi = UnifiClient::new("USERNAMEGOESHERE", "PASSWORDGOESHERE");
    unifi.login().await.unwrap();
    println!("Traffic rules: {:?}", unifi.get_traffic_rules().await.unwrap());

    let mut rule = TrafficRule::block_internet();
    rule.target_devices.push(TargetDevice::for_client_mac("00:23:62:00:08:67"));
    rule.description = "This is a test".to_owned();
    let mut rule = unifi.create_traffic_rule(&rule).await.unwrap();
    println!("Created traffic rule: {:?}", rule);

    rule.enabled = true;
    unifi.update_traffic_rule(&rule).await.unwrap();
    println!("Enabled rule");
  }
}