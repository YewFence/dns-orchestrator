//! 阿里云错误映射

use crate::error::ProviderError;
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::AliyunProvider;

/// 阿里云错误码映射
/// 参考: <https://api.aliyun.com/document/Alidns/2015-01-09/errorCode>
impl ProviderErrorMapper for AliyunProvider {
    fn provider_name(&self) -> &'static str {
        "aliyun"
    }

    fn map_error(&self, raw: RawApiError, context: ErrorContext) -> ProviderError {
        match raw.code.as_deref() {
            // 认证错误
            Some("InvalidAccessKeyId.NotFound" | "SignatureDoesNotMatch") => {
                ProviderError::InvalidCredentials {
                    provider: self.provider_name().to_string(),
                    raw_message: Some(raw.message),
                }
            }
            // 记录已存在
            Some("DomainRecordDuplicate") => ProviderError::RecordExists {
                provider: self.provider_name().to_string(),
                record_name: context.record_name.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 记录不存在
            Some("DomainRecordNotBelongToUser" | "InvalidRecordId.NotFound") => {
                ProviderError::RecordNotFound {
                    provider: self.provider_name().to_string(),
                    record_id: context.record_id.unwrap_or_default(),
                    raw_message: Some(raw.message),
                }
            }
            // 域名不存在
            Some("InvalidDomainName.NoExist") => ProviderError::DomainNotFound {
                provider: self.provider_name().to_string(),
                domain: context.domain.unwrap_or_default(),
                raw_message: Some(raw.message),
            },
            // 其他错误 fallback
            _ => self.unknown_error(raw),
        }
    }
}
