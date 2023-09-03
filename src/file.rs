use std::{path::Path, fs::File};
use anyhow::Result;

/// Creates a file, ensuring that its parent directories exist.
pub fn create_file<P: AsRef<Path>>(path: P) -> Result<File> {
  if path.as_ref().parent().is_some_and(|p| !p.exists()) {
    std::fs::create_dir_all(path.as_ref().parent().unwrap())?;
  }
  Ok(File::create(path)?)
}