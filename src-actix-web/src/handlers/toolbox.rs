//! Toolbox 网络工具处理模块

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioResolver;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

use crate::error::ApiError;

// ============ 请求参数类型 ============

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoisLookupArgs {
    pub domain: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupArgs {
    pub domain: String,
    pub record_type: String,
    pub nameserver: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IpLookupArgs {
    pub query: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SslCheckArgs {
    pub domain: String,
    pub port: Option<u16>,
}

// ============ 响应类型 ============

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoisResult {
    pub raw: String,
    pub registrar: Option<String>,
    pub creation_date: Option<String>,
    pub expiration_date: Option<String>,
    pub name_servers: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupResult {
    pub records: Vec<DnsLookupRecord>,
    pub query_time_ms: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsLookupRecord {
    pub name: String,
    pub record_type: String,
    pub ttl: u32,
    pub value: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IpLookupResult {
    pub ip: String,
    pub hostname: Option<String>,
    pub is_valid: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SslCheckResult {
    pub valid: bool,
    pub issuer: Option<String>,
    pub subject: Option<String>,
    pub not_before: Option<String>,
    pub not_after: Option<String>,
    pub days_remaining: Option<i64>,
    pub error: Option<String>,
}

// ============ Handler 实现 ============

/// WHOIS 查询
pub async fn whois_lookup(domain: &str) -> Result<WhoisResult, ApiError> {
    let whois = whois_rust::WhoIs::from_string(include_str!(
        "../../../resources/whois_servers.json"
    ))
    .map_err(|e| ApiError::Internal(format!("加载 WHOIS 服务器失败: {e}")))?;

    let result = whois
        .lookup_async(whois_rust::WhoIsLookupOptions::from_domain(domain).map_err(|e| {
            ApiError::BadRequest(format!("无效的域名: {e}"))
        })?)
        .await
        .map_err(|e| ApiError::Internal(format!("WHOIS 查询失败: {e}")))?;

    // 解析 WHOIS 响应
    let raw = result.clone();
    let registrar = extract_whois_field(&raw, &["Registrar:", "registrar:"]);
    let creation_date = extract_whois_field(&raw, &["Creation Date:", "created:", "Created Date:"]);
    let expiration_date = extract_whois_field(
        &raw,
        &[
            "Registry Expiry Date:",
            "Expiration Date:",
            "expires:",
            "Expiry Date:",
        ],
    );
    let name_servers = extract_whois_list(&raw, &["Name Server:", "nserver:", "NS:"]);

    Ok(WhoisResult {
        raw,
        registrar,
        creation_date,
        expiration_date,
        name_servers,
    })
}

/// DNS 查询
pub async fn dns_lookup(
    domain: &str,
    record_type: &str,
    nameserver: Option<&str>,
) -> Result<DnsLookupResult, ApiError> {
    let start = std::time::Instant::now();

    let resolver = if let Some(ns) = nameserver {
        // 使用指定的 DNS 服务器
        let ns_ip: IpAddr = ns
            .parse()
            .map_err(|_| ApiError::BadRequest(format!("无效的 DNS 服务器地址: {ns}")))?;

        let mut config = ResolverConfig::new();
        config.add_name_server(hickory_resolver::config::NameServerConfig {
            socket_addr: std::net::SocketAddr::new(ns_ip, 53),
            protocol: hickory_resolver::config::Protocol::Udp,
            tls_dns_name: None,
            trust_negative_responses: true,
            tls_config: None,
            bind_addr: None,
        });

        TokioResolver::tokio(config, ResolverOpts::default())
    } else {
        TokioResolver::tokio_from_system_conf()
            .map_err(|e| ApiError::Internal(format!("无法创建 DNS 解析器: {e}")))?
    };

    let records = match record_type.to_uppercase().as_str() {
        "A" => lookup_a(&resolver, domain).await?,
        "AAAA" => lookup_aaaa(&resolver, domain).await?,
        "CNAME" => lookup_cname(&resolver, domain).await?,
        "MX" => lookup_mx(&resolver, domain).await?,
        "TXT" => lookup_txt(&resolver, domain).await?,
        "NS" => lookup_ns(&resolver, domain).await?,
        "SOA" => lookup_soa(&resolver, domain).await?,
        "ALL" => {
            let mut all = Vec::new();
            if let Ok(r) = lookup_a(&resolver, domain).await {
                all.extend(r);
            }
            if let Ok(r) = lookup_aaaa(&resolver, domain).await {
                all.extend(r);
            }
            if let Ok(r) = lookup_cname(&resolver, domain).await {
                all.extend(r);
            }
            if let Ok(r) = lookup_mx(&resolver, domain).await {
                all.extend(r);
            }
            if let Ok(r) = lookup_txt(&resolver, domain).await {
                all.extend(r);
            }
            if let Ok(r) = lookup_ns(&resolver, domain).await {
                all.extend(r);
            }
            all
        }
        _ => return Err(ApiError::BadRequest(format!("不支持的记录类型: {record_type}"))),
    };

    let query_time_ms = start.elapsed().as_millis() as u64;

    Ok(DnsLookupResult {
        records,
        query_time_ms,
    })
}

/// IP 查询（反向 DNS）
pub async fn ip_lookup(query: &str) -> Result<IpLookupResult, ApiError> {
    let ip: IpAddr = query
        .parse()
        .map_err(|_| ApiError::BadRequest(format!("无效的 IP 地址: {query}")))?;

    let resolver = TokioResolver::tokio_from_system_conf()
        .map_err(|e| ApiError::Internal(format!("无法创建 DNS 解析器: {e}")))?;

    let hostname = resolver
        .reverse_lookup(ip)
        .await
        .ok()
        .and_then(|lookup| lookup.iter().next().map(|name| name.to_string()));

    Ok(IpLookupResult {
        ip: ip.to_string(),
        hostname,
        is_valid: true,
    })
}

/// SSL 证书检查
pub async fn ssl_check(domain: &str, port: Option<u16>) -> Result<SslCheckResult, ApiError> {
    use rustls::pki_types::ServerName;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use x509_parser::prelude::*;

    let port = port.unwrap_or(443);
    let addr = format!("{domain}:{port}");

    // 创建 TLS 配置
    let root_store =
        rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));

    // 连接
    let stream = match tokio::time::timeout(
        Duration::from_secs(10),
        TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            return Ok(SslCheckResult {
                valid: false,
                issuer: None,
                subject: None,
                not_before: None,
                not_after: None,
                days_remaining: None,
                error: Some(format!("连接失败: {e}")),
            });
        }
        Err(_) => {
            return Ok(SslCheckResult {
                valid: false,
                issuer: None,
                subject: None,
                not_before: None,
                not_after: None,
                days_remaining: None,
                error: Some("连接超时".to_string()),
            });
        }
    };

    let server_name = ServerName::try_from(domain.to_string())
        .map_err(|_| ApiError::BadRequest(format!("无效的域名: {domain}")))?;

    let tls_stream = match connector.connect(server_name, stream).await {
        Ok(s) => s,
        Err(e) => {
            return Ok(SslCheckResult {
                valid: false,
                issuer: None,
                subject: None,
                not_before: None,
                not_after: None,
                days_remaining: None,
                error: Some(format!("TLS 握手失败: {e}")),
            });
        }
    };

    // 获取证书
    let certs = tls_stream
        .get_ref()
        .1
        .peer_certificates()
        .ok_or_else(|| ApiError::Internal("无法获取证书".to_string()))?;

    if certs.is_empty() {
        return Ok(SslCheckResult {
            valid: false,
            issuer: None,
            subject: None,
            not_before: None,
            not_after: None,
            days_remaining: None,
            error: Some("服务器未返回证书".to_string()),
        });
    }

    // 解析证书
    let (_, cert) = X509Certificate::from_der(&certs[0])
        .map_err(|e| ApiError::Internal(format!("证书解析失败: {e}")))?;

    let issuer = cert.issuer().to_string();
    let subject = cert.subject().to_string();
    let not_before = cert.validity().not_before.to_rfc2822();
    let not_after = cert.validity().not_after.to_rfc2822();

    let now = chrono::Utc::now();
    let expiry = chrono::DateTime::from_timestamp(cert.validity().not_after.timestamp(), 0)
        .unwrap_or(now);
    let days_remaining = (expiry - now).num_days();

    Ok(SslCheckResult {
        valid: days_remaining > 0,
        issuer: Some(issuer),
        subject: Some(subject),
        not_before: Some(not_before),
        not_after: Some(not_after),
        days_remaining: Some(days_remaining),
        error: None,
    })
}

// ============ 辅助函数 ============

fn extract_whois_field(raw: &str, keys: &[&str]) -> Option<String> {
    for line in raw.lines() {
        for key in keys {
            if line.trim().to_lowercase().starts_with(&key.to_lowercase()) {
                let value = line.trim()[key.len()..].trim();
                if !value.is_empty() {
                    return Some(value.to_string());
                }
            }
        }
    }
    None
}

fn extract_whois_list(raw: &str, keys: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    for line in raw.lines() {
        for key in keys {
            if line.trim().to_lowercase().starts_with(&key.to_lowercase()) {
                let value = line.trim()[key.len()..].trim();
                if !value.is_empty() {
                    results.push(value.to_string());
                }
            }
        }
    }
    results
}

async fn lookup_a(resolver: &TokioResolver, domain: &str) -> Result<Vec<DnsLookupRecord>, ApiError> {
    let lookup = resolver
        .ipv4_lookup(domain)
        .await
        .map_err(|e| ApiError::Internal(format!("A 记录查询失败: {e}")))?;

    Ok(lookup
        .iter()
        .map(|ip| DnsLookupRecord {
            name: domain.to_string(),
            record_type: "A".to_string(),
            ttl: lookup.query().queries().first().map_or(0, |q| 300),
            value: ip.to_string(),
        })
        .collect())
}

async fn lookup_aaaa(
    resolver: &TokioResolver,
    domain: &str,
) -> Result<Vec<DnsLookupRecord>, ApiError> {
    let lookup = resolver
        .ipv6_lookup(domain)
        .await
        .map_err(|e| ApiError::Internal(format!("AAAA 记录查询失败: {e}")))?;

    Ok(lookup
        .iter()
        .map(|ip| DnsLookupRecord {
            name: domain.to_string(),
            record_type: "AAAA".to_string(),
            ttl: 300,
            value: ip.to_string(),
        })
        .collect())
}

async fn lookup_cname(
    resolver: &TokioResolver,
    domain: &str,
) -> Result<Vec<DnsLookupRecord>, ApiError> {
    let lookup = resolver
        .lookup(domain, hickory_resolver::proto::rr::RecordType::CNAME)
        .await
        .map_err(|e| ApiError::Internal(format!("CNAME 记录查询失败: {e}")))?;

    Ok(lookup
        .iter()
        .filter_map(|r| r.as_cname())
        .map(|cname| DnsLookupRecord {
            name: domain.to_string(),
            record_type: "CNAME".to_string(),
            ttl: 300,
            value: cname.to_string(),
        })
        .collect())
}

async fn lookup_mx(resolver: &TokioResolver, domain: &str) -> Result<Vec<DnsLookupRecord>, ApiError> {
    let lookup = resolver
        .mx_lookup(domain)
        .await
        .map_err(|e| ApiError::Internal(format!("MX 记录查询失败: {e}")))?;

    Ok(lookup
        .iter()
        .map(|mx| DnsLookupRecord {
            name: domain.to_string(),
            record_type: "MX".to_string(),
            ttl: 300,
            value: format!("{} {}", mx.preference(), mx.exchange()),
        })
        .collect())
}

async fn lookup_txt(
    resolver: &TokioResolver,
    domain: &str,
) -> Result<Vec<DnsLookupRecord>, ApiError> {
    let lookup = resolver
        .txt_lookup(domain)
        .await
        .map_err(|e| ApiError::Internal(format!("TXT 记录查询失败: {e}")))?;

    Ok(lookup
        .iter()
        .map(|txt| DnsLookupRecord {
            name: domain.to_string(),
            record_type: "TXT".to_string(),
            ttl: 300,
            value: txt.to_string(),
        })
        .collect())
}

async fn lookup_ns(resolver: &TokioResolver, domain: &str) -> Result<Vec<DnsLookupRecord>, ApiError> {
    let lookup = resolver
        .ns_lookup(domain)
        .await
        .map_err(|e| ApiError::Internal(format!("NS 记录查询失败: {e}")))?;

    Ok(lookup
        .iter()
        .map(|ns| DnsLookupRecord {
            name: domain.to_string(),
            record_type: "NS".to_string(),
            ttl: 300,
            value: ns.to_string(),
        })
        .collect())
}

async fn lookup_soa(
    resolver: &TokioResolver,
    domain: &str,
) -> Result<Vec<DnsLookupRecord>, ApiError> {
    let lookup = resolver
        .soa_lookup(domain)
        .await
        .map_err(|e| ApiError::Internal(format!("SOA 记录查询失败: {e}")))?;

    Ok(lookup
        .iter()
        .map(|soa| DnsLookupRecord {
            name: domain.to_string(),
            record_type: "SOA".to_string(),
            ttl: 300,
            value: format!(
                "{} {} {} {} {} {} {}",
                soa.mname(),
                soa.rname(),
                soa.serial(),
                soa.refresh(),
                soa.retry(),
                soa.expire(),
                soa.minimum()
            ),
        })
        .collect())
}
