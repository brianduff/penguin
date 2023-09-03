use std::mem;

use crate::model::{Client, DomainList};

pub trait Identifiable {
  fn id(&self) -> Option<u32>;
  fn set_id(&mut self, id: u32);
}

pub struct IdentifiedList<T: Identifiable + Clone> {
  pub items: Vec<T>
}

impl<T: Identifiable + Clone> IdentifiedList<T> {
  pub fn new(items: Vec<T>) -> Self {
    Self {
      items
    }
  }

  pub fn add(&mut self, item: T) -> &T {
    let mut owned = item.to_owned();

    let max_id = self.items.iter().map(|c| c.id()).fold(0, |a, b| a.max(b.unwrap()));
    owned.set_id(max_id + 1);
    self.items.push(owned);

    self.items.last().unwrap()
  }

  pub fn update(&mut self, id: u32, mut updated: T) -> Option<&T> {
    updated.set_id(id);
    match self.items.iter().position(|c| c.id() == Some(id)) {
      Some(pos) => {
        let _ = mem::replace(&mut self.items[pos], updated);
        self.items.get(pos)
      },
      None => None
    }
  }

  pub fn delete(&mut self, id: u32) -> Option<T> {
    if let Some(pos) = self.items.iter().position(|c| c.id() == Some(id)) {
      Some(self.items.remove(pos))
    } else {
      None
    }
  }

}

impl Identifiable for Client {
    fn id(&self) -> Option<u32> {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = Some(id)
    }
}

impl Identifiable for DomainList {
  fn id(&self) -> Option<u32> {
    self.id
  }

  fn set_id(&mut self, id: u32) {
      self.id = Some(id)
  }
}