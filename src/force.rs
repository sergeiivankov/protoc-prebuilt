use std::{ env::{ VarError, var }, fs::metadata, path::PathBuf };
use crate::error::Error;

// Inner testable logic check force binary path
fn check_force_bin(
  env_var_value: Result<String, VarError>
) -> Result<Option<PathBuf>, Error<'static>> {
  if let Ok(force_protoc_path) = env_var_value {
    // Check is passed path exists
    let attr = match metadata(&force_protoc_path) {
      Ok(attr) => attr,
      Err(_) => return Err(Error::ForcePath(
        format!("nothing exists by PROTOC_PREBUILT_FORCE_PROTOC_PATH path {}", force_protoc_path)
      ))
    };

    // Check is file in passed path
    if attr.is_dir() {
      return Err(Error::ForcePath(
        format!("directory found by PROTOC_PREBUILT_FORCE_PROTOC_PATH path {}", force_protoc_path)
      ))
    }

    return Ok(Some(force_protoc_path.into()));
  }

  Ok(None)
}

// Inner testable logic check force include path
fn check_force_include(
  env_var_value: Result<String, VarError>
) -> Result<Option<PathBuf>, Error<'static>> {
  if let Ok(force_include_path) = env_var_value {
    // Check is passed path exists
    let attr = match metadata(&force_include_path) {
      Ok(attr) => attr,
      Err(_) => return Err(Error::ForcePath(
        format!("nothing exists by PROTOC_PREBUILT_FORCE_INCLUDE_PATH path {}", force_include_path)
      ))
    };

    // Check is directory in passed path
    if attr.is_file() {
      return Err(Error::ForcePath(
        format!("file found by PROTOC_PREBUILT_FORCE_INCLUDE_PATH path {}", force_include_path)
      ))
    }

    return Ok(Some(force_include_path.into()));
  }

  Ok(None)
}

// Check is need use force include path and check is it exists
pub(crate) fn get_force_bin() -> Result<Option<PathBuf>, Error<'static>> {
  check_force_bin(var("PROTOC_PREBUILT_FORCE_PROTOC_PATH"))
}

// Check is need use force include path and check is it exists
pub(crate) fn get_force_include() -> Result<Option<PathBuf>, Error<'static>> {
  check_force_include(var("PROTOC_PREBUILT_FORCE_INCLUDE_PATH"))
}

#[cfg(test)]
mod test {
  use std::{
    env::{ VarError, temp_dir },
    fs::{ File, create_dir_all, remove_dir_all, remove_file },
    path::PathBuf
  };
  use crate::error::Error;
  use super::{ check_force_bin, check_force_include };

  // Store file path in struct to clear test artifacts in drop implementation
  struct FilePath(PathBuf);

  impl Drop for FilePath {
    fn drop(&mut self) {
      remove_file(&self.0).unwrap();
    }
  }

  // Store directory path in struct to clear test artifacts in drop implementation
  struct DirectoryPath(PathBuf);

  impl Drop for DirectoryPath {
    fn drop(&mut self) {
      remove_dir_all(&self.0).unwrap();
    }
  }

  #[test]
  fn bin_return_ok_none_for_absence_env_var() {
    let result = check_force_bin(Err(VarError::NotPresent));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
  }

  #[test]
  fn include_return_ok_none_for_absence_env_var() {
    let result = check_force_include(Err(VarError::NotPresent));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
  }

  #[test]
  fn bin_return_err_for_non_exists_path() {
    let force_path = temp_dir()
      .join("protoc-prebuilt-test")
      .join("bin_return_err_for_non_exists_path");

    let result = check_force_bin(Ok(String::from(force_path.to_str().unwrap())));

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      Error::ForcePath(message) if message.contains("nothing exists")
    ));
  }

  #[test]
  fn include_return_err_for_non_exists_path() {
    let force_path = temp_dir()
      .join("protoc-prebuilt-test")
      .join("include_return_err_for_non_exists_path");

    let result = check_force_include(Ok(String::from(force_path.to_str().unwrap())));

    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      Error::ForcePath(message) if message.contains("nothing exists")
    ));
  }

  #[test]
  fn bin_return_err_for_directory_by_path() {
    let force_bin_path = temp_dir()
      .join("protoc-prebuilt-test")
      .join("bin_return_err_for_directory_by_path");
    remove_dir_all(&force_bin_path).ok();
    create_dir_all(&force_bin_path).unwrap();
    let force_bin_path = DirectoryPath(force_bin_path);

    let result = check_force_bin(Ok(String::from(force_bin_path.0.to_str().unwrap())));
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      Error::ForcePath(message) if message.contains("directory found")
    ));
  }

  #[test]
  fn include_return_err_for_file_by_path() {
    let force_include_path_parent = temp_dir().join("protoc-prebuilt-test");
    let force_include_path = force_include_path_parent.clone()
      .join("include_return_err_for_file_by_path");

    remove_file(&force_include_path).ok();
    create_dir_all(&force_include_path_parent).unwrap();
    File::create(&force_include_path).unwrap();
    let force_include_path = FilePath(force_include_path);

    let result = check_force_include(Ok(String::from(force_include_path.0.to_str().unwrap())));
    assert!(result.is_err());
    assert!(matches!(
      result.unwrap_err(),
      Error::ForcePath(message) if message.contains("file found")
    ));
  }

  #[test]
  fn bin_return_ok_path() {
    let force_bin_path_parent = temp_dir().join("protoc-prebuilt-test");
    let force_bin_path = force_bin_path_parent.clone().join("bin_return_ok_path");

    remove_file(&force_bin_path).ok();
    create_dir_all(&force_bin_path_parent).unwrap();
    File::create(&force_bin_path).unwrap();
    let force_bin_path = FilePath(force_bin_path);

    let result = check_force_bin(Ok(String::from(force_bin_path.0.to_str().unwrap())));
    assert!(result.is_ok());

    let option = result.unwrap();
    assert!(option.is_some());

    assert_eq!(option.unwrap().to_str().unwrap(), force_bin_path.0.to_str().unwrap());
  }

  #[test]
  fn include_return_ok_path() {
    let force_include_path = temp_dir().join("protoc-prebuilt-test").join("include_return_ok_path");
    remove_dir_all(&force_include_path).ok();
    create_dir_all(&force_include_path).unwrap();
    let force_include_path = DirectoryPath(force_include_path);

    let result = check_force_include(Ok(String::from(force_include_path.0.to_str().unwrap())));
    assert!(result.is_ok());

    let option = result.unwrap();
    assert!(option.is_some());

    assert_eq!(option.unwrap().to_str().unwrap(), force_include_path.0.to_str().unwrap());
  }
}