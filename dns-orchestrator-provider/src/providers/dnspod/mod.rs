//! 腾讯云 DNSPod Provider

mod error;
mod http;
mod provider;
mod sign;
mod types;

use reqwest::Client;

pub(crate) use types::{
    CreateRecordResponse, DomainListResponse, ModifyRecordResponse, RecordListResponse,
    TencentResponse,
};

pub(crate) const DNSPOD_API_HOST: &str = "dnspod.tencentcloudapi.com";
pub(crate) const DNSPOD_SERVICE: &str = "dnspod";
pub(crate) const DNSPOD_VERSION: &str = "2021-03-23";

/// 腾讯云 DNSPod Provider
pub struct DnspodProvider {
    pub(crate) client: Client,
    pub(crate) secret_id: String,
    pub(crate) secret_key: String,
}

impl DnspodProvider {
    pub fn new(secret_id: String, secret_key: String) -> Self {
        Self {
            client: Client::new(),
            secret_id,
            secret_key,
        }
    }
}
