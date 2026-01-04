use reqwest::{Method, Client};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::str::FromStr;
use anyhow::{Result, Context};
use crate::app::HttpMethod;

#[derive(Debug)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: String,
    pub body: String,
}

pub async fn make_request(
    method: HttpMethod,
    url: String,
    headers_str: String,
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

    let mut headers = HeaderMap::new();
    for line in headers_str.lines() {
        if let Some((k, v)) = line.split_once(':') {
            if let (Ok(hn), Ok(hv)) = (HeaderName::from_str(k.trim()), HeaderValue::from_str(v.trim())) {
                headers.insert(hn, hv);
            }
        }
    }

    let mut builder = client.request(req_method, &url)
        .headers(headers);
    
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
