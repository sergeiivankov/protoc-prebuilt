use std::env::var;
use ureq::{ AgentBuilder, Proxy, Response };
use crate::helpers::var_bool;

// GitHub API require User-Agent header
static CRATE_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

// Check proxy and prepare it for usage in `ureq`
fn check_proxy<'a>(proxy: &'a str, no_proxy_option: Option<String>, url: &str) -> Option<&'a str> {
  if let Some(no_proxy) = no_proxy_option {
    // Return None if proxy disable for all hosts
    if no_proxy == "*" {
      return None
    }

    let is_main = url.starts_with("https://github.com/");
    let is_api = url.starts_with("https://api.github.com/");

    let not_use = no_proxy
      .split(',')
      .map(|host| host.trim())
      .filter(|host| !host.is_empty())
      .any(|host| {
        // Disable for github.com and all subdomains
        if host == ".github.com" {
          return true
        }
        // Disable for github.com
        if host == "github.com" && is_main {
          return true
        }
        // Disable for api.github.com
        if (host == "api.github.com" || host == ".api.github.com") && is_api {
          return true
        }

        false
      });

    if not_use {
      return None
    }
  }

  // Remove protocol (`ureq` doesn't digest "https://" prefix)
  let prepared_proxy = if let Some(stripped) = proxy.strip_prefix("https://") {
    stripped
  } else {
    proxy
  };

  Some(prepared_proxy)
}

// Send request to passed URL with passed token in `Authorization` header
// and generated above `User-Agent`
#[allow(clippy::result_large_err)]
pub(crate) fn request_with_token(
  url: &str, token: &Option<String>
) -> Result<Response, ureq::Error> {
  let mut agent_builder = AgentBuilder::new();

  if !var_bool("PROTOC_PREBUILT_NOT_USE_PROXY") {
    // Try get proxy from different environment variables
    let proxy_result = var("http_proxy")
      .or_else(|_| var("HTTP_PROXY"))
      .or_else(|_| var("https_proxy"))
      .or_else(|_| var("HTTPS_PROXY"));

    if let Ok(proxy) = proxy_result {
      let no_proxy_option = var("no_proxy").or_else(|_| var("NO_PROXY")).ok();

      if let Some(prepared_proxy) = check_proxy(&proxy, no_proxy_option, url) {
        agent_builder = agent_builder.proxy(Proxy::new(prepared_proxy)?);
      }
    }
  }

  let agent = agent_builder.build();
  let mut req = agent.get(url).set("User-Agent", CRATE_USER_AGENT);

  if let Some(value) = token {
    req = req.set("Authorization", &format!("Bearer {}", value))
  }

  req.call()
}

#[cfg(test)]
mod test {
  use super::{ CRATE_USER_AGENT, check_proxy, request_with_token };

  #[test]
  fn check_proxy_success() {
    let option = check_proxy("http://localhost", None, "https://github.com/");
    assert!(option.is_some());
    assert_eq!(option.unwrap(), "http://localhost");
  }

  #[test]
  fn prepare_proxy() {
    let option = check_proxy("https://localhost", None, "https://github.com/");
    assert!(option.is_some());
    assert_eq!(option.unwrap(), "localhost");
  }

  #[test]
  fn no_proxy_asterisk() {
    let option = check_proxy("http://localhost", Some(String::from("*")), "https://github.com/");
    assert!(option.is_none());
  }

  #[test]
  fn no_proxy_hosts() {
    let option = check_proxy(
      "http://localhost", Some(String::from("github.com")), "https://github.com/"
    );
    assert!(option.is_none());

    let option = check_proxy(
      "http://localhost", Some(String::from(".github.com")), "https://github.com/"
    );
    assert!(option.is_none());

    let option = check_proxy(
      "http://localhost", Some(String::from(".github.com")), "https://api.github.com/"
    );
    assert!(option.is_none());

    let option = check_proxy(
      "http://localhost", Some(String::from("api.github.com")), "https://api.github.com/"
    );
    assert!(option.is_none());

    let option = check_proxy(
      "http://localhost", Some(String::from(".api.github.com")), "https://api.github.com/"
    );
    assert!(option.is_none());

    let option = check_proxy(
      "http://localhost", Some(String::from("other.org , github.com")), "https://github.com/"
    );
    assert!(option.is_none());
  }

  #[test]
  fn request_fail_to_non_exists_domain() {
    let result = request_with_token("https://bf2d04e1aea451f5b530e4c36666c0f0.com", &None);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ureq::Error::Transport { .. }));
  }

  #[test]
  fn check_user_agent() {
    let result = request_with_token("https://httpbin.org/get", &None);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.status(), 200);

    let text_result = response.into_string();
    assert!(text_result.is_ok());

    let text = text_result.unwrap();
    assert!(text.contains(CRATE_USER_AGENT))
  }

  #[test]
  fn request_fail_authorization() {
    let result = request_with_token(
      "https://api.github.com/repos/protocolbuffers/protobuf/releases",
      &Some("ghp_000000000000000000000000000000000000".to_string())
    );

    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(matches!(error, ureq::Error::Status { .. }));

    let response = error.into_response().unwrap();
    assert_eq!(response.status(), 401);
  }
}