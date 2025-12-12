//! 华为云 DNS Provider

mod error;
mod http;
mod provider;
mod sign;
mod types;

use reqwest::Client;

pub(crate) const HUAWEICLOUD_DNS_HOST: &str = "dns.myhuaweicloud.com";

/// 华为云 DNS Provider
pub struct HuaweicloudProvider {
    pub(crate) client: Client,
    pub(crate) access_key_id: String,
    pub(crate) secret_access_key: String,
}

impl HuaweicloudProvider {
    pub fn new(access_key_id: String, secret_access_key: String) -> Self {
        Self {
            client: Client::new(),
            access_key_id,
            secret_access_key,
        }
    }
}
