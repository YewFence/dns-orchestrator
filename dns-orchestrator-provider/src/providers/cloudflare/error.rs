//! Cloudflare 错误映射

use crate::error::ProviderError;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::CloudflareProvider;

/// Cloudflare 错误码映射
/// 参考: <https://api.cloudflare.com/#getting-started-responses>
impl ProviderErrorMapper for CloudflareProvider {
    fn provider_name(&self) -> &'static str {
        "cloudflare"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // 认证错误
            Some("9109" | "10000") => ProviderError::InvalidCredentials {
                provider: self.provider_name().to_string(),
                raw_message: Some(raw.message),
            },
            // 记录已存在
            Some("81057") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 记录不存在
            Some("81044") => ProviderError::RecordNotFound {
                provider: self.provider_name().to_string(),
                record_id: context.record_id.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // Zone 不存在
            Some("7003") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}
