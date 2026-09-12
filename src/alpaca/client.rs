//! Alpaca REST API client.
//!
//! All HTTP calls to Alpaca go through `AlpacaClient`.
//! This keeps Alpaca-specific auth/URL logic in one place.

use crate::config::Config;
use anyhow::Result;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};

/// Thin wrapper around `reqwest::Client` with Alpaca credentials.
#[derive(Clone)]
pub struct AlpacaClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
}

impl AlpacaClient {
    /// Create a new client from the application config.
    ///
    /// # Errors
    /// Returns an error if the API key/secret contain non-ASCII characters.
    pub fn new(config: &Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "APCA-API-KEY-ID",
            HeaderValue::from_str(&config.api_key)?,
        );
        headers.insert(
            "APCA-API-SECRET-KEY",
            HeaderValue::from_str(&config.api_secret)?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let http = Client::builder()
            .default_headers(headers)
            .build()?;

        Ok(Self {
            http,
            base_url: config.base_url.clone(),
        })
    }
}
