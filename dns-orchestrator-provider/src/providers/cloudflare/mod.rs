//! Cloudflare DNS Provider

mod error;
mod http;
mod provider;
mod types;

use reqwest::Client;

pub(crate) use types::{CloudflareDnsRecord, CloudflareResponse, CloudflareZone};

pub(crate) const CF_API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare DNS Provider
pub struct CloudflareProvider {
    pub(crate) client: Client,
    pub(crate) api_token: String,
}

impl CloudflareProvider {
    pub fn new(api_token: String) -> Self {
        Self {
            client: Client::new(),
            api_token,
        }
    }
}
