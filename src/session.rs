use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use log::debug;
use quick_xml::de::from_str as xml_from_str;
use quick_xml::se::Serializer as XmlSerializer;
use reqwest::header::{CONTENT_TYPE, HeaderValue};
use reqwest::{Client, ClientBuilder};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::normalize_router_url;

/// Huawei router session handling CSRF tokens and XML API communication.
pub struct HuaweiSession {
    url: String,
    client: Client,
    tokens: Mutex<Vec<String>>,
}

impl HuaweiSession {
    pub fn new(url: &str) -> Result<Self> {
        let target = normalize_router_url(url)?;

        #[allow(unused_mut)]
        let mut builder = ClientBuilder::new()
            .cookie_store(true)
            .pool_max_idle_per_host(0)
            .timeout(Duration::from_secs(60));

        #[cfg(any(feature = "tls-native", feature = "tls-rustls"))]
        {
            builder = builder.danger_accept_invalid_certs(true);
        }

        let client = builder.build()?;

        let session = Self {
            url: target,
            client,
            tokens: Mutex::new(Vec::new()),
        };

        Ok(session)
    }

    /// Initialize CSRF tokens by fetching the home page and/or token API.
    pub async fn initialize(&self) -> Result<()> {
        // Try to get tokens from the home page first
        let resp = self
            .client
            .get(&self.url)
            .send()
            .await
            .context("Failed to fetch home page")?;
        let body = resp.text().await.unwrap_or_default();

        let mut tokens = Vec::new();

        // Parse csrf_token from HTML meta tags
        for cap in regex_lite_find_csrf(&body) {
            tokens.push(cap);
        }

        if tokens.is_empty() {
            // Try the token API endpoint
            if let Ok(token) = self.get_token().await {
                tokens.push(token);
            }
        }

        if tokens.is_empty() {
            // Try SesTokInfo endpoint
            if let Ok(tok) = self.get_ses_tok_info().await {
                tokens.push(tok);
            }
        }

        *self.tokens.lock().unwrap() = tokens;
        Ok(())
    }

    async fn get_token(&self) -> Result<String> {
        let url = format!("{}api/webserver/token", self.url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get token")?;
        let body = resp.text().await?;
        let parsed: Value = parse_xml_to_json(&body)?;
        parsed
            .get("token")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No token in response"))
    }

    async fn get_ses_tok_info(&self) -> Result<String> {
        let url = format!("{}api/webserver/SesTokInfo", self.url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to get SesTokInfo")?;
        let body = resp.text().await?;
        let parsed: Value = parse_xml_to_json(&body)?;
        parsed
            .get("TokInfo")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("No TokInfo in response"))
    }

    fn build_url(&self, endpoint: &str) -> String {
        format!("{}api/{}", self.url, endpoint)
    }

    fn pop_token(&self) -> Option<String> {
        let mut tokens = self.tokens.lock().unwrap();
        if tokens.len() > 1 {
            Some(tokens.remove(0))
        } else {
            tokens.first().cloned()
        }
    }

    fn push_token(&self, token: String) {
        self.tokens.lock().unwrap().push(token);
    }

    /// Send a GET request to an API endpoint and parse the XML response.
    pub async fn api_get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = self.build_url(endpoint);

        let mut req = self.client.get(&url);

        if let Some(token) = self.pop_token() {
            req = req.header("__RequestVerificationToken", &token);
        }

        debug!("GET {}", url);
        let resp = req.send().await.context("API GET request failed")?;

        self.extract_tokens_from_response(&resp);

        let body = resp.text().await?;
        debug!("Response body: {}", &body);
        parse_xml_response(&body)
    }

    /// Send a GET request and return the raw JSON Value for flexible parsing.
    pub async fn api_get_raw(&self, endpoint: &str) -> Result<serde_json::Value> {
        let url = self.build_url(endpoint);

        let mut req = self.client.get(&url);

        if let Some(token) = self.pop_token() {
            req = req.header("__RequestVerificationToken", &token);
        }

        debug!("GET {}", url);
        let resp = req.send().await.context("API GET request failed")?;

        self.extract_tokens_from_response(&resp);

        let body = resp.text().await?;
        debug!("Response body: {}", &body);
        parse_xml_response_raw(&body)
    }

    /// Send a POST (set) request to an API endpoint with XML body.
    pub async fn api_post_set(
        &self,
        endpoint: &str,
        data: &impl Serialize,
    ) -> Result<String> {
        let url = self.build_url(endpoint);
        let xml_body = build_request_xml(data)?;

        let mut req = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/xml"))
            .body(xml_body.clone());

        if let Some(token) = self.pop_token() {
            req = req.header("__RequestVerificationToken", &token);
        }

        debug!("POST {} body={}", url, xml_body);
        let resp = req.send().await.context("API POST request failed")?;

        self.extract_tokens_from_response(&resp);

        let body = resp.text().await?;
        debug!("Response body: {}", &body);
        check_response_for_error(&body)?;
        Ok(body)
    }

    /// Send a POST (set) request and refresh CSRF tokens afterwards.
    pub async fn api_post_set_refresh(
        &self,
        endpoint: &str,
        data: &impl Serialize,
    ) -> Result<String> {
        // Clear existing tokens before the call
        {
            let mut tokens = self.tokens.lock().unwrap();
            tokens.clear();
        }
        self.api_post_set(endpoint, data).await
    }

    fn extract_tokens_from_response(&self, resp: &reqwest::Response) {
        let headers = resp.headers();

        if let Some(tok1) = headers.get("__RequestVerificationTokenone") {
            if let Ok(s) = tok1.to_str() {
                self.push_token(s.to_string());
                if let Some(tok2) = headers.get("__RequestVerificationTokentwo") {
                    if let Ok(s2) = tok2.to_str() {
                        self.push_token(s2.to_string());
                    }
                }
            }
        } else if let Some(tok) = headers.get("__RequestVerificationToken") {
            if let Ok(s) = tok.to_str() {
                self.push_token(s.to_string());
            }
        }
    }

    /// Get the state-login info to determine password type.
    pub async fn get_state_login(&self) -> Result<StateLogin> {
        self.api_get("user/state-login").await
    }

    /// Encode password for login.
    ///
    /// Password type 4 (SHA256):
    ///   base64(sha256(username + base64(sha256(password).hex()) + token).hex())
    ///
    /// Password type 0 (base64):
    ///   base64(password)
    pub fn encode_password(
        &self,
        username: &str,
        password: &str,
        password_type: i32,
    ) -> String {
        if password_type == 4 {
            // SHA256 mode
            let token = {
                let tokens = self.tokens.lock().unwrap();
                tokens.first().cloned().unwrap_or_default()
            };

            let pw_hash = {
                let mut hasher = Sha256::new();
                hasher.update(password.as_bytes());
                hex::encode(hasher.finalize())
            };
            let pw_b64 = BASE64.encode(pw_hash.as_bytes());

            let concentrated = format!("{}{}{}", username, pw_b64, token);
            let mut hasher = Sha256::new();
            hasher.update(concentrated.as_bytes());
            let final_hash = hex::encode(hasher.finalize());
            BASE64.encode(final_hash.as_bytes())
        } else {
            // Base64 mode
            BASE64.encode(password.as_bytes())
        }
    }

    /// Login to the router.
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let state = self.get_state_login().await?;
        let password_type = state.password_type.unwrap_or(0);

        let encoded_pw = self.encode_password(username, password, password_type);

        #[derive(Serialize)]
        struct LoginRequest {
            #[serde(rename = "Username")]
            username: String,
            #[serde(rename = "Password")]
            password: String,
            password_type: i32,
        }

        let req = LoginRequest {
            username: username.to_string(),
            password: encoded_pw,
            password_type,
        };

        let xml_body = build_request_xml(&req)?;

        let url = self.build_url("user/login");

        let mut http_req = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, HeaderValue::from_static("application/xml"))
            .body(xml_body);

        if let Some(token) = self.pop_token() {
            http_req = http_req.header("__RequestVerificationToken", &token);
        }

        // Clear tokens on login
        {
            let mut tokens = self.tokens.lock().unwrap();
            tokens.clear();
        }

        let resp = http_req.send().await.context("Login request failed")?;
        self.extract_tokens_from_response(&resp);

        let body = resp.text().await?;
        check_response_for_error(&body)?;

        Ok(())
    }

    /// Logout from the router.
    pub async fn logout(&self) -> Result<()> {
        #[derive(Serialize)]
        struct LogoutRequest {
            #[serde(rename = "Logout")]
            logout: i32,
        }

        self.api_post_set("user/logout", &LogoutRequest { logout: 1 })
            .await?;
        Ok(())
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct StateLogin {
    #[serde(rename = "State")]
    pub state: Option<String>,
    pub password_type: Option<i32>,
    #[serde(rename = "extern_password_type")]
    pub extern_password_type: Option<i32>,
}

/// Build an XML `<request>` element from a serializable struct.
fn build_request_xml(data: &impl Serialize) -> Result<String> {
    let mut buf = String::new();
    let ser = XmlSerializer::with_root(&mut buf, Some("request"))
        .context("Failed to create XML serializer")?;
    data.serialize(ser).context("Failed to serialize to XML")?;
    Ok(buf)
}

/// Parse an XML body into a JSON Value (handles `<response>` envelope).
fn parse_xml_to_json(body: &str) -> Result<Value> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(Value::Object(Default::default()));
    }

    let value: Value = xml_from_str(trimmed)
        .context("Failed to parse XML response")?;

    Ok(value)
}

/// Parse XML response body, extracting the inner content of `<response>` element.
fn parse_xml_response<T: DeserializeOwned>(body: &str) -> Result<T> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        bail!("Empty response body");
    }

    // First check for error
    check_response_for_error(trimmed)?;

    // Try to find <response> tag and extract the content
    if let Some(start) = trimmed.find("<response>") {
        let content = &trimmed[start..];
        // Wrap it for deserialization — the content between <response>...</response>
        let inner_start = content.find('>').unwrap() + 1;
        let inner_end = content
            .rfind("</response>")
            .unwrap_or(content.len());
        let inner = &content[inner_start..inner_end];

        // Build a wrapper XML for deserialization
        let wrapper = format!("<root>{}</root>", inner);
        let result: T = xml_from_str(&wrapper)
            .context("Failed to deserialize response content")?;
        return Ok(result);
    }

    // Fallback: try direct deserialization
    xml_from_str(trimmed).context("Failed to deserialize XML response")
}

/// Parse XML response body into raw JSON Value.
fn parse_xml_response_raw(body: &str) -> Result<Value> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(Value::Object(Default::default()));
    }

    check_response_for_error(trimmed)?;

    let value: Value =
        xml_from_str(trimmed).context("Failed to parse XML to JSON")?;

    // If it's wrapped in a "response" key, unwrap it
    if let Some(inner) = value.as_object().and_then(|m| m.get("response")) {
        return Ok(inner.clone());
    }

    Ok(value)
}

/// Check XML body for `<error>` element and return Err if found.
fn check_response_for_error(body: &str) -> Result<()> {
    if body.contains("<error>") {
        // Try to extract error code
        let code = extract_xml_tag(body, "code").unwrap_or_default();
        let message = extract_xml_tag(body, "message").unwrap_or_default();
        bail!("API error {}: {}", code, message);
    }
    Ok(())
}

/// Simple XML tag content extraction for error parsing.
fn extract_xml_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}>", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(xml[start..end].to_string())
}

/// Simple regex-free CSRF token extraction from HTML.
fn regex_lite_find_csrf(html: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let needle = "name=\"csrf_token\"";
    let content_prefix = "content=\"";

    let mut search_from = 0;
    while let Some(pos) = html[search_from..].find(needle) {
        let abs_pos = search_from + pos;
        // Look around for content="..." attribute
        let region_start = abs_pos.saturating_sub(100);
        let region_end = (abs_pos + 200).min(html.len());
        let region = &html[region_start..region_end];

        if let Some(content_pos) = region.find(content_prefix) {
            let val_start = content_pos + content_prefix.len();
            if let Some(val_end) = region[val_start..].find('"') {
                let token = region[val_start..val_start + val_end].to_string();
                if !token.is_empty() {
                    tokens.push(token);
                }
            }
        }
        search_from = abs_pos + needle.len();
    }
    tokens
}
