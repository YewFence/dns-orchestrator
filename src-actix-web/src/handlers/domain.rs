//! Domain 处理模块

use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::state::AppState;

// ============ 请求参数类型 ============

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDomainsArgs {
    pub account_id: String,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDomainArgs {
    pub account_id: String,
    pub domain_id: String,
}

// ============ 响应类型 ============

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomainInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub record_count: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: u32,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

// ============ Handler 实现 ============

/// 获取域名列表
pub async fn list_domains(
    state: &AppState,
    args: ListDomainsArgs,
) -> Result<PaginatedResponse<DomainInfo>, ApiError> {
    let provider = state
        .registry
        .get(&args.account_id)
        .await
        .ok_or_else(|| ApiError::AccountNotFound(args.account_id.clone()))?;

    let params = dns_orchestrator_provider::PaginationParams {
        page: args.page.unwrap_or(1),
        page_size: args.page_size.unwrap_or(20),
    };

    let result = provider.list_domains(&params).await?;

    Ok(PaginatedResponse {
        items: result
            .items
            .into_iter()
            .map(|d| DomainInfo {
                id: d.id,
                name: d.name,
                status: format!("{:?}", d.status),
                record_count: d.record_count,
            })
            .collect(),
        total_count: result.total_count,
        page: result.page,
        page_size: result.page_size,
        has_more: result.has_more,
    })
}

/// 获取域名详情
pub async fn get_domain(state: &AppState, args: GetDomainArgs) -> Result<DomainInfo, ApiError> {
    let provider = state
        .registry
        .get(&args.account_id)
        .await
        .ok_or_else(|| ApiError::AccountNotFound(args.account_id.clone()))?;

    let domain = provider.get_domain(&args.domain_id).await?;

    Ok(DomainInfo {
        id: domain.id,
        name: domain.name,
        status: format!("{:?}", domain.status),
        record_count: domain.record_count,
    })
}
