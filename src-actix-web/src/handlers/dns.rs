//! DNS 记录处理模块

use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

// ============ 请求参数类型 ============

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDnsRecordsArgs {
    pub account_id: String,
    pub domain_id: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub keyword: Option<String>,
    pub record_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDnsRecordArgs {
    pub account_id: String,
    pub request: CreateDnsRecordRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDnsRecordRequest {
    pub domain_id: String,
    pub name: String,
    pub record_type: String,
    pub content: String,
    pub ttl: Option<u32>,
    pub priority: Option<u16>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDnsRecordArgs {
    pub account_id: String,
    pub record_id: String,
    pub request: UpdateDnsRecordRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDnsRecordRequest {
    pub domain_id: String,
    pub name: Option<String>,
    pub record_type: Option<String>,
    pub content: Option<String>,
    pub ttl: Option<u32>,
    pub priority: Option<u16>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDnsRecordArgs {
    pub account_id: String,
    pub record_id: String,
    pub domain_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteDnsRecordsArgs {
    pub account_id: String,
    pub request: BatchDeleteRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteRequest {
    pub domain_id: String,
    pub record_ids: Vec<String>,
}

// ============ 响应类型 ============

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsRecordInfo {
    pub id: String,
    pub name: String,
    pub record_type: String,
    pub content: String,
    pub ttl: u32,
    pub priority: Option<u16>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

// ============ Handler 实现 ============

/// 获取 DNS 记录列表
pub async fn list_dns_records(
    state: &AppState,
    args: ListDnsRecordsArgs,
) -> Result<PaginatedResponse<DnsRecordInfo>, ApiError> {
    let provider = state
        .registry
        .get(&args.account_id)
        .await
        .ok_or_else(|| ApiError::AccountNotFound(args.account_id.clone()))?;

    let params = dns_orchestrator_provider::RecordQueryParams {
        page: args.page.unwrap_or(1),
        page_size: args.page_size.unwrap_or(20),
        keyword: args.keyword,
        record_type: args.record_type.and_then(|t| parse_record_type(&t)),
    };

    let result = provider.list_records(&args.domain_id, &params).await?;

    Ok(PaginatedResponse {
        items: result.items.into_iter().map(convert_record).collect(),
        total: result.total,
        page: result.page,
        page_size: result.page_size,
        total_pages: result.total_pages,
    })
}

/// 创建 DNS 记录
pub async fn create_dns_record(
    state: &AppState,
    args: CreateDnsRecordArgs,
) -> Result<DnsRecordInfo, ApiError> {
    let provider = state
        .registry
        .get(&args.account_id)
        .await
        .ok_or_else(|| ApiError::AccountNotFound(args.account_id.clone()))?;

    let record_type = parse_record_type(&args.request.record_type)
        .ok_or_else(|| ApiError::BadRequest(format!("无效的记录类型: {}", args.request.record_type)))?;

    let req = dns_orchestrator_provider::CreateDnsRecordRequest {
        domain_id: args.request.domain_id,
        name: args.request.name,
        record_type,
        content: args.request.content,
        ttl: args.request.ttl,
        priority: args.request.priority,
        proxied: args.request.proxied,
    };

    let record = provider.create_record(&req).await?;
    Ok(convert_record(record))
}

/// 更新 DNS 记录
pub async fn update_dns_record(
    state: &AppState,
    args: UpdateDnsRecordArgs,
) -> Result<DnsRecordInfo, ApiError> {
    let provider = state
        .registry
        .get(&args.account_id)
        .await
        .ok_or_else(|| ApiError::AccountNotFound(args.account_id.clone()))?;

    let record_type = args
        .request
        .record_type
        .as_ref()
        .and_then(|t| parse_record_type(t));

    let req = dns_orchestrator_provider::UpdateDnsRecordRequest {
        domain_id: args.request.domain_id,
        name: args.request.name,
        record_type,
        content: args.request.content,
        ttl: args.request.ttl,
        priority: args.request.priority,
        proxied: args.request.proxied,
    };

    let record = provider.update_record(&args.record_id, &req).await?;
    Ok(convert_record(record))
}

/// 删除 DNS 记录
pub async fn delete_dns_record(state: &AppState, args: DeleteDnsRecordArgs) -> Result<(), ApiError> {
    let provider = state
        .registry
        .get(&args.account_id)
        .await
        .ok_or_else(|| ApiError::AccountNotFound(args.account_id.clone()))?;

    provider
        .delete_record(&args.record_id, &args.domain_id)
        .await?;
    Ok(())
}

/// 批量删除 DNS 记录
pub async fn batch_delete_dns_records(
    state: &AppState,
    args: BatchDeleteDnsRecordsArgs,
) -> Result<BatchDeleteResult, ApiError> {
    let provider = state
        .registry
        .get(&args.account_id)
        .await
        .ok_or_else(|| ApiError::AccountNotFound(args.account_id.clone()))?;

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut errors = Vec::new();

    for record_id in &args.request.record_ids {
        match provider
            .delete_record(record_id, &args.request.domain_id)
            .await
        {
            Ok(()) => success_count += 1,
            Err(e) => {
                failed_count += 1;
                errors.push(format!("删除 {} 失败: {}", record_id, e));
            }
        }
    }

    Ok(BatchDeleteResult {
        success_count,
        failed_count,
        errors,
    })
}

// ============ 辅助函数 ============

fn parse_record_type(s: &str) -> Option<dns_orchestrator_provider::DnsRecordType> {
    use dns_orchestrator_provider::DnsRecordType;
    match s.to_uppercase().as_str() {
        "A" => Some(DnsRecordType::A),
        "AAAA" => Some(DnsRecordType::AAAA),
        "CNAME" => Some(DnsRecordType::CNAME),
        "MX" => Some(DnsRecordType::MX),
        "TXT" => Some(DnsRecordType::TXT),
        "NS" => Some(DnsRecordType::NS),
        "SRV" => Some(DnsRecordType::SRV),
        "CAA" => Some(DnsRecordType::CAA),
        _ => None,
    }
}

fn convert_record(record: dns_orchestrator_provider::DnsRecord) -> DnsRecordInfo {
    DnsRecordInfo {
        id: record.id,
        name: record.name,
        record_type: format!("{:?}", record.record_type),
        content: record.content,
        ttl: record.ttl,
        priority: record.priority,
        proxied: record.proxied,
    }
}
