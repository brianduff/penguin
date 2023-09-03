use std::{path::Path, fs::File, io::{LineWriter, Write}};
use anyhow::Result;

/// Creates a file, ensuring that its parent directories exist.
pub fn create_file<P: AsRef<Path>>(path: P) -> Result<File> {
  if path.as_ref().parent().is_some_and(|p| !p.exists()) {
    std::fs::create_dir_all(path.as_ref().parent().unwrap())?;
  }
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