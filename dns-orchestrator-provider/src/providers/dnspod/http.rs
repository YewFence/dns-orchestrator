//! DNSPod HTTP 请求方法

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{ProviderError, Result};
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::{DNSPOD_API_HOST, DNSPOD_VERSION, DnspodProvider, TencentResponse};

impl DnspodProvider {
    /// 执行腾讯云 API 请求
    pub(crate) async fn request<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        action: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        let payload =
            serde_json::to_string(body).map_err(|e| ProviderError::SerializationError {
                provider: self.provider_name().to_string(),
                detail: e.to_string(),
            })?;

        let timestamp = Utc::now().timestamp();
        let authorization = self.sign(action, &payload, timestamp);

        let url = format!("https://{DNSPOD_API_HOST}");
        log::debug!("POST {url} Action: {action}");
        log::debug!("Request Body: {payload}");

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json; charset=utf-8")
            .header("Host", DNSPOD_API_HOST)
            .header("X-TC-Action", action)
            .header("X-TC-Version", DNSPOD_VERSION)
            .header("X-TC-Timestamp", timestamp.to_string())
            .header("Authorization", authorization)
            .body(payload)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        log::debug!("Response Status: {status}");

        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Body: {response_text}");

        let tc_response: TencentResponse<T> =
            serde_json::from_str(&response_text).map_err(|e| {
                log::error!("JSON 解析失败: {e}");
                log::error!("原始响应: {response_text}");
                self.parse_error(e)
            })?;

        if let Some(error) = tc_response.response.error {
            log::error!("API 错误: {} - {}", error.code, error.message);
            return Err(self.map_error(
                RawApiError::with_code(&error.code, &error.message),
                ctx,
            ));
        }

        tc_response
            .response
            .data
            .ok_or_else(|| self.parse_error("响应中缺少数据"))
    }
}
