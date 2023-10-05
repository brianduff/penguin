use std::process::Command;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};

/// Request that squid reload its configuration.
/// This requires an entry in /etc/sudoers, otherwise it'll prompt for a
/// password and fail.
pub fn reload_config() {
  let output = Command::new("sudo")
    .args(["pkill", "-HUP", "squid"])
    .output();
  if let Err(e) = output {
    tracing::error!("Failed to HUP squid: {:?}", e);
  } else {
    tracing::info!("Successfully sent HUP to squid");
  }
}

#[derive(PartialEq)]
struct LogEntry {
  date: DateTime<Utc>,
  response_time_millis: u64,
  client_ip: String,
  client_fqdn: String,
  status_code: String,
  request_size_bytes: u64,
  request_method: String,
  request_url: String,
  username: String,
  peer_fqdn: String,
  mime_type: String,
}

impl LogEntry {
  #[rustfmt::skip]
  fn parse(s: &str) -> Result<Self> {
    let mut parts = s.split_whitespace();

    let first = parts.next().ok_or(anyhow!("Missing date"))?.to_owned();
    let date = DateTime::parse_from_str(&first, "%Y-%m-%dT%H:%M:%S%.3f%z").with_context(|| format!("Date '{}' didn't parse", first))?.with_timezone(&Utc);
    let response_time_millis = parts.next().ok_or(anyhow!("Missing response_time_millis"))?.parse()?;
    let client_ip = parts.next().ok_or(anyhow!("Missing client_up"))?.to_owned();
    let client_fqdn = parts.next().ok_or(anyhow!("Missing client_fqdn"))?.to_owned();
    let status_code = parts.next().ok_or(anyhow!("Missing status_code"))?.to_owned();
    let request_size_bytes = parts.next().ok_or(anyhow!("Missing request_size_bytes"))?.parse()?;
    let request_method = parts.next().ok_or(anyhow!("Missing request_method"))?.to_owned();
    let request_url = parts.next().ok_or(anyhow!("Missing request_url"))?.to_owned();
    let username = parts.next().ok_or(anyhow!("Missing username"))?.to_owned();
    let peer_fqdn = parts.next().ok_or(anyhow!("Missing peer_fqdn"))?.to_owned();
    let mime_type = parts.next().ok_or(anyhow!("Missing mime_type"))?.to_owned();

    Ok(LogEntry {
      date,
      response_time_millis,
      client_ip,
      client_fqdn,
      status_code,
      request_size_bytes,
      request_method,
      request_url,
      username,
      peer_fqdn,
      mime_type
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use chrono::TimeZone;

  #[test]
  fn check_parse() -> Result<()> {
    let log_line = "2023-10-04T00:14:09.000-0700   1374 192.168.1.224 192.168.1.224 TCP_TUNNEL/200 10630 CONNECT weather-data.apple.com:443 - HIER_DIRECT/weather-data.apple.com -";

    let entry = LogEntry::parse(log_line)?;


    let date = Utc.with_ymd_and_hms(2023, 10, 4, 7, 14, 9).unwrap();
    let expected = LogEntry {
      date,
      response_time_millis: 1374,
      client_ip: "192.168.1.224".to_owned(),
      client_fqdn: "192.168.1.224".to_owned(),
      status_code: "TCP_TUNNEL/200".to_owned(),
      request_size_bytes: 10630,
      request_method: "CONNECT".to_owned(),
      request_url: "weather-data.apple.com:443".to_owned(),
      username: "-".to_owned(),
      peer_fqdn: "HIER_DIRECT/weather-data.apple.com".to_owned(),
      mime_type: "-".to_owned()
    };

    assert!(expected == entry);

    Ok(())

  }
}
