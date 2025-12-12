//! 阿里云 ACS3-HMAC-SHA256 签名

use sha2::{Digest, Sha256};

use crate::providers::common::hmac_sha256;

use super::{ALIYUN_DNS_HOST, ALIYUN_DNS_VERSION, AliyunProvider, EMPTY_BODY_SHA256};

impl AliyunProvider {
    /// 生成 ACS3-HMAC-SHA256 签名
    /// 参考: <https://www.alibabacloud.com/help/zh/sdk/product-overview/v3-request-structure-and-signature>
    pub(crate) fn sign(
        &self,
        action: &str,
        query_string: &str,
        timestamp: &str,
        nonce: &str,
    ) -> String {
        // 1. 构造规范化请求头 (使用空 body 的 hash)
        let canonical_headers = format!(
            "host:{ALIYUN_DNS_HOST}\nx-acs-action:{action}\nx-acs-content-sha256:{EMPTY_BODY_SHA256}\nx-acs-date:{timestamp}\nx-acs-signature-nonce:{nonce}\nx-acs-version:{ALIYUN_DNS_VERSION}\n"
        );

        let signed_headers =
            "host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version";

        // 2. 构造规范化请求 (RPC 风格: 参数在 query string 中)
        let canonical_request = format!(
            "POST\n/\n{query_string}\n{canonical_headers}\n{signed_headers}\n{EMPTY_BODY_SHA256}"
        );

        log::debug!("CanonicalRequest:\n{canonical_request}");

        // 3. 构造待签名字符串
        let hashed_canonical_request = hex::encode(Sha256::digest(canonical_request.as_bytes()));
        let string_to_sign = format!("ACS3-HMAC-SHA256\n{hashed_canonical_request}");

        log::debug!("StringToSign:\n{string_to_sign}");

        // 4. 计算签名
        let signature = hex::encode(hmac_sha256(
            self.access_key_secret.as_bytes(),
            string_to_sign.as_bytes(),
        ));

        // 5. 构造 Authorization 头
        format!(
            "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
            self.access_key_id, signed_headers, signature
        )
    }
}
