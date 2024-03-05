use std::{ env::var, ffi::OsStr };

// Convert &str to bool, "", "0", "no", "off", "false" values reduced to false
fn str_to_bool(value: &str) -> bool {
  !matches!(value, "" | "0" | "no" | "off" | "false")
}

// Fetches GitHub authorization token from environment variable
pub(crate) fn get_github_token() -> Option<String> {
  // Return None if GitHub authorization token usage disable
  if var_bool("PROTOC_PREBUILT_NOT_ADD_GITHUB_TOKEN") {
    return None
  }

  // Get contains GitHub authorization token environment variable name
  let github_token_key = var("PROTOC_PREBUILT_GITHUB_TOKEN_ENV_NAME")
    .unwrap_or("GITHUB_TOKEN".to_string());

  // Fetch, convert to string and discard empty
  var(github_token_key)
    .ok()
    .map(|value| value.trim().to_string())
    .filter(|value| !value.is_empty())
}

// Fetches the environment variable key from the current process and convert result to boolean,
// for non exists variable or with value reduceds to false (see `str_to_bool` above) return false
pub(crate) fn var_bool<K: AsRef<OsStr>>(key: K) -> bool {
  match var(key) {
    Ok(value) => str_to_bool(value.as_str()),
    Err(_) => false
  }
}

#[cfg(test)]
mod test {
  use super::str_to_bool;

  #[test]
  fn true_values() {
    assert!(str_to_bool("1"));
    assert!(str_to_bool("yes"));
    assert!(str_to_bool("on"));
    assert!(str_to_bool("true"));
    assert!(str_to_bool("RaNdOmStRiNg"));
  }

  #[test]
  fn false_values() {
    assert!(!str_to_bool(""));
    assert!(!str_to_bool("0"));
    assert!(!str_to_bool("no"));
    assert!(!str_to_bool("off"));
    assert!(!str_to_bool("false"));
  }
}