use protoc_prebuilt::{ Error, init };
use std::{
  env::{ set_var, temp_dir },
  fs::{ create_dir_all, metadata, remove_dir_all, remove_file },
  path::PathBuf
};

// Store directory path in struct to clear test artifacts in drop implementation
struct DirectoryPath(PathBuf);

impl Drop for DirectoryPath {
  fn drop(&mut self) {
    remove_dir_all(&self.0).unwrap();
  }
}

#[test]
fn test_integration() {
  let version = "22.0";

  // Check environment variable "OUT_DIR" is missing
  let result = init(version);
  assert!(result.is_err());
  assert!(matches!(result.unwrap_err(), Error::VarError { .. }));

  // Create environment variable "OUT_DIR" pointed to temp directory, having previously cleared
  let out_dir = DirectoryPath(temp_dir().join("protoc-prebuilt-integration"));
  remove_dir_all(&out_dir.0).ok();
  create_dir_all(&out_dir.0).unwrap();
  set_var("OUT_DIR", out_dir.0.to_str().unwrap());

  // Init crate for first time
  let result = init(version);
  assert!(result.is_ok());
  let (protoc_bin, protoc_include) = result.unwrap();

  // Check installation paths exists
  assert!(metadata(&protoc_bin).is_ok());
  assert!(metadata(protoc_include).is_ok());

  // Delete protoc binary to check what in next initialization not run installation
  // To check is installation need out directory with asset subdirectory exists checking,
  // not it content
  remove_file(&protoc_bin).unwrap();

  // Init crate for second time
  let result = init(version);
  assert!(result.is_ok());
  let (protoc_bin, _) = result.unwrap();

  // Check protoc binary is not exists
  assert!(metadata(protoc_bin).is_err());
}