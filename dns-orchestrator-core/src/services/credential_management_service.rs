//! 凭证管理服务
//!
//! 负责凭证的验证、保存、加载、删除，以及 Provider 实例的注册和管理

use std::collections::HashMap;
use std::sync::Arc;

use dns_orchestrator_provider::{create_provider, DnsProvider, ProviderCredentials, ProviderType};

use crate::error::{CoreError, CoreResult};
use crate::traits::{CredentialStore, ProviderRegistry};

/// 凭证管理服务
pub struct CredentialManagementService {
    credential_store: Arc<dyn CredentialStore>,
    provider_registry: Arc<dyn ProviderRegistry>,
}

impl CredentialManagementService {
    /// 创建凭证管理服务实例
    #[must_use]
    pub fn new(
        credential_store: Arc<dyn CredentialStore>,
        provider_registry: Arc<dyn ProviderRegistry>,
    ) -> Self {
        Self {
            credential_store,
            provider_registry,
        }
    }

    /// 验证凭证并创建 Provider 实例
    pub async fn validate_and_create_provider(
        &self,
        provider_type: &ProviderType,
        credentials: &HashMap<String, String>,
    ) -> CoreResult<Arc<dyn DnsProvider>> {
        // 1. 转换凭证
        let provider_credentials = ProviderCredentials::from_map(provider_type, credentials)
            .map_err(CoreError::CredentialValidation)?;

        // 2. 创建 Provider
        let provider = create_provider(provider_credentials)?;

        // 3. 验证凭证
        let is_valid = provider.validate_credentials().await?;
        if !is_valid {
            return Err(CoreError::InvalidCredentials(provider_type.to_string()));
        }

        Ok(provider)
    }

    /// 保存凭证
    pub async fn save_credentials(
        &self,
        account_id: &str,
        credentials: &HashMap<String, String>,
    ) -> CoreResult<()> {
        self.credential_store
            .save(account_id, credentials)
            .await
            .map_err(|e| CoreError::CredentialError(e.to_string()))
    }

    /// 加载凭证
    pub async fn load_credentials(&self, account_id: &str) -> CoreResult<HashMap<String, String>> {
        self.credential_store
            .load(account_id)
            .await
            .map_err(|e| CoreError::CredentialError(e.to_string()))
    }

    /// 删除凭证
    pub async fn delete_credentials(&self, account_id: &str) -> CoreResult<()> {
        self.credential_store
            .delete(account_id)
            .await
            .map_err(|e| CoreError::CredentialError(e.to_string()))
    }

    /// 加载所有凭证
    pub async fn load_all_credentials(
        &self,
    ) -> CoreResult<HashMap<String, HashMap<String, String>>> {
        self.credential_store
            .load_all()
            .await
            .map_err(|e| CoreError::CredentialError(e.to_string()))
    }

    /// 注册 Provider 到 Registry
    pub async fn register_provider(&self, account_id: String, provider: Arc<dyn DnsProvider>) {
        self.provider_registry.register(account_id, provider).await;
    }

    /// 注销 Provider
    pub async fn unregister_provider(&self, account_id: &str) {
        self.provider_registry.unregister(account_id).await;
    }
}
