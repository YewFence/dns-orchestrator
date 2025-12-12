//! 华为云错误映射

use crate::error::ProviderError;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::HuaweicloudProvider;

/// 华为云错误码映射
/// 参考: <https://support.huaweicloud.com/api-dns/ErrorCode.html>
impl ProviderErrorMapper for HuaweicloudProvider {
    fn provider_name(&self) -> &'static str {
        "huaweicloud"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // 认证错误
            Some("APIGW.0301" | "APIGW.0101") => ProviderError::InvalidCredentials {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message),
            },
            // 记录已存在
            Some("DNS.0312") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 记录不存在
            Some("DNS.0305") => ProviderError::RecordNotFound {
                provider: self.provider_name().to_string(),
                record_id: context.record_id.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // Zone 不存在
            Some("DNS.0101") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}
