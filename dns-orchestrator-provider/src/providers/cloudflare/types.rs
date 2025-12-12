//! Cloudflare API 类型定义

use serde::{Deserialize, Serialize};

/// Cloudflare API 通用响应
#[derive(Debug, Deserialize)]
pub struct CloudflareResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    pub errors: Option<Vec<CloudflareError>>,
    pub result_info: Option<CloudflareResultInfo>,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareError {
    #[allow(dead_code)]
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudflareResultInfo {
    #[allow(dead_code)]
    pub page: u32,
    #[allow(dead_code)]
    pub per_page: u32,
    pub total_count: u32,
}

/// Cloudflare Zone 结构
#[derive(Debug, Deserialize)]
pub struct CloudflareZone {
    pub id: String,
    pub name: String,
    pub status: String,
}

/// Cloudflare DNS Record 结构
#[derive(Debug, Deserialize, Serialize)]
pub struct CloudflareDnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_on: Option<String>,
}
