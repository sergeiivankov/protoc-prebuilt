use std::{ fs::{ remove_file, File }, io::copy, path::{ Path, PathBuf } };
use ureq::Response;
use zip::ZipArchive;
use crate::{ error::Error, helpers::get_github_token, request::request_with_token };

// Check is repository tag exists for passed version
fn check_version_exists<'a>(version: &'a str, token: &Option<String>) -> Result<(), Error<'a>> {
  match request_with_token(
    &format!("https://api.github.com/repos/protocolbuffers/protobuf/releases/tags/v{}", version),
    token
  ) {
    Ok(_) => Ok(()),
    Err(ureq::Error::Status(code, response)) => {
      match code {
        404 => Err(Error::NonExistsVersion(version)),
        _ => {
          let text = response.into_string().map_err(Error::Io)?;
          Err(Error::GitHubApi((code, text)))
        }
      }
    },
    Err(err) => Err(Error::Ureq(Box::new(err)))
  }
}

// Download required version asset
fn download<'a>(
  version: &'a str, token: &Option<String>, protoc_asset_file_name: &str
) -> Result<Response, Error<'a>> {
  match request_with_token(
    &format!(
      "https://github.com/protocolbuffers/protobuf/releases/download/v{}/{}",
      version, protoc_asset_file_name
    ),
    token
  ) {
    Ok(response) => Ok(response),
    Err(ureq::Error::Status(code, response)) => {
      match code {
        404 => Err(Error::NonExistsPlatformVersion(version)),
        _ => {
          let text = response.into_string().map_err(Error::Io)?;
          Err(Error::GitHubApi((code, text)))
        }
      }
    },
    Err(err) => Err(Error::Ureq(Box::new(err)))
  }
}

// Download and unpack requred protobuf compiler version and platform
pub(crate) fn install<'a>(
  version: &'a str, out_dir: &Path, protoc_asset_name: &String, protoc_out_dir: &PathBuf
) -> Result<(), Error<'a>> {
  let token = get_github_token();

  check_version_exists(version, &token)?;

  let protoc_asset_file_name = format!("{}.zip", protoc_asset_name);

  // Try download binaries
  let response = download(version, &token, &protoc_asset_file_name)?;

  let protoc_asset_file_path = out_dir.join(&protoc_asset_file_name);

  // Remove previous asset file
  if protoc_asset_file_path.exists() {
    remove_file(&protoc_asset_file_path).map_err(Error::Io)?;
  }

  // Create asset file
  let mut file = File::options()
    .create(true).read(true).write(true)
    .open(&protoc_asset_file_path)
    .map_err(Error::Io)?;

  // Write content to file
  let mut response_reader = response.into_reader();
  copy(&mut response_reader, &mut file).map_err(Error::Io)?;

  // Extract archive and delete file
  let mut archive = ZipArchive::new(file).map_err(Error::Zip)?;
  archive.extract(protoc_out_dir).map_err(Error::Zip)?;

  remove_file(&protoc_asset_file_path).map_err(Error::Io)?;

  Ok(())
}

#[cfg(test)]
mod test {
  use crate::error::Error;
  use super::{ check_version_exists, download };

  #[test]
  fn check_version_exists_success() {
    assert!(check_version_exists("22.0", &None).is_ok());
    assert!(check_version_exists("3.7.0", &None).is_ok());
  }

  #[test]
  fn check_version_exists_fail() {
    let result = check_version_exists("0.1.0", &None);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NonExistsVersion { .. }));
  }

  #[test]
  fn download_success() {
    let result = download("2.4.1", &None, "protoc-2.4.1-win32.zip");
    assert!(result.is_ok());
  }

  #[test]
  fn download_fail_version() {
    // Version 3.19.4 has not yet been pre-builded for Apple M1
    let result = download("3.19.4", &None, "protoc-3.19.4-osx-aarch_64.zip");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NonExistsPlatformVersion { .. }));
  }
}