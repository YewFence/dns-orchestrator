//! 华为云 HTTP 请求方法

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{ProviderError, Result};
use crate::traits::{ErrorContext, ProviderErrorMapper, RawApiError};

use super::types::ErrorResponse;
use super::{HUAWEICLOUD_DNS_HOST, HuaweicloudProvider};

impl HuaweicloudProvider {
    /// 执行 GET 请求
    pub(crate) async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        path: &str,
        query: &str,
        ctx: ErrorContext,
    ) -> Result<T> {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
        ];

        let authorization = self.sign("GET", path, query, &headers, "", &timestamp);

        let url = if query.is_empty() {
            format!("https://{HUAWEICLOUD_DNS_HOST}{path}")
        } else {
            format!("https://{HUAWEICLOUD_DNS_HOST}{path}?{query}")
        };

        log::debug!("GET {url}");

        let response = self
            .client
            .get(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Authorization", authorization)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Status: {status}, Body: {response_text}");

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self.map_error(
                    RawApiError::with_code(
                        error.error_code.unwrap_or_default(),
                        error.error_msg.unwrap_or_default(),
                    ),
                    ctx,
                ));
            }
            return Err(
                self.unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
            );
        }

        serde_json::from_str(&response_text).map_err(|e| {
            log::error!("JSON 解析失败: {e}");
            self.parse_error(e)
        })
    }

    /// 执行 POST 请求
    pub(crate) async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        let payload =
            serde_json::to_string(body).map_err(|e| ProviderError::SerializationError {
                provider: self.provider_name().to_string(),
                detail: e.to_string(),
            })?;

        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let authorization = self.sign("POST", path, "", &headers, &payload, &timestamp);

        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");
        log::debug!("POST {url} Body: {payload}");

        let response = self
            .client
            .post(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Content-Type", "application/json")
            .header("Authorization", authorization)
            .body(payload)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Status: {status}, Body: {response_text}");

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self.map_error(
                    RawApiError::with_code(
                        error.error_code.unwrap_or_default(),
                        error.error_msg.unwrap_or_default(),
                    ),
                    ctx,
                ));
            }
            return Err(
                self.unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
            );
        }

        serde_json::from_str(&response_text).map_err(|e| {
            log::error!("JSON 解析失败: {e}");
            self.parse_error(e)
        })
    }

    /// 执行 PUT 请求
    pub(crate) async fn put<T: for<'de> Deserialize<'de>, B: Serialize>(
        &self,
        path: &str,
        body: &B,
        ctx: ErrorContext,
    ) -> Result<T> {
        let payload =
            serde_json::to_string(body).map_err(|e| ProviderError::SerializationError {
                provider: self.provider_name().to_string(),
                detail: e.to_string(),
            })?;

        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let authorization = self.sign("PUT", path, "", &headers, &payload, &timestamp);

        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");
        log::debug!("PUT {url} Body: {payload}");

        let response = self
            .client
            .put(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Content-Type", "application/json")
            .header("Authorization", authorization)
            .body(payload)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

        log::debug!("Response Status: {status}, Body: {response_text}");

        if !status.is_success() {
            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self.map_error(
                    RawApiError::with_code(
                        error.error_code.unwrap_or_default(),
                        error.error_msg.unwrap_or_default(),
                    ),
                    ctx,
                ));
            }
            return Err(
                self.unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
            );
        }

        serde_json::from_str(&response_text).map_err(|e| {
            log::error!("JSON 解析失败: {e}");
            self.parse_error(e)
        })
    }

    /// 执行 DELETE 请求
    pub(crate) async fn delete(&self, path: &str, ctx: ErrorContext) -> Result<()> {
        let now = Utc::now();
        let timestamp = now.format("%Y%m%dT%H%M%SZ").to_string();

        let headers = vec![
            ("Host".to_string(), HUAWEICLOUD_DNS_HOST.to_string()),
            ("X-Sdk-Date".to_string(), timestamp.clone()),
        ];

        let authorization = self.sign("DELETE", path, "", &headers, "", &timestamp);

        let url = format!("https://{HUAWEICLOUD_DNS_HOST}{path}");
        log::debug!("DELETE {url}");

        let response = self
            .client
            .delete(&url)
            .header("Host", HUAWEICLOUD_DNS_HOST)
            .header("X-Sdk-Date", &timestamp)
            .header("Authorization", authorization)
            .send()
            .await
            .map_err(|e| self.network_error(e))?;

        let status = response.status();

        if !status.is_success() {
            let response_text = response
                .text()
                .await
                .map_err(|e| self.network_error(format!("读取响应失败: {e}")))?;

            if let Ok(error) = serde_json::from_str::<ErrorResponse>(&response_text) {
                return Err(self.map_error(
                    RawApiError::with_code(
                        error.error_code.unwrap_or_default(),
                        error.error_msg.unwrap_or_default(),
                    ),
                    ctx,
                ));
            }
            return Err(
                self.unknown_error(RawApiError::new(format!("HTTP {status}: {response_text}")))
            );
        }

        Ok(())
    }
}
