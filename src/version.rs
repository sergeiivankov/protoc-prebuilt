use crate::error::Error;

// In protobuf repository for release cadidate versions used "v22.0-rc3" tag name,
// for example, but in asset name $VERSION part looks like "22.0-rc-3"
// (with `-` delimiter between `rc` prefix and subversion number)
//
// Exceptional cases:
// - "3.7.0-rc.3" (dot between `rc` part and number)
//   have assets names "protoc-3.7.0-rc-3-*"
// - "3.7.0rc2" (no hyphen between main version and `rc` part)
//   have assets names "protoc-3.7.0-rc-2-*"
// - "3.7.0rc1" (no hyphen between main version and `rc` part)
//   have assets names "protoc-3.7.0-rc1-*" (no hyphen between `rc` part and number)
// - "3.2.0rc2" (no hyphen between main version and `rc` part)
//   have assets names "protoc-3.2.0rc2-*" (no hyphens between main version and `rc` part
//   and `rc` part and number)
fn prepare_asset_version(version: &str) -> String {
  if !version.contains("rc") {
    return String::from(version)
  }

  if version == "3.7.0-rc.3" {
    return String::from("3.7.0-rc-3")
  }
  if version == "3.7.0rc2" {
    return String::from("3.7.0-rc-2")
  }
  if version == "3.7.0rc1" {
    return String::from("3.7.0-rc1")
  }
  if version == "3.2.0rc2" {
    return String::from("3.2.0rc2")
  }

  let parts = version.split_once("rc").unwrap();
  format!("{}rc-{}", parts.0, parts.1)
}

// Compare required protobuf compiler version with version returned
// by calling protoc with "--version" argument
//
// Last protobuf compiler versions return same version as in GitHub tag name
//
// Exceptional cases:
// - before "3.14.0-rc1" version release candidates, alpha and beta versions return same name
//   as main version, for example, "3.13.0-rc3" return "3.13.0", "3.0.0-beta-1" return "3.0.0"
// - "21.*" versions returns "3.21.*"
//
// Next protobuf compiler versions return error version values:
// - "3.0.2" -> "3.0.0"
// - "3.10.0-rc1" -> "30.10.0"
// - "3.12.2" -> "3.12.1"
// - "3.19.0-rc2" -> "3.19.0-rc1"
// - "21.0-rc1" -> "" (return nothing if call with "--version" argument)
// - "21.0-rc2" -> "" (return nothing if call with "--version" argument)
pub(crate) fn compare_versions(required: &str, returned: &str) -> bool {
  // Protobuf errors
  if (required == "3.0.2" && returned == "3.0.0") ||
     (required == "3.10.0-rc1" && returned == "30.10.0") ||
     (required == "3.12.2" && returned == "3.12.1") ||
     (required == "3.19.0-rc2" && returned == "3.19.0-rc1") ||
     (returned.is_empty() && (required == "21.0-rc1" || required == "21.0-rc2"))
  {
    return true
  }

  // Non default `rc` versions names
  if (required == "3.2.0rc2" && returned == "3.2.0") ||
     (
       (required == "3.7.0rc1" || required == "3.7.0rc2" || required == "3.7.0-rc.3") &&
       returned == "3.7.0"
     )
  {
    return true
  }

  // Old `rc` versions
  if required.contains("-rc") &&
     (required.starts_with("3.8.") || required.starts_with("3.9.") ||
      required.starts_with("3.10.") || required.starts_with("3.11.") ||
      required.starts_with("3.12.") || required.starts_with("3.13.")
     )
  {
    return required.split_once("-rc").unwrap().0 == returned
  }

  // 21.* versions
  if required.starts_with("21.") {
    return format!("3.{}", required) == returned
  }

  // Alpha and beta versions
  if (required == "3.0.0-alpha-1" || required == "3.0.0-alpha-2" || required == "3.0.0-alpha-3" ||
      required == "3.0.0-beta-1" || required == "3.0.0-beta-2" ||
      required == "3.0.0-beta-3" || required == "3.0.0-beta-4") &&
     returned == "3.0.0"
  {
    return true
  }

  required == returned
}

// Format protoc pre-built package name by `protoc-$VERSION-$PLATFORM` view,
// depending on protobuf version, target os and architecture
//
// Exceptional cases:
// - "3.0.0-beta-4" have 32-bit linux asset name "protoc-3.0.0-beta-4-linux-x86-32.zip"
//   (with hyphen instead of underscore in architecture part)
// - from "3.10.0-rc1" to "3.12.0-rc1" (not included) linux s390x architecture named "s390x_64"
// - from "3.12.0-rc1" to "3.16.0-rc1" (not included) linux s390x architecture named "s390x"
pub(crate) fn get_protoc_asset_name<'a>(
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
      "s390x" => match version.get(0..4) {
        Some("3.10" | "3.11") => "s390x_64",
        Some("3.12" | "3.13" | "3.14" | "3.15") => "s390x",
        _ => "s390_64"
      },
      "x86" => match version {
        "3.0.0-beta-4" => "x86-32",
        _ => "x86_32"
      },
      "x86_64" => "x86_64",
      _ => return Err(Error::NotProvidedPlatform)
    },
    "macos" => match arch {
      "aarch64" => "aarch_64",
      "x86" => "x86_32",
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

#[cfg(test)]
mod test {
  use crate::error::Error;
  use super::{ compare_versions, get_protoc_asset_name, prepare_asset_version };

  fn check_protoc_assets_name_ok(result: Result<String, Error>, expect: &str) {
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expect);
  }

  fn check_get_protoc_asset_name_err(result: Result<String, Error>) {
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::NotProvidedPlatform { .. }));
  }

  #[test]
  fn compare_version_correct() {
    assert!(compare_versions("2.6.1", "2.6.1"));
    assert!(compare_versions("3.5.0", "3.5.0"));
    assert!(compare_versions("3.14.0-rc2", "3.14.0-rc2"));
    assert!(compare_versions("3.15.0-rc1", "3.15.0-rc1"));
    assert!(compare_versions("3.19.5", "3.19.5"));
    assert!(compare_versions("22.0", "22.0"));
    assert!(compare_versions("26.0-rc1", "26.0-rc1"));
  }

  #[test]
  fn compare_version_incorrect() {
    assert!(!compare_versions("2.4.1", "2.5.0"));
    assert!(!compare_versions("3.14.0-rc2", "3.14.0"));
  }

  #[test]
  fn compare_version_old_rc_alpha_beta() {
    assert!(compare_versions("3.0.0-alpha-1", "3.0.0"));
    assert!(compare_versions("3.0.0-beta-4", "3.0.0"));
    assert!(compare_versions("3.2.0rc2", "3.2.0"));
    assert!(compare_versions("3.7.0rc1", "3.7.0"));
    assert!(compare_versions("3.7.0-rc.3", "3.7.0"));
    assert!(compare_versions("3.8.0-rc1", "3.8.0"));
    assert!(compare_versions("3.8.0-rc1", "3.8.0"));
    assert!(compare_versions("3.11.0-rc2", "3.11.0"));
    assert!(compare_versions("3.13.0-rc3", "3.13.0"));
  }

  #[test]
  fn compare_version_21x() {
    assert!(compare_versions("21.0", "3.21.0"));
    assert!(compare_versions("21.12", "3.21.12"));
  }

  #[test]
  fn compare_version_protoc_errors() {
    assert!(compare_versions("3.0.2", "3.0.0"));
    assert!(compare_versions("3.10.0-rc1", "30.10.0"));
    assert!(compare_versions("3.12.2", "3.12.1"));
    assert!(compare_versions("3.19.0-rc2", "3.19.0-rc1"));
    assert!(compare_versions("21.0-rc1", ""));
    assert!(compare_versions("21.0-rc2", ""));
  }

  #[test]
  fn prepare_assets_version_default() {
    assert_eq!(prepare_asset_version("22.0"), "22.0");
    assert_eq!(prepare_asset_version("22.0-rc3"), "22.0-rc-3");
    assert_eq!(prepare_asset_version("3.0.0-beta-3"), "3.0.0-beta-3");
  }

  #[test]
  fn prepare_assets_version_exceptions() {
    assert_eq!(prepare_asset_version("3.7.0-rc.3"), "3.7.0-rc-3");
    assert_eq!(prepare_asset_version("3.7.0rc2"), "3.7.0-rc-2");
    assert_eq!(prepare_asset_version("3.7.0rc1"), "3.7.0-rc1");
    assert_eq!(prepare_asset_version("3.2.0rc2"), "3.2.0rc2");
  }

  #[test]
  fn get_protoc_assets_name_default() {
    check_protoc_assets_name_ok(
      get_protoc_asset_name("22.0", "linux", "x86"),
      "protoc-22.0-linux-x86_32"
    );
    check_protoc_assets_name_ok(
      get_protoc_asset_name("22.0-rc3", "macos", "aarch64"),
      "protoc-22.0-rc-3-osx-aarch_64"
    );
    check_protoc_assets_name_ok(
      get_protoc_asset_name("21.12", "windows", "x86_64"),
      "protoc-21.12-win64"
    );
    check_protoc_assets_name_ok(
      get_protoc_asset_name("21.0", "linux", "s390x"),
      "protoc-21.0-linux-s390_64"
    );
  }

  #[test]
  fn get_protoc_assets_name_exceptions() {
    check_protoc_assets_name_ok(
      get_protoc_asset_name("3.0.0-beta-4", "linux", "x86"),
      "protoc-3.0.0-beta-4-linux-x86-32"
    );
    check_protoc_assets_name_ok(
      get_protoc_asset_name("3.10.0-rc1", "linux", "s390x"),
      "protoc-3.10.0-rc-1-linux-s390x_64"
    );
    check_protoc_assets_name_ok(
      get_protoc_asset_name("3.11.2", "linux", "s390x"),
      "protoc-3.11.2-linux-s390x_64"
    );
    check_protoc_assets_name_ok(
      get_protoc_asset_name("3.12.0-rc1", "linux", "s390x"),
      "protoc-3.12.0-rc-1-linux-s390x"
    );
    check_protoc_assets_name_ok(
      get_protoc_asset_name("3.15.4", "linux", "s390x"),
      "protoc-3.15.4-linux-s390x"
    );
  }

  #[test]
  fn get_protoc_asset_name_err() {
    check_get_protoc_asset_name_err(get_protoc_asset_name("22.0", "freebsd", "x86_64"));
    check_get_protoc_asset_name_err(get_protoc_asset_name("22.0", "freebsd", "aarch64"));
    check_get_protoc_asset_name_err(get_protoc_asset_name("22.0", "windows", "aarch64"));
  }
}