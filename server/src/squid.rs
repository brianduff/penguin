use std::{
  fs::File,
  io::{BufReader, Read},
  path::PathBuf,
  process::Command,
};

use anyhow::{anyhow, Context, Result};
use chrono::{serde::ts_milliseconds, DateTime, Utc};
use flate2::read::GzDecoder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use tracing::{error, warn};

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

#[derive(PartialEq, Debug, Serialize)]
pub enum ActiveState {
  Active,
  Deactivating,
  Inactive,
  Unknown,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
  active: ActiveState,
}

impl ServiceStatus {
  fn from_output(output: &str) -> ServiceStatus {
    let re = Regex::new(r" +Active: ([^ ]+) ").unwrap();

    let capture = re.captures_iter(output).next();

    let active = if let Some(capture) = capture {
      let status = capture.get(1);
      if let Some(status) = status {
        match status.as_str() {
          "active" => ActiveState::Active,
          "deactivating" => ActiveState::Deactivating,
          "inactive" => ActiveState::Inactive,
          _ => ActiveState::Unknown,
        }
      } else {
        ActiveState::Unknown
      }
    } else {
      warn!("Unexpected output from service status: {}", output);
      ActiveState::Unknown
    };

    ServiceStatus { active }
  }
}

/// Get the current status of the squid service.
pub fn get_status() -> ServiceStatus {
  let output = Command::new("/usr/sbin/service")
    .args(["squid", "status"])
    .output();

  match output {
    Ok(out) => {
      let code = out.status.code().ok_or(-1).unwrap();
      if (0..=4).contains(&code) {
        ServiceStatus::from_output(&String::from_utf8(out.stdout).unwrap())
      } else {
        error!("/usr/sbin/service status squid failed: {}\n{}",
          String::from_utf8(out.stdout).unwrap(),
          String::from_utf8(out.stderr).unwrap());
        ServiceStatus {
          active: ActiveState::Unknown
        }
      }
    },
    Err(e) => {
      error!("Failed to get squid status: {:?}", e);
      ServiceStatus {
        active: ActiveState::Unknown
      }
    }
  }

  // if let Err(e) = output {
  //   tracing::error!("Failed to HUP squid: {:?}", e);
  // } else {
  //   tracing::info!("Successfully sent HUP to squid");
  // }
}

/// Gets all Squid logs by reading files in the log directory.
/// TODO: hoist log location out into a config variable.
pub fn get_all_logs() -> Result<Vec<LogEntry>> {
  let log_dir = PathBuf::from("/var/log/squid");

  let mut result = Vec::new();
  if log_dir.exists() {
    for entry in log_dir.read_dir()? {
      let path = entry?.path();
      if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_str().ok_or_else(|| anyhow!("Invalid path"))?;
        if file_name.starts_with("access.log") {
          let file = File::open(&path)?;
          if file_name.ends_with(".gz") {
            read_logs(GzDecoder::new(file), &mut result)?;
          } else {
            read_logs(file, &mut result)?;
          };
        }
      }
    }
  }

  Ok(result)
}

fn read_logs<R: Read>(read: R, out: &mut Vec<LogEntry>) -> Result<()> {
  let buf = BufReader::new(read);
  for line in buf.lines() {
    out.push(LogEntry::parse(&line?)?);
  }

  Ok(())
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct LogEntry {
  #[serde(with = "ts_milliseconds")]
  date: DateTime<Utc>,
  response_time_millis: u64,
  pub client_ip: String,
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
      mime_type: "-".to_owned(),
    };

    assert!(expected == entry);

    Ok(())
  }

  #[test]
  fn check_status_active() {
    let output = r#"
    ● squid.service - Squid Web Proxy Server
     Loaded: loaded (/lib/systemd/system/squid.service; enabled; vendor preset: enabled)
     Active: active (running) since Mon 2024-03-04 23:28:53 PST; 2min 20s ago
       Docs: man:squid(8)
    Process: 4196 ExecStartPre=/usr/sbin/squid --foreground -z (code=exited, status=0/SUCCESS)
   Main PID: 4200 (squid)
      Tasks: 4 (limit: 1069)
     Memory: 16.9M
        CPU: 74ms
     CGroup: /system.slice/squid.service
             ├─4200 /usr/sbin/squid --foreground -sYC
             ├─4202 "(squid-1)" --kid squid-1 --foreground -sYC
             ├─4203 "(logfile-daemon)" /var/log/squid/access.log
             └─4204 "(pinger)"

  Notice: journal has been rotated since unit was started, output may be incomplete.
    "#;

    let status = ServiceStatus::from_output(output);
    assert_eq!(status.active, ActiveState::Active);
  }

  #[test]
  fn check_status_inactive() {
    let output = r#"
    ○ squid.service - Squid Web Proxy Server
    Loaded: loaded (/lib/systemd/system/squid.service; enabled; vendor preset: enabled)
    Active: inactive (dead) since Mon 2024-03-04 23:48:11 PST; 13s ago
      Docs: man:squid(8)
   Process: 4537 ExecStartPre=/usr/sbin/squid --foreground -z (code=exited, status=0/SUCCESS)
   Process: 4540 ExecStart=/usr/sbin/squid --foreground -sYC (code=exited, status=0/SUCCESS)
  Main PID: 4540 (code=exited, status=0/SUCCESS)
       CPU: 107ms

Mar 04 23:48:11 ambitious-mealworm squid[4542]:   Finished.  Wrote 0 entries.
Mar 04 23:48:11 ambitious-mealworm squid[4542]:   Took 0.00 seconds (  0.00 entries/sec).
Mar 04 23:48:11 ambitious-mealworm squid[4542]: Logfile: closing log daemon:/var/log/squid/ac>
Mar 04 23:48:11 ambitious-mealworm squid[4542]: Logfile Daemon: closing log daemon:/var/log/s>
Mar 04 23:48:11 ambitious-mealworm squid[4542]: Open FD UNSTARTED    10 IPC UNIX STREAM Parent
Mar 04 23:48:11 ambitious-mealworm squid[4542]: Squid Cache (Version 5.7): Exiting normally.
Mar 04 23:48:11 ambitious-mealworm squid[4540]: Squid Parent: squid-1 process 4542 exited wit>
Mar 04 23:48:11 ambitious-mealworm squid[4540]: Removing PID file (/run/squid.pid)
Mar 04 23:48:11 ambitious-mealworm systemd[1]: squid.service: Deactivated successfully.
Mar 04 23:48:11 ambitious-mealworm systemd[1]: Stopped Squid Web Proxy Server.
    "#;

    let status = ServiceStatus::from_output(output);
    assert_eq!(status.active, ActiveState::Inactive);
  }

  #[test]
  fn check_status_unknown() {
    let output = "Random stuff";
    let status = ServiceStatus::from_output(output);
    assert_eq!(status.active, ActiveState::Unknown);
  }
}
