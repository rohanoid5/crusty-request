use reqwest::{Method, Client};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;
use anyhow::{Result, Context};
use crate::app::HttpMethod;
use crate::key_value::KeyValueEntries;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: String,
    pub body: String,
}

pub async fn make_request(
    method: HttpMethod,
    url: String,
    headers: &KeyValueEntries,
    params: &KeyValueEntries,
    auth: &KeyValueEntries,
    body_str: String
) -> Result<ApiResponse> {
    let client = Client::new();
    
    let req_method = match method {
        HttpMethod::GET => Method::GET,
        HttpMethod::POST => Method::POST,
        HttpMethod::PUT => Method::PUT,
        HttpMethod::DELETE => Method::DELETE,
        HttpMethod::PATCH => Method::PATCH,
    };

    // Build headers from KeyValueEntries
    let mut header_map = HeaderMap::new();
    for entry in &headers.entries {
        if entry.enabled {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_str(entry.key.trim()),
                HeaderValue::from_str(entry.value.trim())
            ) {
                header_map.insert(hn, hv);
            }
        }
    }

    // Build query params from KeyValueEntries
    let mut query_params = Vec::new();
    for entry in &params.entries {
        if entry.enabled {
            query_params.push((entry.key.clone(), entry.value.clone()));
        }
    }

    // Handle authorization - look for common auth patterns
    for entry in &auth.entries {
        if entry.enabled {
            // Handle Bearer token
            if entry.key.eq_ignore_ascii_case("Authorization") || entry.key.eq_ignore_ascii_case("Bearer") {
                if let Ok(hv) = HeaderValue::from_str(&entry.value) {
                    header_map.insert(reqwest::header::AUTHORIZATION, hv);
                }
            }
            // Handle API Key
            else if entry.key.eq_ignore_ascii_case("API-Key") || entry.key.eq_ignore_ascii_case("X-API-Key") {
                if let (Ok(hn), Ok(hv)) = (
                    HeaderName::from_str(&entry.key),
                    HeaderValue::from_str(&entry.value)
                ) {
                    header_map.insert(hn, hv);
                }
            }
            // Handle username/password for Basic auth
            else if entry.key.eq_ignore_ascii_case("username") {
                // Look for password entry
                if let Some(password_entry) = auth.entries.iter().find(|e| {
                    e.enabled && e.key.eq_ignore_ascii_case("password")
                }) {
                    let credentials = format!("{}:{}", entry.value, password_entry.value);
                    let encoded = general_purpose::STANDARD.encode(credentials.as_bytes());
                    if let Ok(hv) = HeaderValue::from_str(&format!("Basic {}", encoded)) {
                        header_map.insert(reqwest::header::AUTHORIZATION, hv);
                    }
                }
            }
        }
    }

    // Build URL with query params
    let final_url = if query_params.is_empty() {
        url
    } else {
        let query_string = query_params.iter()
            .map(|(k, v)| format!("{}={}", 
                urlencoding::encode(k), 
                urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}?{}", url, query_string)
    };

    let mut builder = client.request(req_method, &final_url)
        .headers(header_map);
    
    // For MVP, if there is body content, assume JSON and attach it.
    if !body_str.trim().is_empty() {
        builder = builder.header("Content-Type", "application/json")
                         .body(body_str);
    }

    let resp = builder.send().await.context("Failed to send request")?;
    
    let status = resp.status().as_u16();
    let headers_text = format!("{:#?}", resp.headers()); // Debug print headers for now
    
    let body_text = resp.text().await.context("Failed to read response body")?;

    // Try to prettify JSON
    let pretty_body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_text) {
        serde_json::to_string_pretty(&json).unwrap_or(body_text)
    } else {
        body_text
    };

    Ok(ApiResponse {
        status,
        headers: headers_text,
        body: pretty_body,
    })
}
