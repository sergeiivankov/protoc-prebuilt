#![doc = include_str!("../readme.md")]

use ureq::{ Response, request };
use std::{
  env::{ consts::{ ARCH, OS }, VarError, var },
  fmt::{ Display, Formatter, Result as FmtResult },
  fs::{ File, remove_file },
  io::copy,
  path::{ Path, PathBuf }
};
use zip::{ result::ZipError, ZipArchive };

// GitHub API require User-Agent header
static CRATE_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Error returned if installation or initialization fail
#[derive(Debug)]
pub enum Error<'a> {
  /// Pre-built binary not provided for current platform
  NotProvidedPlatform,
  /// Passed version not exists
  NonExistsVersion(&'a str),
  /// Pre-built binary not provided for current platform and passed version
  NonExistsPlatformVersion(&'a str),
  /// GitHub API response error
  GitHubApi((u16, String)),
  /// Read environment variable fail
  VarError(VarError),
  /// I/O operation error
  Io(std::io::Error),
  /// Ureq crate error
  Ureq(Box<ureq::Error>),
  /// Zip crate error
  Zip(ZipError)
}

impl<'a> Display for Error<'a> {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    match self {
      Error::NotProvidedPlatform => {
        write!(f, "Pre-built binaries for `{}-{}` platform don't provided", OS, ARCH)
      },
      Error::NonExistsVersion(version) => {
        write!(f, "Pre-built binaries version `{}` not exists", version)
      },
      Error::NonExistsPlatformVersion(version) => {
        write!(
          f,
          "Pre-built binaries version `{}` for `{}-{}` platform don't provided",
          version, OS, ARCH
        )
      },
      Error::GitHubApi((status, response)) => {
        write!(f, "GitHub API response error: {} {}", status, response)
      },
      Error::VarError(err) => write!(f, "{}", err),
      Error::Io(err) => write!(f, "{}", err),
      Error::Ureq(err) => write!(f, "{}", err),
      Error::Zip(err) => write!(f, "{}", err)
    }
  }
}

// In protobuf repository for release cadidate versions used "v22.0-rc3" tag name,
// for example, but in asset name $VERSION part looks like "22.0-rc-3"
// (with `-` delimiter between `rc` prefix and subversion number)
fn prepare_asset_version(version: &str) -> String {
  if !version.contains("rc") {
    return String::from(version)
  }

  let parts = version.split_once("rc").unwrap();
  format!("{}rc-{}", parts.0, parts.1)
}

// Format protoc pre-built package name by `protoc-$VERSION-$PLATFORM` view,
// depending on compiler version, target os and architecture
fn get_protoc_asset_name<'a>(
  version: &str, os: &str, arch: &str
) -> Result<String, Error<'a>> {
  // Rename os by protobuf compiler assets version
  let asset_os = match os {
    "linux" => "linux",
    "macos" => "osx",
    "windows" => "win",
    _ => return Err(Error::NotProvidedPlatform)
  };

  // Rename arch by protobuf compiler assets version and target os
  let asset_arch = match os {
    "linux" => match arch {
      "aarch64" => "aarch_64",
      "powerpc64" => "ppcle_64",
      "s390x" => "s390_64",
      "x86" => "x86_32",
      "x86_64" => "x86_64",
      _ => return Err(Error::NotProvidedPlatform)
    },
    "macos" => match arch {
      "aarch64" => "aarch_64",
      "x86_64" => "x86_64",
      _ => return Err(Error::NotProvidedPlatform)
    },
    "windows" => match arch {
      "x86" => "32",
      "x86_64" => "64",
      _ => return Err(Error::NotProvidedPlatform)
    },
    _ => unreachable!()
  };

  // In protobuf compiler assets windows builds don't have a separator between os and arch
  let os_arch_delimiter = match os {
    "windows" => "",
    _ => "-"
  };

  Ok(format!(
    "protoc-{}-{}{}{}", prepare_asset_version(version), asset_os, os_arch_delimiter, asset_arch
  ))
}

#[allow(clippy::result_large_err)]
fn get(url: &str) -> Result<Response, ureq::Error> {
  let mut req = request("GET", url).set("User-Agent", CRATE_USER_AGENT);

  if let Ok(raw_github_token) = var("GITHUB_TOKEN") {
    let github_token = raw_github_token.trim();
    if !github_token.is_empty() {
      req = req.set("Authorization", &format!("Bearer {}", github_token))
    }
  }

  req.call()
}

fn install<'a>(
  version: &'a str, out_dir: &Path, protoc_asset_name: &String, protoc_out_dir: &PathBuf
) -> Result<(), Error<'a>> {
  match get(&format!(
    "https://api.github.com/repos/protocolbuffers/protobuf/releases/tags/v{}", version
  )) {
    Ok(_) => {},
    Err(ureq::Error::Status(code, response)) => {
      match code {
        404 => return Err(Error::NonExistsVersion(version)),
        _ => {
          let text = response.into_string().map_err(Error::Io)?;
          return Err(Error::GitHubApi((code, text)))
        }
      }
    },
    Err(err) => return Err(Error::Ureq(Box::new(err)))
  }

  // Try download binaries
  let protoc_asset_file_name = format!("{}.zip", protoc_asset_name);

  let response = match get(&format!(
    "https://github.com/protocolbuffers/protobuf/releases/download/v{}/{}",
    version, protoc_asset_file_name
  )) {
    Ok(response) => response,
    Err(ureq::Error::Status(code, response)) => {
      match code {
        404 => return Err(Error::NonExistsPlatformVersion(version)),
        _ => {
          let text = response.into_string().map_err(Error::Io)?;
          return Err(Error::GitHubApi((code, text)))
        }
      }
    },
    Err(err) => return Err(Error::Ureq(Box::new(err)))
  };

  // Write content to file
  let protoc_asset_file_path = out_dir.join(&protoc_asset_file_name);
  if protoc_asset_file_path.exists() {
    remove_file(&protoc_asset_file_path).map_err(Error::Io)?;
  }

  let mut file = File::options()
    .create(true).read(true).write(true)
    .open(&protoc_asset_file_path)
    .map_err(Error::Io)?;

  let mut response_reader = response.into_reader();
  copy(&mut response_reader, &mut file).map_err(Error::Io)?;

  // Extract archive and delete file
  let mut archive = ZipArchive::new(file).map_err(Error::Zip)?;
  archive.extract(protoc_out_dir).map_err(Error::Zip)?;

  remove_file(&protoc_asset_file_path).map_err(Error::Io)?;

  Ok(())
}

/// Install pre-built protobuf compiler binary if it hasn't been done before
/// and return paths to it content
///
/// Version parameter should be a tag name from protobuf repository without `v` prefix,
/// for example, "21.12" or "22.0-rc3"
/// (see [protobuf repository tags](https://github.com/protocolbuffers/protobuf/tags)).
///
/// Return a tuple contains paths to `protoc` binary and `include` directory.
pub fn init(version: &str) -> Result<(PathBuf, PathBuf), Error> {
  let out_dir = PathBuf::from(var("OUT_DIR").map_err(Error::VarError)?);

  let protoc_asset_name = get_protoc_asset_name(version, OS, ARCH)?;
  let protoc_out_dir = out_dir.join(&protoc_asset_name);

  // Install if installation directory doesn't exist
  if !protoc_out_dir.exists() {
    install(version, &out_dir, &protoc_asset_name, &protoc_out_dir)?;
  }

  let mut protoc_bin = protoc_out_dir.clone();
  protoc_bin.push("bin");
  protoc_bin.push(format!("protoc{}", match OS { "windows" => ".exe", _ => "" }));

  let mut protoc_include = protoc_out_dir;
  protoc_include.push("include");

  Ok((protoc_bin, protoc_include))
}

#[cfg(test)]
mod test {
  use std::env::{ remove_var, set_var, temp_dir };
  use crate::{
    CRATE_USER_AGENT, Error, prepare_asset_version, get_protoc_asset_name, get, install
  };

  #[test]
  fn prepare_assets_versions() {
    assert_eq!(prepare_asset_version("22.0"), "22.0");
    assert_eq!(prepare_asset_version("22.0-rc3"), "22.0-rc-3");
  }

  #[test]
  fn get_protoc_assets_names() {
    fn check_not_provided_platform_err(result: Result<String, Error>) {
      assert!(result.is_err());
      assert!(matches!(result.unwrap_err(), Error::NotProvidedPlatform { .. }));
    }

    fn check_protoc_asset_name_result(result: Result<String, Error>, expect: &str) {
      assert!(result.is_ok());
      assert_eq!(result.unwrap(), expect);
    }

    check_not_provided_platform_err(get_protoc_asset_name("22.0", "freebsd", "x86_64"));
    check_not_provided_platform_err(get_protoc_asset_name("22.0", "freebsd", "aarch64"));
    check_not_provided_platform_err(get_protoc_asset_name("22.0", "windows", "aarch64"));

    check_protoc_asset_name_result(
      get_protoc_asset_name("22.0", "linux", "x86"),
      "protoc-22.0-linux-x86_32"
    );
    check_protoc_asset_name_result(
      get_protoc_asset_name("22.0-rc3", "macos", "aarch64"),
      "protoc-22.0-rc-3-osx-aarch_64"
    );
    check_protoc_asset_name_result(
      get_protoc_asset_name("21.12", "windows", "x86_64"),
      "protoc-21.12-win64"
    );
  }

  #[test]
  fn get_fail() {
    let result = get("https://bf2d04e1aea451f5b530e4c36666c0f0.com");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ureq::Error::Transport { .. }));
  }

  #[test]
  fn get_user_agent() {
    let result = get("https://httpbin.org/get");
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status(), 200);

    let text_result = response.into_string();
    assert!(text_result.is_ok());

    let text = text_result.unwrap();
    assert!(text.contains(CRATE_USER_AGENT))
  }

  #[test]
  // Need to be ignored, because it change environment variable
  // and it has an influence to other tests,
  // to test it run `cargo test -- --ignored`
  #[ignore]
  fn get_fail_github_token() {
    set_var("GITHUB_TOKEN", "ghp_000000000000000000000000000000000000");
    let result = get("https://api.github.com/repos/protocolbuffers/protobuf/releases");
    remove_var("GITHUB_TOKEN");

    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, ureq::Error::Status { .. }));

    let response = error.into_response().unwrap();
    assert_eq!(response.status(), 401);
  }

  #[test]
  fn install_fail_non_exists_version() {
    let version = "0.1";
    let out_dir = temp_dir().join("protoc-prebuilt-unit");
    let protos_asset_name = get_protoc_asset_name(version, "windows", "x86_64").unwrap();
    let protoc_out_dir = out_dir.join(&protos_asset_name);

    let result = install(version, &out_dir, &protos_asset_name, &protoc_out_dir);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NonExistsVersion { .. }));
  }

  #[test]
  fn install_fail_non_exists_platform_version() {
    // Version 3.19.4 has not yet been pre-builded for Apple M1
    let version = "3.19.4";
    let out_dir = temp_dir().join("protoc-prebuilt-unit");
    let protos_asset_name = get_protoc_asset_name(version, "macos", "aarch64").unwrap();
    let protoc_out_dir = out_dir.join(&protos_asset_name);

    let result = install(version, &out_dir, &protos_asset_name, &protoc_out_dir);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NonExistsPlatformVersion { .. }));
  }
}