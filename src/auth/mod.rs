use serde::{Deserialize, Serialize};

/// Auth profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProfile {
    pub name: String,
    #[serde(rename = "type")]
    pub auth_type: AuthType,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum AuthType {
    Bearer,
    Basic,
    ApiKey,
    OAuth2,
    #[default]
    None,
}

impl AuthProfile {
    /// Resolve environment variable references in token/password/key fields.
    /// Syntax: `${ENV_VAR}` is replaced with the value of `ENV_VAR`.
    pub fn resolve_env_vars(&mut self) {
        if let Some(ref mut token) = self.token {
            *token = resolve_env(token);
        }
        if let Some(ref mut password) = self.password {
            *password = resolve_env(password);
        }
        if let Some(ref mut key) = self.key {
            *key = resolve_env(key);
        }
    }

    /// Build the HTTP headers this auth profile should inject.
    pub fn to_headers(&self) -> Vec<(String, String)> {
        match self.auth_type {
            AuthType::Bearer => {
                if let Some(ref token) = self.token {
                    vec![("Authorization".into(), format!("Bearer {token}"))]
                } else {
                    vec![]
                }
            }
            AuthType::Basic => {
                if let (Some(user), Some(pass)) = (&self.username, &self.password) {
                    let encoded = base64_encode(format!("{user}:{pass}").as_bytes());
                    vec![("Authorization".into(), format!("Basic {encoded}"))]
                } else {
                    vec![]
                }
            }
            AuthType::ApiKey => {
                if let (Some(header), Some(key)) = (&self.header, &self.key) {
                    vec![(header.clone(), key.clone())]
                } else {
                    vec![]
                }
            }
            AuthType::OAuth2 | AuthType::None => vec![],
        }
    }
}

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

fn base64_encode(input: &[u8]) -> String {
    STANDARD.encode(input)
}

/// Resolve `${VAR}` patterns in a string, replacing with the env var value
/// or leaving unchanged if not set.
fn resolve_env(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_name = String::new();
            for vc in chars.by_ref() {
                if vc == '}' {
                    break;
                }
                var_name.push(vc);
            }
            match std::env::var(&var_name) {
                Ok(val) => result.push_str(&val),
                Err(_) => result.push_str(&format!("${{{var_name}}}")),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_env_no_vars() {
        assert_eq!(resolve_env("hello world"), "hello world");
    }

    #[test]
    fn test_resolve_env_with_var() {
        unsafe {
            std::env::set_var("NETINJECT_TEST_RESOLVE", "resolved_value");
        }
        assert_eq!(
            resolve_env("token_${NETINJECT_TEST_RESOLVE}_end"),
            "token_resolved_value_end"
        );
        unsafe {
            std::env::remove_var("NETINJECT_TEST_RESOLVE");
        }
    }

    #[test]
    fn test_resolve_env_missing_var() {
        assert_eq!(
            resolve_env("${NETINJECT_SURELY_MISSING}"),
            "${NETINJECT_SURELY_MISSING}"
        );
    }

    #[test]
    fn test_bearer_headers() {
        let auth = AuthProfile {
            name: "test".into(),
            auth_type: AuthType::Bearer,
            token: Some("mytoken".into()),
            username: None,
            password: None,
            header: None,
            key: None,
        };
        let headers = auth.to_headers();
        assert_eq!(
            headers,
            vec![("Authorization".into(), "Bearer mytoken".into())]
        );
    }

    #[test]
    fn test_apikey_headers() {
        let auth = AuthProfile {
            name: "test".into(),
            auth_type: AuthType::ApiKey,
            token: None,
            username: None,
            password: None,
            header: Some("X-API-Key".into()),
            key: Some("secret123".into()),
        };
        let headers = auth.to_headers();
        assert_eq!(headers, vec![("X-API-Key".into(), "secret123".into())]);
    }
}
