use std::fs;
use std::path::Path;
use crate::file::create_writer;
use crate::list::Identifiable;
use crate::{model::{Client, DomainList}, list::IdentifiedList};
use anyhow::Result;
use chrono::Utc;


static DENY_RULE: &str = "deny_http_access";
static ALLOW_RULE: &str = "allow_http_access";

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

pub fn generate_squid_config<P: AsRef<Path>>(out_dir: P, clients: &IdentifiedList<Client>, domainlists: &IdentifiedList<DomainList>) -> Result<()> {
  let out_dir = out_dir.as_ref();
  fs::create_dir_all(out_dir)?;

  for client in clients.items.iter().filter(|c| !c.rules.is_empty()) {
    let client_name = id_string("client", client);
    let mut b = create_writer(out_dir, format!("{}.conf", client_name))?;

    // First, figure out if there are any domains that are temporarily allowed due to a lease rule.
    let now = Utc::now().naive_local();
    let allowed_domains : Vec<_> = client.leases.iter()
        .filter(|l| l.rule.kind == ALLOW_RULE && l.end_date > now)
        .flat_map(|l| l.rule.domainlists.iter() )
        .collect();

    b.writeln(format!("acl {} src {}/255.255.255.255", client_name, client.ip))?;
    for domain in client.rules.iter().filter(|r| r.kind == DENY_RULE).flat_map(|r| r.domainlists.iter() ) {
      if !allowed_domains.contains(&domain) {
        b.writeln(format!("http_access deny {} {}", client_name, id_string("domains", domain)))?;
      }
    }
  }

  if !domainlists.items.is_empty() {
    let mut b = create_writer(out_dir, "domains.conf")?;

    for domainlist in domainlists.items.iter() {
      let domainlist_name = id_string("domains", domainlist);
      b.writeln(format!("acl {} dstdomain \"./{}.txt\"", domainlist_name, domainlist_name))?;
      let mut b = create_writer(out_dir, format!("{}.txt", domainlist_name))?;
      b.writeln(domainlist.domains.join("\n"))?;
    }
  }

  Ok(())
}