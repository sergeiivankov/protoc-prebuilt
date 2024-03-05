use std::{
  env::{ consts::{ ARCH, OS }, VarError },
  fmt::{ Display, Formatter, Result as FmtResult }
};
use zip::result::ZipError;

/// Error returned if installation or initialization fail
#[derive(Debug)]
pub enum Error<'a> {
  /// Pre-built binary not provided for current platform
  NotProvidedPlatform,
  /// Passed version not exists
  NonExistsVersion(&'a str),
  /// Pre-built binary not provided for current platform and passed version
  NonExistsPlatformVersion(&'a str),
  /// Pre-built binary version check fail, contain tuple with required version
  /// and version returned by binary calling with "--version" argument
  VersionCheck((&'a str, &'a str)),
  /// GitHub API response error
  GitHubApi((u16, String)),
  // Force defined paths error
  ForcePath(String),
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
      Error::VersionCheck((required, returned)) => {
        write!(
          f,
          "Pre-built binaries version check error: require `{}`, returned `{}`",
          required, returned
        )
      },
      Error::GitHubApi((status, response)) => {
        write!(f, "GitHub API response error: {} {}", status, response)
      },
      Error::ForcePath(message) => {
        write!(f, "Force defined paths error: {}", message)
      },
      Error::VarError(err) => write!(f, "{}", err),
      Error::Io(err) => write!(f, "{}", err),
      Error::Ureq(err) => write!(f, "{}", err),
      Error::Zip(err) => write!(f, "{}", err)
    }
  }
}