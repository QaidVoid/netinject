use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("failed to parse OpenAPI spec: {0}")]
    ParseFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Normalized API specification, independent of input format (OpenAPI 3.x, Swagger 2.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSpec {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
    pub base_url: Option<String>,
    pub endpoints: Vec<Endpoint>,
    pub auth_schemes: Vec<AuthScheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    pub method: HttpMethod,
    pub path: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: Vec<ResponseSpec>,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
            HttpMethod::Trace => write!(f, "TRACE"),
        }
    }
}

impl From<http::Method> for HttpMethod {
    fn from(m: http::Method) -> Self {
        match m {
            http::Method::GET => HttpMethod::Get,
            http::Method::POST => HttpMethod::Post,
            http::Method::PUT => HttpMethod::Put,
            http::Method::PATCH => HttpMethod::Patch,
            http::Method::DELETE => HttpMethod::Delete,
            http::Method::HEAD => HttpMethod::Head,
            http::Method::OPTIONS => HttpMethod::Options,
            http::Method::TRACE => HttpMethod::Trace,
            _ => HttpMethod::Get,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub location: ParamLocation,
    pub required: bool,
    pub description: Option<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ParamLocation {
    Query,
    Header,
    Path,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub content_types: Vec<String>,
    pub required: bool,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseSpec {
    pub status_code: String,
    pub description: Option<String>,
    pub content_type: Option<String>,
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthScheme {
    pub name: String,
    #[serde(rename = "type")]
    pub auth_type: String,
    pub header_name: Option<String>,
    pub scheme: Option<String>,
    pub bearer_format: Option<String>,
}

/// Parse an OpenAPI 3.x spec from a file path.
pub fn parse_openapi(path: &std::path::Path) -> Result<ApiSpec, SpecError> {
    let content = std::fs::read_to_string(path)?;
    parse_openapi_str(&content)
}

/// Parse an OpenAPI 3.x spec from a string.
pub fn parse_openapi_str(content: &str) -> Result<ApiSpec, SpecError> {
    let spec = oas3::from_json(content).map_err(|e| SpecError::ParseFailed(e.to_string()))?;
    Ok(normalize_oas3(&spec))
}

fn normalize_oas3(spec: &oas3::Spec) -> ApiSpec {
    let info = &spec.info;
    let base_url = spec.servers.first().map(|s| {
        let url = s.url.trim_end_matches('/').to_string();
        if url.is_empty() || url == "/" {
            "/".to_string()
        } else {
            url
        }
    });

    let mut auth_schemes = Vec::new();
    if let Some(components) = &spec.components {
        for (name, scheme_ref) in &components.security_schemes {
            let scheme = match scheme_ref {
                oas3::spec::ObjectOrReference::Ref { .. } => continue,
                oas3::spec::ObjectOrReference::Object(s) => s,
            };
            let (auth_type, header_name, scheme_name, bearer_format) = match scheme {
                oas3::spec::SecurityScheme::ApiKey {
                    name: key_name,
                    location,
                    ..
                } => (
                    "apiKey".to_string(),
                    Some(key_name.clone()),
                    Some(location.clone()),
                    None,
                ),
                oas3::spec::SecurityScheme::Http {
                    scheme: s,
                    bearer_format,
                    ..
                } => (
                    "http".to_string(),
                    None,
                    Some(s.clone()),
                    bearer_format.clone(),
                ),
                oas3::spec::SecurityScheme::OAuth2 { .. } => {
                    ("oauth2".to_string(), None, None, None)
                }
                oas3::spec::SecurityScheme::OpenIdConnect { .. } => {
                    ("openIdConnect".to_string(), None, None, None)
                }
                oas3::spec::SecurityScheme::MutualTls { .. } => {
                    ("mutualTls".to_string(), None, None, None)
                }
            };
            auth_schemes.push(AuthScheme {
                name: name.clone(),
                auth_type,
                header_name,
                scheme: scheme_name,
                bearer_format,
            });
        }
    }

    let mut endpoints = Vec::new();
    for (path, method, op) in spec.operations() {
        let http_method: HttpMethod = method.into();

        let parameters = op
            .parameters
            .iter()
            .filter_map(|p| match p {
                oas3::spec::ObjectOrReference::Ref { .. } => None,
                oas3::spec::ObjectOrReference::Object(pv) => Some(Parameter {
                    name: pv.name.clone(),
                    location: match pv.location {
                        oas3::spec::ParameterIn::Query => ParamLocation::Query,
                        oas3::spec::ParameterIn::Header => ParamLocation::Header,
                        oas3::spec::ParameterIn::Path => ParamLocation::Path,
                        oas3::spec::ParameterIn::Cookie => ParamLocation::Cookie,
                    },
                    required: pv.required.unwrap_or(false),
                    description: pv.description.clone(),
                    schema: None,
                }),
            })
            .collect();

        let request_body = op.request_body.as_ref().and_then(|rb| match rb {
            oas3::spec::ObjectOrReference::Ref { .. } => None,
            oas3::spec::ObjectOrReference::Object(v) => Some(RequestBody {
                content_types: v.content.keys().cloned().collect(),
                required: v.required.unwrap_or(false),
                schema: None,
            }),
        });

        let responses = op
            .responses
            .as_ref()
            .map(|r| {
                r.keys()
                    .map(|code| ResponseSpec {
                        status_code: code.clone(),
                        description: None,
                        content_type: None,
                        schema: None,
                    })
                    .collect()
            })
            .unwrap_or_default();

        endpoints.push(Endpoint {
            method: http_method,
            path,
            summary: op.summary.clone(),
            description: op.description.clone(),
            parameters,
            request_body,
            responses,
            auth_required: !op.security.is_empty(),
        });
    }

    ApiSpec {
        title: info.title.clone(),
        version: info.version.clone(),
        description: info.description.clone(),
        base_url,
        endpoints,
        auth_schemes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_spec() {
        let json = r#"{
            "openapi": "3.0.0",
            "info": { "title": "Test API", "version": "1.0.0" },
            "paths": {
                "/users": {
                    "get": {
                        "summary": "List users",
                        "responses": { "200": { "description": "OK" } }
                    }
                }
            }
        }"#;
        let spec = parse_openapi_str(json).unwrap();
        assert_eq!(spec.title, "Test API");
        assert_eq!(spec.version, "1.0.0");
        assert_eq!(spec.endpoints.len(), 1);
        assert_eq!(spec.endpoints[0].method, HttpMethod::Get);
        assert_eq!(spec.endpoints[0].path, "/users");
    }

    #[test]
    fn test_parse_invalid_spec() {
        let result = parse_openapi_str("not valid json for openapi");
        assert!(result.is_err());
    }
}
