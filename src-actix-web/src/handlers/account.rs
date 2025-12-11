//! Account 处理模块

use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entity::{account, Account};
use crate::error::ApiError;
use crate::state::AppState;

// ============ 请求参数类型 ============

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountArgs {
    pub request: CreateAccountRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccountRequest {
    pub name: String,
    pub provider_type: String,
    pub credentials: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteAccountArgs {
    pub account_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsArgs {
    pub request: ExportAccountsRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsRequest {
    pub account_ids: Vec<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewImportArgs {
    pub content: String,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAccountsArgs {
    pub request: ImportAccountsRequest,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportAccountsRequest {
    pub content: String,
    pub password: Option<String>,
    pub account_ids: Vec<String>,
}

// ============ 响应类型 ============

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_fields: Vec<ProviderField>,
    pub features: ProviderFeatures,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderField {
    pub key: String,
    pub label: String,
    pub field_type: String,
    pub placeholder: Option<String>,
    pub help_text: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderFeatures {
    pub proxy: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportAccountsResponse {
    pub content: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub accounts: Vec<ImportPreviewAccount>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewAccount {
    pub id: String,
    pub name: String,
    pub provider_type: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub errors: Vec<String>,
}

// ============ Handler 实现 ============

/// 获取所有账户
pub async fn list_accounts(state: &AppState) -> Result<Vec<AccountInfo>, ApiError> {
    let accounts = Account::find().all(&state.db).await?;

    Ok(accounts
        .into_iter()
        .map(|a| AccountInfo {
            id: a.id,
            name: a.name,
            provider_type: a.provider_type,
            created_at: a.created_at.to_rfc3339(),
        })
        .collect())
}

/// 创建账户
pub async fn create_account(
    state: &AppState,
    req: CreateAccountRequest,
) -> Result<AccountInfo, ApiError> {
    // 将凭证转换为 ProviderCredentials
    let credentials = parse_credentials(&req.provider_type, &req.credentials)?;

    // 创建 Provider 并验证凭证
    let provider = dns_orchestrator_provider::create_provider(credentials.clone())?;
    let valid = provider.validate_credentials().await.map_err(|e| {
        ApiError::CredentialValidation(format!("凭证验证失败: {e}"))
    })?;

    if !valid {
        return Err(ApiError::CredentialValidation("凭证无效".to_string()));
    }

    // 加密凭证
    let credentials_json = serde_json::to_string(&credentials)?;
    let encrypted_credentials = state.crypto.encrypt(&credentials_json)?;

    // 保存到数据库
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();

    let account = account::ActiveModel {
        id: Set(id.clone()),
        name: Set(req.name.clone()),
        provider_type: Set(req.provider_type.clone()),
        encrypted_credentials: Set(encrypted_credentials),
        created_at: Set(now),
        updated_at: Set(now),
    };

    account.insert(&state.db).await?;

    // 注册 Provider
    state.registry.register(id.clone(), provider).await;

    Ok(AccountInfo {
        id,
        name: req.name,
        provider_type: req.provider_type,
        created_at: now.to_rfc3339(),
    })
}

/// 删除账户
pub async fn delete_account(state: &AppState, account_id: &str) -> Result<(), ApiError> {
    let result = Account::delete_by_id(account_id).exec(&state.db).await?;

    if result.rows_affected == 0 {
        return Err(ApiError::AccountNotFound(account_id.to_string()));
    }

    // 注销 Provider
    state.registry.unregister(account_id).await;

    Ok(())
}

/// 获取所有支持的 Provider
pub fn list_providers() -> Vec<ProviderInfo> {
    dns_orchestrator_provider::get_all_provider_metadata()
        .into_iter()
        .map(|m| ProviderInfo {
            id: m.id,
            name: m.name,
            description: m.description,
            required_fields: m
                .required_fields
                .into_iter()
                .map(|f| ProviderField {
                    key: f.key,
                    label: f.label,
                    field_type: match f.field_type {
                        dns_orchestrator_provider::FieldType::Text => "text".to_string(),
                        dns_orchestrator_provider::FieldType::Password => "password".to_string(),
                    },
                    placeholder: f.placeholder,
                    help_text: f.help_text,
                })
                .collect(),
            features: ProviderFeatures {
                proxy: m.features.proxy,
            },
        })
        .collect()
}

/// 导出账户
pub async fn export_accounts(
    state: &AppState,
    req: ExportAccountsRequest,
) -> Result<ExportAccountsResponse, ApiError> {
    let mut export_data = Vec::new();

    for account_id in &req.account_ids {
        let account = Account::find_by_id(account_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| ApiError::AccountNotFound(account_id.clone()))?;

        // 解密凭证
        let credentials_json = state.crypto.decrypt(&account.encrypted_credentials)?;

        export_data.push(serde_json::json!({
            "id": account.id,
            "name": account.name,
            "providerType": account.provider_type,
            "credentials": serde_json::from_str::<serde_json::Value>(&credentials_json)?,
        }));
    }

    let content = serde_json::to_string(&export_data)?;

    // 如果提供了密码，加密内容
    let final_content = if let Some(password) = &req.password {
        encrypt_with_password(&content, password)?
    } else {
        content
    };

    Ok(ExportAccountsResponse {
        content: final_content,
        count: export_data.len(),
    })
}

/// 预览导入
pub fn preview_import(content: &str, password: Option<&str>) -> Result<ImportPreview, ApiError> {
    // 如果提供了密码，解密内容
    let decrypted = if let Some(pwd) = password {
        decrypt_with_password(content, pwd)?
    } else {
        content.to_string()
    };

    let accounts: Vec<serde_json::Value> = serde_json::from_str(&decrypted)
        .map_err(|e| ApiError::BadRequest(format!("无效的导入数据: {e}")))?;

    let preview_accounts: Vec<ImportPreviewAccount> = accounts
        .iter()
        .filter_map(|a| {
            Some(ImportPreviewAccount {
                id: a.get("id")?.as_str()?.to_string(),
                name: a.get("name")?.as_str()?.to_string(),
                provider_type: a.get("providerType")?.as_str()?.to_string(),
            })
        })
        .collect();

    Ok(ImportPreview {
        total: preview_accounts.len(),
        accounts: preview_accounts,
    })
}

/// 导入账户
pub async fn import_accounts(
    state: &AppState,
    req: ImportAccountsRequest,
) -> Result<ImportResult, ApiError> {
    // 解密内容
    let decrypted = if let Some(pwd) = &req.password {
        decrypt_with_password(&req.content, pwd)?
    } else {
        req.content.clone()
    };

    let accounts: Vec<serde_json::Value> = serde_json::from_str(&decrypted)
        .map_err(|e| ApiError::BadRequest(format!("无效的导入数据: {e}")))?;

    let mut imported = 0;
    let mut skipped = 0;
    let mut errors = Vec::new();

    for account_data in accounts {
        let id = account_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        // 检查是否在选择列表中
        if !req.account_ids.contains(&id.to_string()) {
            skipped += 1;
            continue;
        }

        // 提取数据
        let name = account_data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let provider_type = account_data
            .get("providerType")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let credentials = account_data
            .get("credentials")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        // 创建账户
        match create_account(
            state,
            CreateAccountRequest {
                name: name.clone(),
                provider_type,
                credentials,
            },
        )
        .await
        {
            Ok(_) => imported += 1,
            Err(e) => errors.push(format!("导入 {} 失败: {}", name, e)),
        }
    }

    Ok(ImportResult {
        imported,
        skipped,
        errors,
    })
}

// ============ 辅助函数 ============

/// 解析凭证
fn parse_credentials(
    provider_type: &str,
    credentials: &serde_json::Value,
) -> Result<dns_orchestrator_provider::ProviderCredentials, ApiError> {
    use dns_orchestrator_provider::ProviderCredentials;

    match provider_type {
        "cloudflare" => {
            let api_token = credentials
                .get("apiToken")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::BadRequest("缺少 apiToken".to_string()))?
                .to_string();
            Ok(ProviderCredentials::Cloudflare { api_token })
        }
        "aliyun" => {
            let access_key_id = credentials
                .get("accessKeyId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::BadRequest("缺少 accessKeyId".to_string()))?
                .to_string();
            let access_key_secret = credentials
                .get("accessKeySecret")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::BadRequest("缺少 accessKeySecret".to_string()))?
                .to_string();
            Ok(ProviderCredentials::Aliyun {
                access_key_id,
                access_key_secret,
            })
        }
        "dnspod" => {
            let secret_id = credentials
                .get("secretId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::BadRequest("缺少 secretId".to_string()))?
                .to_string();
            let secret_key = credentials
                .get("secretKey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::BadRequest("缺少 secretKey".to_string()))?
                .to_string();
            Ok(ProviderCredentials::Dnspod {
                secret_id,
                secret_key,
            })
        }
        "huaweicloud" => {
            let access_key_id = credentials
                .get("accessKeyId")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::BadRequest("缺少 accessKeyId".to_string()))?
                .to_string();
            let secret_access_key = credentials
                .get("secretAccessKey")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ApiError::BadRequest("缺少 secretAccessKey".to_string()))?
                .to_string();
            Ok(ProviderCredentials::Huaweicloud {
                access_key_id,
                secret_access_key,
            })
        }
        _ => Err(ApiError::ProviderNotFound(provider_type.to_string())),
    }
}

/// 使用密码加密
fn encrypt_with_password(data: &str, password: &str) -> Result<String, ApiError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use pbkdf2::pbkdf2_hmac_array;
    use rand::RngCore;
    use sha2::Sha256;

    // 生成随机 salt
    let mut salt = [0u8; 16];
    rand::rng().fill_bytes(&mut salt);

    // 派生密钥
    let key = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), &salt, 100_000);

    // 生成随机 nonce
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 加密
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ApiError::Encryption(format!("创建加密器失败: {e}")))?;
    let ciphertext = cipher
        .encrypt(nonce, data.as_bytes())
        .map_err(|e| ApiError::Encryption(format!("加密失败: {e}")))?;

    // 组合: salt || nonce || ciphertext
    let mut combined = Vec::with_capacity(16 + 12 + ciphertext.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(combined))
}

/// 使用密码解密
fn decrypt_with_password(data: &str, password: &str) -> Result<String, ApiError> {
    use aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    };
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use pbkdf2::pbkdf2_hmac_array;
    use sha2::Sha256;

    let combined = BASE64
        .decode(data)
        .map_err(|e| ApiError::Encryption(format!("Base64 解码失败: {e}")))?;

    if combined.len() < 28 {
        return Err(ApiError::Encryption("加密数据格式无效".to_string()));
    }

    let salt = &combined[..16];
    let nonce_bytes = &combined[16..28];
    let ciphertext = &combined[28..];

    // 派生密钥
    let key = pbkdf2_hmac_array::<Sha256, 32>(password.as_bytes(), salt, 100_000);

    // 解密
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ApiError::Encryption(format!("创建解密器失败: {e}")))?;
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| ApiError::Encryption("解密失败，密码可能错误".to_string()))?;

    String::from_utf8(plaintext).map_err(|e| ApiError::Encryption(format!("UTF-8 解码失败: {e}")))
}
