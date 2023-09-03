use std::path::{PathBuf, Path};
use std::io::{Write, BufWriter};
use crate::{model::{Client, DomainList}, list::IdentifiedList, file::create_file};
use anyhow::Result;

static SQUID_DIR: &str = "squid";
static DENY_RULE: &str = "deny_http_access";

pub fn generate_squid_config(clients: &IdentifiedList<Client>, domainlists: &IdentifiedList<DomainList>) -> Result<()> {
  let base_path = Path::new(SQUID_DIR);
  for client in clients.items.iter() {
    if !client.rules.is_empty() {
      let client_name = format!("client_{}", client.id.unwrap());
      let path = base_path.join(format!("{}.conf", client_name));
      let file = create_file(path)?;
      let mut b = BufWriter::new(file);
      b.write_all(format!("acl {} src {}/255.255.255.255\n", client_name, client.ip).as_bytes())?;

      for rule in client.rules.iter() {
        if DENY_RULE == rule.kind {
          for domain in rule.domainlists.iter() {
            b.write_all(format!("http_access deny {} domains_{}\n", client_name, domain).as_bytes())?;
          }
        }
      }
    }
  }

  if !domainlists.items.is_empty() {
    let path = base_path.join("domains.conf");
    let file = create_file(path)?;
    let mut b = BufWriter::new(file);

    for domainlist in domainlists.items.iter() {
      let domainlist_name = format!("domains_{}", domainlist.id.unwrap());
      b.write_all(format!("acl {} dstdomain \"./{}.txt\"\n", domainlist_name, domainlist_name).as_bytes())?;
      let domain_file = create_file(base_path.join(format!("{}.txt", domainlist_name)))?;
      let mut b = BufWriter::new(domain_file);
      b.write_all(domainlist.domains.join("\n").as_bytes())?;
    }
  }

  Ok(())
}