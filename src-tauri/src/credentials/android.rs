//! Android 凭证存储实现
//!
//! 使用 tauri-plugin-store 持久化凭证到应用私有目录

use std::collections::HashMap;
use std::sync::RwLock;

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use super::{CredentialStore, CredentialsMap};
use crate::error::{DnsError, Result};

const STORE_FILE_NAME: &str = "credentials.json";
const CREDENTIALS_KEY: &str = "all-credentials";

/// Android 凭证存储实现
///
/// 使用 tauri-plugin-store 将凭证持久化到应用私有目录
/// Android 的应用沙箱提供基本的数据保护
pub struct AndroidCredentialStore {
    app_handle: AppHandle,
    /// 内存缓存，减少磁盘 I/O
    credentials: RwLock<CredentialsMap>,
}

impl AndroidCredentialStore {
    pub fn new(app_handle: AppHandle) -> Self {
        let store = Self {
            app_handle,
            credentials: RwLock::new(HashMap::new()),
        };
        // 启动时从持久化存储加载
        if let Ok(creds) = store.read_from_store() {
            if let Ok(mut cache) = store.credentials.write() {
                *cache = creds;
            }
        }
        store
    }

    /// 从 Store 读取所有凭证
    fn read_from_store(&self) -> Result<CredentialsMap> {
        let store = self
            .app_handle
            .store(STORE_FILE_NAME)
            .map_err(|e| DnsError::CredentialError(format!("Failed to access store: {}", e)))?;

        match store.get(CREDENTIALS_KEY) {
            Some(value) => serde_json::from_value(value.clone())
                .map_err(|e| DnsError::SerializationError(e.to_string())),
            None => Ok(HashMap::new()),
        }
    }

    /// 将所有凭证写入 Store
    fn write_to_store(&self, credentials: &CredentialsMap) -> Result<()> {
        let store = self
            .app_handle
            .store(STORE_FILE_NAME)
            .map_err(|e| DnsError::CredentialError(format!("Failed to access store: {}", e)))?;

        let json = serde_json::to_value(credentials)
            .map_err(|e| DnsError::SerializationError(e.to_string()))?;

        store.set(CREDENTIALS_KEY.to_string(), json);
        store
            .save()
            .map_err(|e| DnsError::CredentialError(format!("Failed to save store: {}", e)))?;

        Ok(())
    }
}

impl CredentialStore for AndroidCredentialStore {
    fn load_all(&self) -> Result<CredentialsMap> {
        log::debug!("Loading all credentials from Android store");
        let credentials = self
            .credentials
            .read()
            .map_err(|e| DnsError::CredentialError(format!("Lock poisoned: {}", e)))?;
        log::info!("Loaded {} accounts from Android store", credentials.len());
        Ok(credentials.clone())
    }

    fn save(&self, account_id: &str, credentials: &HashMap<String, String>) -> Result<()> {
        log::debug!("Saving credentials for account: {}", account_id);

        // 更新内存缓存
        let mut cache = self
            .credentials
            .write()
            .map_err(|e| DnsError::CredentialError(format!("Lock poisoned: {}", e)))?;
        cache.insert(account_id.to_string(), credentials.clone());

        // 持久化到 Store
        self.write_to_store(&cache)?;

        log::info!("Credentials saved for account: {}", account_id);
        Ok(())
    }

    fn load(&self, account_id: &str) -> Result<HashMap<String, String>> {
        let cache = self
            .credentials
            .read()
            .map_err(|e| DnsError::CredentialError(format!("Lock poisoned: {}", e)))?;
        cache.get(account_id).cloned().ok_or_else(|| {
            DnsError::CredentialError(format!("No credentials found for account: {}", account_id))
        })
    }

    fn delete(&self, account_id: &str) -> Result<()> {
        log::debug!("Deleting credentials for account: {}", account_id);

        // 更新内存缓存
        let mut cache = self
            .credentials
            .write()
            .map_err(|e| DnsError::CredentialError(format!("Lock poisoned: {}", e)))?;
        cache.remove(account_id);

        // 持久化到 Store
        self.write_to_store(&cache)?;

        log::info!("Credentials deleted for account: {}", account_id);
        Ok(())
    }

    fn exists(&self, account_id: &str) -> bool {
        self.credentials
            .read()
            .map(|cache| cache.contains_key(account_id))
            .unwrap_or(false)
    }
}
