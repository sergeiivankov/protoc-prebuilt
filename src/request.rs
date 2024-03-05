use ureq::{ Response, request };

// GitHub API require User-Agent header
static CRATE_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

// Send request to passed URL with passed token in `Authorization` header
// and generated above `User-Agent`
#[allow(clippy::result_large_err)]
pub(crate) fn request_with_token(
  url: &str, token: &Option<String>
) -> Result<Response, ureq::Error> {
  let mut req = request("GET", url).set("User-Agent", CRATE_USER_AGENT);

  if let Some(value) = token {
    req = req.set("Authorization", &format!("Bearer {}", value))
  }

  req.call()
}

#[cfg(test)]
mod test {
  use super::{ CRATE_USER_AGENT, request_with_token };

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