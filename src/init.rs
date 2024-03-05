use std::{
  env::{ consts::{ ARCH, OS }, var },
  fs::metadata,
  io::{ Error as IoError, ErrorKind },
  path::PathBuf,
  process::Command,
  str::from_utf8
};
use crate::{
  error::Error,
  helpers::var_bool,
  force::{ get_force_bin, get_force_include },
  install::install,
  path::{ get_bin_path, get_include_path },
  version::{ compare_versions, get_protoc_asset_name }
};

/// Install pre-built protobuf compiler binary if it hasn't been done before
/// and return paths to it content
///
/// Version parameter should be a tag name from protobuf repository without `v` prefix,
/// for example, "21.12" or "22.0-rc3"
/// (see [protobuf repository tags](https://github.com/protocolbuffers/protobuf/tags)).
///
/// Return a tuple contains paths to `protoc` binary and `include` directory.
pub fn init(version: &str) -> Result<(PathBuf, PathBuf), Error> {
  let protoc_bin: PathBuf = get_force_bin()?.map_or_else(|| -> Result<PathBuf, Error> {
    let out_dir = PathBuf::from(var("OUT_DIR").map_err(Error::VarError)?);

    let protoc_asset_name = get_protoc_asset_name(version, OS, ARCH)?;
    let protoc_out_dir = out_dir.join(&protoc_asset_name);

    // Install if installation directory doesn't exist
    if !protoc_out_dir.exists() {
      install(version, &out_dir, &protoc_asset_name, &protoc_out_dir)?;
    }

    Ok(get_bin_path(version, &protoc_out_dir))
  }, Ok)?;

  // Check binary file exists
  metadata(&protoc_bin).map_err(Error::Io)?;

  // Test run binary file
  let output = Command::new(&protoc_bin).args(["--version"]).output().map_err(Error::Io)?;
  if !output.status.success() {
    return Err(Error::Io(IoError::new(ErrorKind::Other, "test run protoc fail")))
  }

  if !var_bool("PROTOC_PREBUILT_NOT_CHECK_VERSION") {
    let stdout = match from_utf8(&output.stdout) {
      Ok(stdout) => stdout,
      Err(_) => return Err(
        Error::Io(IoError::new(ErrorKind::Other, "parse test run protoc output fail"))
      )
    };

    let returned = stdout.trim().replace("libprotoc ", "");

    if !compare_versions(version, &returned) {
      return Err(Error::VersionCheck((version, returned)))
    }
  }

  let protoc_include: PathBuf = get_force_include()?
    .map_or_else(|| Ok(get_include_path(version, &protoc_bin)), Ok)?;

  Ok((protoc_bin, protoc_include))
}