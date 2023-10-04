use std::{path::Path, fs::File, io::{LineWriter, Write, BufReader, BufWriter}};
use anyhow::{anyhow, Result};
use serde_json::Value;

pub fn read_json_value(path: &Path) -> anyhow::Result<Value> {
  let file = File::open(path)?;
  let reader = BufReader::new(&file);

  Ok(serde_json::from_reader(reader)?)
}

pub fn write_json_value(path: &Path, value: &Value) -> anyhow::Result<()> {
  let file = File::options().write(true).open(path)?;
  let writer = BufWriter::new(&file);

  serde_json::to_writer_pretty(writer, value)?;

  Ok(())
}

pub fn get_parent_or_die(path: &Path) -> anyhow::Result<&Path> {
  path.parent().ok_or_else(|| anyhow!("Failed to get parent of {:?}", path))
}

/// Creates a file, ensuring that its parent directories exist.
pub fn create_file<P: AsRef<Path>>(path: P) -> Result<File> {
  let parent = get_parent_or_die(path.as_ref())?;
  std::fs::create_dir_all(parent)?;
  Ok(File::create(path)?)
}

pub fn create_writer<P: AsRef<Path>, S: Into<String>>(base_path: P, filename: S) -> Result<NiceLineWriter<File>> {
  let path = base_path.as_ref().join(filename.into());
  Ok(NiceLineWriter::new(create_file(path)?))
}

pub struct NiceLineWriter<T: Write> {
  inner: LineWriter<T>
}

impl<T: Write> NiceLineWriter<T> {
  fn new(write: T) -> Self {
    Self {
      inner: LineWriter::new(write)
    }
  }

  pub fn writeln<S: Into<String>>(&mut self, s: S) -> Result<()> {
    self.inner.write_all(s.into().as_bytes())?;
    self.inner.write_all(b"\n")?;
    Ok(())
  }
}