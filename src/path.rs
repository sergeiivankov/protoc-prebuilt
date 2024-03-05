use std::{ env::consts::OS, path::{ Path, PathBuf } };

// GitHub protobuf asset structure information:
// - binary file located in `bin` directory;
// - includes located in `include` directory;
// - before "3.0.0-beta-4" binary file located in asset root;
// - from "3.0.0-alpha-3" to "3.0.0-beta-3" (included) `include` directory content
//   located in asset root;
// - before "3.0.0-alpha-3" `include` directory content not provided.

// Check is binary file located in root by version
fn is_binary_in_root(version: &str) -> bool {
  version == "2.4.1" || version == "2.5.0" || version == "2.6.0" || version == "2.6.1" ||
  version == "3.0.0-alpha-1" || version == "3.0.0-alpha-2" || version == "3.0.0-alpha-3" ||
  version == "3.0.0-beta-1" || version == "3.0.0-beta-2" ||
  version == "3.0.0-beta-3" || version == "3.0.0-beta-4"
}

// Generate binary path path by protoc version and out directory path,
// will be called if path to binary not set force
//
// For special variants of older versions return path to binary without `bin` subdirectory
pub(crate) fn get_bin_path(version: &str, protoc_out_dir: &Path) -> PathBuf {
  let mut protoc_bin: PathBuf = protoc_out_dir.to_path_buf();

  // For old versions no need add `bin` part
  if !is_binary_in_root(version) {
    protoc_bin.push("bin");
  }

  // Add binary file name
  protoc_bin.push(format!("protoc{}", match OS { "windows" => ".exe", _ => "" }));

  protoc_bin
}

// Generate `include` directory path by protoc version and binary path,
// `protoc-prebuilt` means that `include` directory located near binary,
// if it located in another place, use `PROTOC_PREBUILT_FORCE_INCLUDE_PATH` environment variable
//
// For special variants of older versions return path to directory where binary file located
pub(crate) fn get_include_path(version: &str, protoc_bin: &Path) -> PathBuf {
  let mut protoc_include: PathBuf = protoc_bin.to_path_buf();

  // Remove binary name
  protoc_include.pop();

  // For old versions no need remove `bin` and add `includes` parts
  if !is_binary_in_root(version) {
    protoc_include.pop();
    protoc_include.push("include");
  }

  protoc_include
}

#[cfg(test)]
mod test {
  use std::{ env::consts::OS, path::Path };
  use super::{ get_bin_path, get_include_path };

  #[test]
  fn with_bin_subdirectory() {
    assert_eq!(
      get_bin_path("22.0", Path::new("/opt/protoc/22.0")),
      match OS {
        "windows" => Path::new("/opt/protoc/22.0/bin/protoc.exe"),
        _ => Path::new("/opt/protoc/22.0/bin/protoc")
      }
    );
  }

  #[test]
  fn without_bin_subdirectory() {
    assert_eq!(
      get_bin_path("2.4.1", Path::new("/opt/protoc/2.4.1")),
      match OS {
        "windows" => Path::new("/opt/protoc/2.4.1/protoc.exe"),
        _ => Path::new("/opt/protoc/2.4.1/protoc")
      }
    );
  }

  #[test]
  fn with_include_subdirectory() {
    assert_eq!(
      get_include_path("22.0", Path::new("/opt/protoc/22.0/bin/protoc")),
      Path::new("/opt/protoc/22.0/include")
    );
  }

  #[test]
  fn without_include_subdirectory() {
    assert_eq!(
      get_include_path("2.4.1", Path::new("/opt/protoc/2.4.1/protoc")),
      Path::new("/opt/protoc/2.4.1")
    );
  }
}