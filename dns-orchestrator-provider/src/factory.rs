//! Provider factory functions and metadata

use std::sync::Arc;

use crate::error::Result;
use crate::traits::DnsProvider;
use crate::types::{
    FieldType, ProviderCredentialField, ProviderCredentials, ProviderFeatures, ProviderMetadata,
    ProviderType,
};

#[cfg(feature = "aliyun")]
use crate::providers::AliyunProvider;
#[cfg(feature = "cloudflare")]
use crate::providers::CloudflareProvider;
#[cfg(feature = "dnspod")]
use crate::providers::DnspodProvider;
#[cfg(feature = "huaweicloud")]
use crate::providers::HuaweicloudProvider;

/// 工厂函数 - 根据凭证类型创建 Provider 实例
pub fn create_provider(credentials: ProviderCredentials) -> Result<Arc<dyn DnsProvider>> {
    match credentials {
        #[cfg(feature = "cloudflare")]
        ProviderCredentials::Cloudflare { api_token } => {
            Ok(Arc::new(CloudflareProvider::new(api_token)))
        }
        #[cfg(feature = "aliyun")]
        ProviderCredentials::Aliyun {
            access_key_id,
            access_key_secret,
        } => Ok(Arc::new(AliyunProvider::new(
            access_key_id,
            access_key_secret,
        ))),
        #[cfg(feature = "dnspod")]
        ProviderCredentials::Dnspod {
            secret_id,
            secret_key,
        } => Ok(Arc::new(DnspodProvider::new(secret_id, secret_key))),
        #[cfg(feature = "huaweicloud")]
        ProviderCredentials::Huaweicloud {
            access_key_id,
            secret_access_key,
        } => Ok(Arc::new(HuaweicloudProvider::new(
            access_key_id,
            secret_access_key,
        ))),
    }
}

/// 获取所有支持的提供商元数据
pub fn get_all_provider_metadata() -> Vec<ProviderMetadata> {
    let mut providers = Vec::new();

    #[cfg(feature = "cloudflare")]
    providers.push(ProviderMetadata {
        id: ProviderType::Cloudflare,
        name: "Cloudflare".to_string(),
        description: "全球领先的 CDN 和 DNS 服务商".to_string(),
        required_fields: vec![ProviderCredentialField {
            key: "apiToken".to_string(),
            label: "API Token".to_string(),
            field_type: FieldType::Password,
            placeholder: Some("输入 Cloudflare API Token".to_string()),
            help_text: Some("在 Cloudflare Dashboard -> My Profile -> API Tokens 创建".to_string()),
        }],
        features: ProviderFeatures { proxy: true },
    });

    #[cfg(feature = "aliyun")]
    providers.push(ProviderMetadata {
        id: ProviderType::Aliyun,
        name: "阿里云 DNS".to_string(),
        description: "阿里云域名解析服务".to_string(),
        required_fields: vec![
            ProviderCredentialField {
                key: "accessKeyId".to_string(),
                label: "AccessKey ID".to_string(),
                field_type: FieldType::Text,
                placeholder: Some("输入 AccessKey ID".to_string()),
                help_text: None,
            },
            ProviderCredentialField {
                key: "accessKeySecret".to_string(),
                label: "AccessKey Secret".to_string(),
                field_type: FieldType::Password,
                placeholder: Some("输入 AccessKey Secret".to_string()),
                help_text: None,
            },
        ],
        features: ProviderFeatures::default(),
    });

    #[cfg(feature = "dnspod")]
    providers.push(ProviderMetadata {
        id: ProviderType::Dnspod,
        name: "腾讯云 DNSPod".to_string(),
        description: "腾讯云 DNS 解析服务".to_string(),
        required_fields: vec![
            ProviderCredentialField {
                key: "secretId".to_string(),
                label: "SecretId".to_string(),
                field_type: FieldType::Text,
                placeholder: Some("输入 SecretId".to_string()),
                help_text: None,
            },
            ProviderCredentialField {
                key: "secretKey".to_string(),
                label: "SecretKey".to_string(),
                field_type: FieldType::Password,
                placeholder: Some("输入 SecretKey".to_string()),
                help_text: None,
            },
        ],
        features: ProviderFeatures::default(),
    });

    #[cfg(feature = "huaweicloud")]
    providers.push(ProviderMetadata {
        id: ProviderType::Huaweicloud,
        name: "华为云 DNS".to_string(),
        description: "华为云云解析服务".to_string(),
        required_fields: vec![
            ProviderCredentialField {
                key: "accessKeyId".to_string(),
                label: "Access Key ID".to_string(),
                field_type: FieldType::Text,
                placeholder: Some("输入 Access Key ID".to_string()),
                help_text: None,
            },
            ProviderCredentialField {
                key: "secretAccessKey".to_string(),
                label: "Secret Access Key".to_string(),
                field_type: FieldType::Password,
                placeholder: Some("输入 Secret Access Key".to_string()),
                help_text: None,
            },
        ],
        features: ProviderFeatures::default(),
    });

    providers
}
