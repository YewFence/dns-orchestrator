//! 错误处理模块

use actix_web::{HttpResponse, ResponseError};
use serde::Serialize;

/// API 错误类型
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("账户未找到: {0}")]
    AccountNotFound(String),

    #[error("Provider 未找到: {0}")]
    ProviderNotFound(String),

    #[error("域名未找到: {0}")]
    DomainNotFound(String),

    #[error("记录未找到: {0}")]
    RecordNotFound(String),

    #[error("凭证验证失败: {0}")]
    CredentialValidation(String),

    #[error("Provider 错误: {0}")]
    Provider(String),

    #[error("数据库错误: {0}")]
    Database(String),

    #[error("加密错误: {0}")]
    Encryption(String),

    #[error("请求参数错误: {0}")]
    BadRequest(String),

    #[error("未知命令: {0}")]
    UnknownCommand(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// API 响应包装
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}


impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status = match self {
            Self::AccountNotFound(_)
            | Self::ProviderNotFound(_)
            | Self::DomainNotFound(_)
            | Self::RecordNotFound(_) => actix_web::http::StatusCode::NOT_FOUND,
            Self::BadRequest(_) | Self::UnknownCommand(_) => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
            Self::CredentialValidation(_) => actix_web::http::StatusCode::UNAUTHORIZED,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        };

        HttpResponse::build(status).json(ApiResponse::<()>::error(self.to_string()))
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::Database(err.to_string())
    }
}

impl From<dns_orchestrator_provider::ProviderError> for ApiError {
    fn from(err: dns_orchestrator_provider::ProviderError) -> Self {
        Self::Provider(err.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err.to_string())
    }
}
