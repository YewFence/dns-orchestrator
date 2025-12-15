//! Provider 元数据服务
//!
//! 提供 DNS Provider 的静态元数据信息（无状态服务）

use dns_orchestrator_provider::get_all_provider_metadata;

use crate::types::ProviderMetadata;

/// Provider 元数据服务（无状态）
pub struct ProviderMetadataService;

impl ProviderMetadataService {
    /// 创建 Provider 元数据服务实例
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// 获取所有支持的提供商列表
    pub fn list_providers(&self) -> Vec<ProviderMetadata> {
        get_all_provider_metadata()
    }
}

impl Default for ProviderMetadataService {
    fn default() -> Self {
        Self::new()
    }
}
