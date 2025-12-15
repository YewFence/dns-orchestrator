//! 通用 HTTP 客户端工具
//!
//! 提供可复用的 HTTP 请求处理逻辑，减少各 Provider 的重复代码。
//! 各 Provider 保留完全的签名灵活性，自己构造 RequestBuilder。
//!
//! # 设计原则
//! - **不强制统一签名逻辑** - 各 provider 的签名算法差异太大
//! - **统一通用的 HTTP 处理流程** - 发送请求、日志记录、读取响应
//! - **灵活的响应解析** - 提供工具函数，但不限制解析方式

use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

use crate::error::ProviderError;

/// HTTP 工具函数集
pub struct HttpUtils;

impl HttpUtils {
    /// 执行 HTTP 请求并返回响应文本
    ///
    /// 统一处理：发送请求、日志记录、错误处理
    ///
    /// # Arguments
    /// * `request_builder` - 已配置好的请求构造器（包含 URL、headers、body 等）
    /// * `provider_name` - Provider 名称（用于日志）
    /// * `method_name` - 请求方法名（如 "GET", "POST"，用于日志）
    /// * `url_or_action` - URL 或 Action 名称（用于日志）
    ///
    /// # Returns
    /// * `Ok((status_code, response_text))` - 成功时返回状态码和响应文本
    /// * `Err(ProviderError::NetworkError)` - 网络错误
    pub async fn execute_request(
        request_builder: RequestBuilder,
        provider_name: &str,
        method_name: &str,
        url_or_action: &str,
    ) -> Result<(u16, String), ProviderError> {
        log::debug!("[{}] {} {}", provider_name, method_name, url_or_action);

        // 发送请求
        let response = request_builder
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError {
                provider: provider_name.to_string(),
                detail: e.to_string(),
            })?;

        let status_code = response.status().as_u16();
        log::debug!("[{}] Response Status: {}", provider_name, status_code);

        // 读取响应体
        let response_text = response
            .text()
            .await
            .map_err(|e| ProviderError::NetworkError {
                provider: provider_name.to_string(),
                detail: format!("读取响应失败: {e}"),
            })?;

        log::debug!("[{}] Response Body: {}", provider_name, response_text);

        Ok((status_code, response_text))
    }

    /// 解析 JSON 响应
    ///
    /// # Type Parameters
    /// * `T` - 目标类型
    ///
    /// # Arguments
    /// * `response_text` - JSON 文本
    /// * `provider_name` - Provider 名称（用于错误消息）
    ///
    /// # Returns
    /// * `Ok(T)` - 成功解析
    /// * `Err(ProviderError::ParseError)` - 解析失败
    pub fn parse_json<T>(response_text: &str, provider_name: &str) -> Result<T, ProviderError>
    where
        T: DeserializeOwned,
    {
        serde_json::from_str(response_text).map_err(|e| {
            log::error!("[{}] JSON 解析失败: {}", provider_name, e);
            log::error!("[{}] 原始响应: {}", provider_name, response_text);
            ProviderError::ParseError {
                provider: provider_name.to_string(),
                detail: e.to_string(),
            }
        })
    }

    /// 组合：执行请求并解析 JSON
    ///
    /// 最常用的场景：发送请求 -> 获取响应 -> 解析 JSON
    pub async fn execute_and_parse_json<T>(
        request_builder: RequestBuilder,
        provider_name: &str,
        method_name: &str,
        url_or_action: &str,
    ) -> Result<T, ProviderError>
    where
        T: DeserializeOwned,
    {
        let (_status, text) =
            Self::execute_request(request_builder, provider_name, method_name, url_or_action)
                .await?;
        Self::parse_json(&text, provider_name)
    }
}
