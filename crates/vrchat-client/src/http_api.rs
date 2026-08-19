use std::collections::HashMap;

use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Serialize;
use serde_json::{json, Value};
use url::Url;
pub use vrcx_0_core::text::normalize_text;
use vrcx_0_core::vrchat_endpoints::{
    VRCHAT_API_DEFAULT_ENDPOINT, VRCHAT_API_HOST, VRCHAT_FILES_HOST, VRCHAT_FILES_S3_HOST,
    VRCHAT_FILES_S3_HOST_PREFIX,
};

use crate::web_client::{WebExecuteRequest, WebUploadMode};

#[derive(Debug, thiserror::Error)]
pub enum HttpApiError {
    #[error("{0}")]
    Custom(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiScope {
    Vrchat,
    VrchatMedia,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponsePolicy {
    pub class: ApiResponseClass,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApiResponseClass {
    Ok,
    Auth,
    RateLimited,
    ClientError,
    ServerError,
    Unknown,
}

impl ApiResponseClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Auth => "auth",
            Self::RateLimited => "rateLimited",
            Self::ClientError => "clientError",
            Self::ServerError => "serverError",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for ApiResponseClass {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum HttpApiRequestBody {
    #[default]
    Empty,
    Json(Value),
    Upload(HttpApiUpload),
}

#[derive(Clone, Debug, PartialEq)]
pub enum HttpApiUpload {
    FilePut {
        file_data: Vec<u8>,
        file_mime: String,
        file_md5: Option<String>,
    },
    Image {
        image_data: String,
        post_data: Option<String>,
        matching_dimensions: bool,
    },
    PrintImage {
        image_data: String,
        post_data: Option<String>,
        crop_white_border: bool,
    },
    LegacyImage {
        image_data: String,
        post_data: Option<String>,
    },
}

impl HttpApiRequestBody {
    pub fn as_json(&self) -> Option<&Value> {
        match self {
            Self::Json(value) => Some(value),
            Self::Empty | Self::Upload(_) => None,
        }
    }

    pub fn as_upload(&self) -> Option<&HttpApiUpload> {
        match self {
            Self::Upload(upload) => Some(upload),
            Self::Empty | Self::Json(_) => None,
        }
    }

    pub fn as_upload_mut(&mut self) -> Option<&mut HttpApiUpload> {
        match self {
            Self::Upload(upload) => Some(upload),
            Self::Empty | Self::Json(_) => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HttpApiRequestInput {
    pub url: Option<String>,
    pub path: Option<String>,
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub query_params: Option<HashMap<String, Value>>,
    pub headers: Option<HashMap<String, String>>,
    pub body: HttpApiRequestBody,
    pub skip_empty_query_string: Option<bool>,
}

#[derive(Clone, Debug, Serialize, specta::Type)]
pub struct HttpApiExecuteResponse {
    pub status: i32,
    pub data: String,
}

#[derive(Clone, Debug)]
pub struct ApiJsonResponse {
    pub status: i32,
    pub json: Value,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{message}")]
pub struct VrchatApiFailure {
    pub status_code: i32,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VrchatAuthFailureKind {
    InvalidCredentials,
    MissingCredentials,
    SessionInvalidated,
    Other,
}

impl ApiJsonResponse {
    pub fn parse(status: i32, data: &str) -> Self {
        Self {
            status,
            json: parse_api_json(data),
        }
    }

    pub fn has_error_field(&self) -> bool {
        self.json
            .as_object()
            .is_some_and(|object| object.contains_key("error"))
    }

    pub fn is_failure(&self) -> bool {
        self.status >= 400 || self.has_error_field()
    }

    pub fn error_message(&self) -> Option<String> {
        let object = self.json.as_object();
        api_message_text(Some(&self.json))
            .or_else(|| api_message_text(object.and_then(|record| record.get("error"))))
            .or_else(|| {
                api_message_text(
                    object
                        .and_then(|record| record.get("error"))
                        .and_then(Value::as_object)
                        .and_then(|error| error.get("message")),
                )
            })
            .or_else(|| api_message_text(object.and_then(|record| record.get("message"))))
    }

    pub fn error_message_or(&self, fallback: &str) -> String {
        self.error_message()
            .unwrap_or_else(|| format!("{fallback} ({})", self.status))
    }

    pub fn error_message_with_http_status(&self, fallback: &str) -> String {
        let message = self.error_message().unwrap_or_else(|| fallback.to_string());
        format!("{message} (HTTP {})", self.status)
    }

    pub fn failure_or(&self, fallback: &str) -> Option<VrchatApiFailure> {
        if !self.is_failure() {
            return None;
        }
        Some(self.to_failure(fallback))
    }

    pub fn to_failure(&self, fallback: &str) -> VrchatApiFailure {
        VrchatApiFailure {
            status_code: self.status,
            message: self.error_message().unwrap_or_else(|| fallback.to_string()),
        }
    }
}

impl From<&HttpApiExecuteResponse> for ApiJsonResponse {
    fn from(response: &HttpApiExecuteResponse) -> Self {
        Self::parse(response.status, &response.data)
    }
}

pub fn vrchat_auth_error_message(response: &HttpApiExecuteResponse) -> Option<String> {
    let json = serde_json::from_str::<Value>(&response.data).ok()?;
    let object = json.as_object();
    let error = object.and_then(|record| record.get("error"));
    json.as_str()
        .map(ToOwned::to_owned)
        .or_else(|| auth_scalar_text(object.and_then(|record| record.get("message"))))
        .or_else(|| {
            auth_scalar_text(
                error
                    .and_then(Value::as_object)
                    .and_then(|record| record.get("message")),
            )
        })
        .or_else(|| error.and_then(Value::as_str).map(ToOwned::to_owned))
}

fn auth_scalar_text(value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(text)) => Some(text.trim().to_string()),
        Some(Value::Number(number)) => Some(number.to_string()),
        Some(Value::Bool(flag)) => Some(flag.to_string()),
        _ => None,
    }
    .filter(|text| !text.is_empty())
}

pub fn classify_vrchat_auth_failure(response: &HttpApiExecuteResponse) -> VrchatAuthFailureKind {
    if response.status == 401 {
        let message = vrchat_auth_error_message(response).unwrap_or_default();
        if message.contains("Invalid Username/Email or Password") {
            return VrchatAuthFailureKind::InvalidCredentials;
        }
        if message.contains("Missing Credentials") {
            return VrchatAuthFailureKind::MissingCredentials;
        }
        return VrchatAuthFailureKind::SessionInvalidated;
    }
    if response.status == 403 {
        return VrchatAuthFailureKind::SessionInvalidated;
    }
    VrchatAuthFailureKind::Other
}

pub fn parse_api_json(data: &str) -> Value {
    serde_json::from_str(data).unwrap_or_else(|_| Value::String(data.to_string()))
}

fn api_message_text(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(|message| message.trim_matches('"').to_string())
}

pub fn classify_api_response(status: i32) -> ApiResponsePolicy {
    let class = match status {
        200..=299 => ApiResponseClass::Ok,
        401 => ApiResponseClass::Auth,
        429 => ApiResponseClass::RateLimited,
        400..=499 => ApiResponseClass::ClientError,
        500..=599 => ApiResponseClass::ServerError,
        _ => ApiResponseClass::Unknown,
    };
    ApiResponsePolicy { class }
}

pub fn execute_response(status: i32, data: String) -> HttpApiExecuteResponse {
    HttpApiExecuteResponse { status, data }
}

pub fn require_text(value: impl AsRef<str>, message: &str) -> Result<String, HttpApiError> {
    let value = normalize_text(value);
    if value.is_empty() {
        return Err(HttpApiError::Custom(message.to_string()));
    }
    Ok(value)
}

pub fn encode_path_segment(value: &str) -> String {
    utf8_percent_encode(value, NON_ALPHANUMERIC).to_string()
}

pub fn json_headers() -> HashMap<String, String> {
    HashMap::from([(
        "Content-Type".to_string(),
        "application/json;charset=utf-8".to_string(),
    )])
}

pub fn object_body(value: Option<Value>) -> Value {
    match value {
        Some(value @ Value::Object(_)) => value,
        _ => json!({}),
    }
}

pub fn api_input(
    endpoint: String,
    method: &str,
    path: impl Into<String>,
    body: Option<Value>,
) -> HttpApiRequestInput {
    HttpApiRequestInput {
        endpoint: Some(endpoint),
        method: Some(method.into()),
        path: Some(path.into()),
        headers: body.as_ref().map(|_| json_headers()),
        body: body
            .map(HttpApiRequestBody::Json)
            .unwrap_or(HttpApiRequestBody::Empty),
        ..Default::default()
    }
}

pub fn get_input(
    endpoint: String,
    path: impl Into<String>,
    query_params: HashMap<String, Value>,
) -> HttpApiRequestInput {
    HttpApiRequestInput {
        endpoint: Some(endpoint),
        method: Some("GET".into()),
        path: Some(path.into()),
        query_params: Some(query_params),
        ..Default::default()
    }
}

pub fn query_input(
    endpoint: String,
    method: &str,
    path: impl Into<String>,
    query_params: HashMap<String, Value>,
) -> HttpApiRequestInput {
    HttpApiRequestInput {
        endpoint: Some(endpoint),
        method: Some(method.into()),
        path: Some(path.into()),
        query_params: Some(query_params),
        ..Default::default()
    }
}

pub fn api_input_skip_empty_query_string(
    endpoint: String,
    method: &str,
    path: impl Into<String>,
    body: Value,
) -> HttpApiRequestInput {
    HttpApiRequestInput {
        endpoint: Some(endpoint),
        method: Some(method.into()),
        path: Some(path.into()),
        headers: Some(json_headers()),
        body: HttpApiRequestBody::Json(body),
        skip_empty_query_string: Some(true),
        ..Default::default()
    }
}

pub fn get_input_skip_empty_query_string(
    endpoint: String,
    path: impl Into<String>,
    query_params: HashMap<String, Value>,
) -> HttpApiRequestInput {
    HttpApiRequestInput {
        endpoint: Some(endpoint),
        method: Some("GET".into()),
        path: Some(path.into()),
        query_params: Some(query_params),
        skip_empty_query_string: Some(true),
        ..Default::default()
    }
}

pub fn build_web_execute_request(
    input: HttpApiRequestInput,
    scope: ApiScope,
) -> Result<WebExecuteRequest, HttpApiError> {
    let method = input
        .method
        .as_deref()
        .unwrap_or("GET")
        .to_ascii_uppercase();
    let mut request = WebExecuteRequest::new(build_request_url(&input, scope)?, method.clone());

    if let Some(headers) = input.headers.as_ref().filter(|headers| !headers.is_empty()) {
        request.headers = headers
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
    }

    if let Some(body) = request_body_text(&input, &method)? {
        request.body = Some(body);
    }

    request.upload = match input.body {
        HttpApiRequestBody::Upload(HttpApiUpload::FilePut {
            file_data,
            file_mime,
            file_md5,
        }) => WebUploadMode::FilePut {
            file_data,
            file_mime,
            file_md5,
        },
        HttpApiRequestBody::Upload(HttpApiUpload::Image {
            image_data,
            post_data,
            ..
        }) => WebUploadMode::Image {
            image_data,
            post_data,
        },
        HttpApiRequestBody::Upload(HttpApiUpload::PrintImage {
            image_data,
            post_data,
            ..
        }) => WebUploadMode::PrintImage {
            image_data,
            post_data,
        },
        HttpApiRequestBody::Upload(HttpApiUpload::LegacyImage {
            image_data,
            post_data,
            ..
        }) => WebUploadMode::LegacyImage {
            image_data,
            post_data,
        },
        HttpApiRequestBody::Empty | HttpApiRequestBody::Json(_) => WebUploadMode::None,
    };

    Ok(request)
}

pub fn normalize_vrchat_api_endpoint(endpoint: Option<&str>) -> String {
    let endpoint = endpoint.unwrap_or("").trim().trim_end_matches('/');
    if endpoint.is_empty() {
        VRCHAT_API_DEFAULT_ENDPOINT.to_string()
    } else {
        endpoint.to_string()
    }
}

fn validated_vrchat_api_endpoint(endpoint: Option<&str>) -> Result<String, HttpApiError> {
    let endpoint = normalize_vrchat_api_endpoint(endpoint);
    let url = parse_http_url(&endpoint)?;
    if url.scheme() != "https"
        || url.host_str() != Some(VRCHAT_API_HOST)
        || url.path().trim_end_matches('/') != "/api/1"
    {
        return Err(HttpApiError::Custom(format!(
            "VRChat API endpoint must be {VRCHAT_API_DEFAULT_ENDPOINT}."
        )));
    }
    Ok(endpoint)
}

fn value_as_query_strings(value: &Value, skip_empty_string: bool) -> Vec<String> {
    match value {
        Value::Null => Vec::new(),
        Value::String(value) => {
            if skip_empty_string && value.is_empty() {
                Vec::new()
            } else {
                vec![value.to_string()]
            }
        }
        Value::Bool(value) => vec![value.to_string()],
        Value::Number(value) => vec![value.to_string()],
        other => vec![other.to_string()],
    }
}

fn append_query_params(url: &mut Url, params: &HashMap<String, Value>, skip_empty_string: bool) {
    for (key, value) in params {
        if let Value::Array(values) = value {
            for item in values {
                for text in value_as_query_strings(item, skip_empty_string) {
                    url.query_pairs_mut().append_pair(key, &text);
                }
            }
            continue;
        }

        let values = value_as_query_strings(value, skip_empty_string);
        if values.len() == 1 {
            url.query_pairs_mut().append_pair(key, &values[0]);
        }
    }
}

fn parse_http_url(url: &str) -> Result<Url, HttpApiError> {
    let url =
        Url::parse(url).map_err(|error| HttpApiError::Custom(format!("bad API URL: {error}")))?;
    if url.scheme() != "https" && url.scheme() != "http" {
        return Err(HttpApiError::Custom("unsupported API URL scheme".into()));
    }
    Ok(url)
}

fn is_allowed_vrchat_media_upload_url(url: &Url) -> bool {
    if url.scheme() != "https" {
        return false;
    }
    let Some(host) = url.host_str().map(|host| host.to_ascii_lowercase()) else {
        return false;
    };

    if host == VRCHAT_FILES_HOST {
        return true;
    }
    if host == VRCHAT_API_HOST {
        return url.path().starts_with("/api/1/file/");
    }
    if host == VRCHAT_FILES_S3_HOST
        || (host.starts_with(VRCHAT_FILES_S3_HOST_PREFIX) && host.ends_with(".amazonaws.com"))
    {
        return true;
    }
    if host.starts_with("s3.") && host.ends_with(".amazonaws.com") {
        return url
            .path_segments()
            .and_then(|segments| segments.into_iter().next())
            == Some(VRCHAT_FILES_HOST);
    }
    false
}

fn validate_vrchat_media_upload_url(url: &Url) -> Result<(), HttpApiError> {
    if is_allowed_vrchat_media_upload_url(url) {
        return Ok(());
    }
    Err(HttpApiError::Custom(
        "VRChat media upload URL must be an official VRChat HTTPS upload target.".into(),
    ))
}

fn is_upload_request(input: &HttpApiRequestInput) -> bool {
    matches!(input.body, HttpApiRequestBody::Upload(_))
}

fn validate_upload_scope(input: &HttpApiRequestInput, scope: ApiScope) -> Result<(), HttpApiError> {
    if is_upload_request(input) && !matches!(scope, ApiScope::VrchatMedia) {
        return Err(HttpApiError::Custom(
            "upload options are only allowed for VRChat media requests".into(),
        ));
    }
    Ok(())
}

fn build_request_url(input: &HttpApiRequestInput, scope: ApiScope) -> Result<String, HttpApiError> {
    validate_upload_scope(input, scope)?;

    if let Some(url) = input
        .url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
    {
        let url = parse_http_url(url)?;
        match scope {
            ApiScope::Vrchat | ApiScope::VrchatMedia => {
                if matches!(scope, ApiScope::VrchatMedia) && is_upload_request(input) {
                    validate_vrchat_media_upload_url(&url)?;
                    return Ok(url.to_string());
                }
                return Err(HttpApiError::Custom(
                    "VRChat API requests must use path and endpoint".into(),
                ));
            }
        }
    }

    let path = input
        .path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .ok_or_else(|| HttpApiError::Custom("Missing API request path".into()))?;

    if let Ok(url) = Url::parse(path) {
        match scope {
            ApiScope::Vrchat | ApiScope::VrchatMedia => {
                if matches!(scope, ApiScope::VrchatMedia) && is_upload_request(input) {
                    validate_vrchat_media_upload_url(&url)?;
                    return Ok(url.to_string());
                }
                return Err(HttpApiError::Custom(
                    "VRChat API requests must use relative paths".into(),
                ));
            }
        }
    }

    let base = format!(
        "{}/",
        validated_vrchat_api_endpoint(input.endpoint.as_deref())?
    );
    let mut url = Url::parse(&base)
        .map_err(|error| HttpApiError::Custom(format!("bad API endpoint: {error}")))?
        .join(path.trim_start_matches('/'))
        .map_err(|error| HttpApiError::Custom(format!("bad API path: {error}")))?;

    if let Some(params) = input.query_params.as_ref() {
        append_query_params(
            &mut url,
            params,
            input.skip_empty_query_string.unwrap_or(false),
        );
    }

    Ok(url.to_string())
}

fn normalize_json_body(value: &Value) -> Value {
    if value.is_object() {
        value.clone()
    } else {
        json!({})
    }
}

fn request_body_text(
    input: &HttpApiRequestInput,
    method: &str,
) -> Result<Option<String>, HttpApiError> {
    if method == "GET" {
        return Ok(None);
    }

    let HttpApiRequestBody::Json(body) = &input.body else {
        return Ok(None);
    };
    serde_json::to_string(&normalize_json_body(body))
        .map(Some)
        .map_err(|error| HttpApiError::Custom(format!("serialize API body: {error}")))
}

#[cfg(test)]
mod tests;
