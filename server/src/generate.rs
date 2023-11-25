use crate::file::create_writer;
use crate::list::Identifiable;
use crate::model::RuleKind;
use crate::{
  list::IdentifiedList,
  model::{Client, DomainList},
};
use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::Path;

impl Identifiable for u32 {
  fn id(&self) -> Option<u32> {
    Some(*self)
  }

  fn set_id(&mut self, _: u32) {
    unimplemented!()
  }
}

fn id_string<T: Identifiable>(type_name: &str, item: &T) -> String {
  format!("{}_{:0>4}", type_name, item.id().unwrap())
}

pub fn generate_squid_config<P: AsRef<Path>>(
  out_dir: P,
  clients: &IdentifiedList<Client>,
  domainlists: &IdentifiedList<DomainList>,
) -> Result<()> {
  let out_dir = out_dir.as_ref();
  fs::create_dir_all(out_dir)?;

  for client in clients.items.iter().filter(|c| !c.rules.is_empty()) {
    let client_name = id_string("client", client);
    let mut b = create_writer(out_dir, format!("{}.conf", client_name))?;

    // First, figure out if there are any domains that are temporarily allowed due to a lease rule.
    let now = Utc::now();
    let allowed_domains: Vec<_> = client
      .leases
      .iter()
      .filter(|l| l.rule.kind == RuleKind::AllowHttpAccess && l.end_date_utc.unwrap() > now)
      .flat_map(|l| l.rule.domainlists.iter())
      .collect();

    b.writeln(format!("acl {} src {}", client_name, client.ip))?;
    for domain in client
      .rules
      .iter()
      .filter(|r| r.kind == RuleKind::DenyHttpAccess)
      .flat_map(|r| r.domainlists.iter())
    {
      if !allowed_domains.contains(&domain) {
        b.writeln(format!(
          "http_access deny {} {}",
          client_name,
          id_string("domains", domain)
        ))?;
      }
    }
  }

  // If there are no clients, we must nevertheless write out a dummy client_*.conf file, otherwise
  // squid will barf.
  let dummy = out_dir.join("client_dummy.conf");
  if !dummy.exists() {
    let mut writer = create_writer(out_dir, "client_dummy.conf")?;
    writer.writeln("# This file is intentionally left blank.")?;
  }

  let mut b = create_writer(out_dir, "domains.conf")?;
  if !domainlists.items.is_empty() {
    for domainlist in domainlists.items.iter() {
      let domainlist_name = id_string("domains", domainlist);
      b.writeln(format!(
        "acl {} dstdomain {}",
        domainlist_name,
        domainlist.domains.join(" ")
      ))?;
    }
  } else {
    // We must always write out a domains.conf, otherwise squid will barf. If there are no domains,
    // just write one with a comment.
    b.writeln("# This file will be populated with penguin domains")?;
  }

  Ok(())
}
