# Architecture Documentation

This document provides an in-depth look at the architectural design of DNS Orchestrator, explaining the key components, design patterns, and technical decisions.

## Table of Contents

- [Overview](#overview)
- [Architecture Diagram](#architecture-diagram)
- [Project Structure](#project-structure)
- [Frontend Architecture](#frontend-architecture)
- [Backend Architecture](#backend-architecture)
- [Provider Library](#provider-library)
- [Security Architecture](#security-architecture)
- [Performance Optimizations](#performance-optimizations)
- [Data Flow](#data-flow)
- [Design Decisions](#design-decisions)

## Overview

DNS Orchestrator is a cross-platform application built with a clear separation between frontend, backend, and DNS provider logic:

- **Frontend**: React-based UI with TypeScript, Tailwind CSS, and Zustand for state management
- **Backend**: Rust-based Tauri commands for business logic (desktop/mobile), with actix-web backend for web
- **Provider Library**: Standalone `dns-orchestrator-provider` crate for DNS provider integrations
- **Communication**: Transport abstraction layer supports both Tauri IPC and HTTP
- **Security**: System keychain integration ensures secure credential storage

### Technology Choices

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **UI Framework** | React 19 + TypeScript | Strong ecosystem, type safety, component reusability |
| **State Management** | Zustand | Lightweight, no boilerplate, simple API |
| **Styling** | Tailwind CSS 4 | Utility-first, rapid development, consistent design |
| **Desktop Framework** | Tauri 2 | Smaller bundle size than Electron, Rust security benefits |
| **Web Backend** | actix-web | High performance, async, production-ready |
| **Provider Library** | Standalone Rust crate | Reusable across Tauri and web backends |
| **HTTP Client** | Reqwest | Industry standard, async, TLS support |
| **Credential Storage** | keyring / Stronghold | Cross-platform system keychain integration |
| **Build Tool** | Vite 7 | Fast HMR, optimized production builds |

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         USER INTERFACE                               │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │  React Components (src/components/)                           │  │
│  │  - AccountList, DnsRecordTable, DomainList, Toolbox           │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Zustand Stores (src/stores/)                                 │  │
│  │  - accountStore, dnsStore, domainStore, toolboxStore          │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Service Layer (src/services/)                                │  │
│  │  - accountService, dnsService, domainService, toolboxService  │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
│                         │                                            │
│  ┌──────────────────────▼────────────────────────────────────────┐  │
│  │  Transport Abstraction (src/services/transport/)              │  │
│  │  - ITransport interface                                       │  │
│  │  - TauriTransport (Tauri IPC) | HttpTransport (REST API)      │  │
│  └──────────────────────┬────────────────────────────────────────┘  │
└─────────────────────────┼────────────────────────────────────────────┘
                          │
        ┌─────────────────┴─────────────────┐
        │                                   │
        ▼ Tauri IPC                         ▼ HTTP REST
┌───────────────────────────┐    ┌───────────────────────────┐
│   TAURI BACKEND           │    │   ACTIX-WEB BACKEND       │
│   (src-tauri/)            │    │   (src-actix-web/)        │
│                           │    │                           │
│  ┌─────────────────────┐  │    │  ┌─────────────────────┐  │
│  │  Commands Layer     │  │    │  │  HTTP Handlers      │  │
│  │  - account.rs       │  │    │  │  (REST endpoints)   │  │
│  │  - dns.rs           │  │    │  │                     │  │
│  │  - domain.rs        │  │    │  └──────────┬──────────┘  │
│  │  - toolbox.rs       │  │    │             │             │
│  └──────────┬──────────┘  │    │  ┌──────────▼──────────┐  │
│             │             │    │  │  SeaORM Database    │  │
│  ┌──────────▼──────────┐  │    │  │  (MySQL/PG/SQLite)  │  │
│  │  AppState           │  │    │  └─────────────────────┘  │
│  │  - ProviderRegistry │  │    │                           │
│  │  - CredentialStore  │  │    └───────────┬───────────────┘
│  └──────────┬──────────┘  │                │
│             │             │                │
└─────────────┼─────────────┘                │
              │                              │
              └──────────────┬───────────────┘
                             │
              ┌──────────────▼───────────────┐
              │  DNS PROVIDER LIBRARY        │
              │  (dns-orchestrator-provider) │
              │                              │
              │  ┌────────────────────────┐  │
              │  │  DnsProvider Trait     │  │
              │  │  - list_domains()      │  │
              │  │  - list_records()      │  │
              │  │  - create/update/del   │  │
              │  └───────────┬────────────┘  │
              │              │               │
              │  ┌───────────▼────────────┐  │
              │  │  Provider Impls        │  │
              │  │  - CloudflareProvider  │  │
              │  │  - AliyunProvider      │  │
              │  │  - DnspodProvider      │  │
              │  │  - HuaweicloudProvider │  │
              │  └───────────┬────────────┘  │
              └──────────────┼───────────────┘
                             │ HTTPS
              ┌──────────────▼───────────────┐
              │       EXTERNAL DNS APIS       │
              │  Cloudflare | Aliyun | DNSPod │
              │  Huawei Cloud                 │
              └───────────────────────────────┘
```

## Project Structure

```
dns-orchestrator/
├── src/                              # Frontend (React + TypeScript)
│   ├── components/                   # React components by feature
│   ├── services/                     # Service layer (NEW)
│   │   ├── transport/                # Transport abstraction
│   │   │   ├── types.ts              # ITransport interface, CommandMap
│   │   │   ├── tauri.transport.ts    # Tauri IPC implementation
│   │   │   └── http.transport.ts     # HTTP REST implementation
│   │   ├── account.service.ts
│   │   ├── dns.service.ts
│   │   ├── domain.service.ts
│   │   └── toolbox.service.ts
│   ├── stores/                       # Zustand state management
│   ├── types/                        # TypeScript type definitions
│   └── i18n/                         # Internationalization
│
├── dns-orchestrator-provider/        # Standalone Provider Library (NEW)
│   ├── src/
│   │   ├── lib.rs                    # Library entry, re-exports
│   │   ├── traits.rs                 # DnsProvider trait definition
│   │   ├── types.rs                  # Shared types (Domain, DnsRecord, etc.)
│   │   ├── error.rs                  # ProviderError enum
│   │   ├── factory.rs                # create_provider(), metadata
│   │   └── providers/                # Provider implementations
│   │       ├── cloudflare.rs
│   │       ├── aliyun.rs
│   │       ├── dnspod.rs
│   │       └── huaweicloud.rs
│   └── Cargo.toml                    # Feature flags for TLS backend
│
├── src-tauri/                        # Tauri Backend (Desktop/Mobile)
│   ├── src/
│   │   ├── commands/                 # Tauri command handlers
│   │   ├── providers/mod.rs          # ProviderRegistry only (simplified)
│   │   ├── credentials/              # Keychain/Stronghold storage
│   │   ├── storage/                  # Local data persistence
│   │   └── error.rs                  # Re-exports library errors
│   └── Cargo.toml                    # Platform-specific dependencies
│
├── src-actix-web/                    # Web Backend (NEW, WIP)
│   ├── src/main.rs                   # Actix-web server entry
│   └── migration/                    # SeaORM database migrations
│
└── vite.config.ts                    # Platform-aware build config
```

## Frontend Architecture

### Service Layer (New in v1.1.0)

The service layer abstracts backend communication:

```typescript
// src/services/transport/types.ts
export interface ITransport {
  invoke<K extends NoArgsCommands>(command: K): Promise<CommandMap[K]["result"]>
  invoke<K extends WithArgsCommands>(
    command: K,
    args: CommandMap[K]["args"]
  ): Promise<CommandMap[K]["result"]>
}

// CommandMap provides type-safe command definitions
export interface CommandMap {
  list_accounts: { args: Record<string, never>; result: ApiResponse<Account[]> }
  create_account: { args: { request: CreateAccountRequest }; result: ApiResponse<Account> }
  // ... all commands with full type safety
}
```

**Transport Implementations**:

```typescript
// src/services/transport/tauri.transport.ts (Desktop/Mobile)
export class TauriTransport implements ITransport {
  async invoke(command, args?) {
    return await tauriInvoke(command, args)
  }
}

// src/services/transport/http.transport.ts (Web)
export class HttpTransport implements ITransport {
  async invoke(command, args?) {
    return await fetch(`/api/${command}`, { method: 'POST', body: JSON.stringify(args) })
  }
}
```

**Build-time Transport Selection**:

```typescript
// vite.config.ts
resolve: {
  alias: {
    "#transport-impl": platform === "web"
      ? "./src/services/transport/http.transport.ts"
      : "./src/services/transport/tauri.transport.ts",
  },
}
```

### Component Structure

Components are organized by feature domain:

```
src/components/
├── account/              # Account management
├── dns/                  # DNS record management
├── domain/               # Domain management
├── domains/              # Domain selector page
├── home/                 # Home dashboard
├── toolbox/              # Network utilities
├── settings/             # Application settings
├── layout/               # Layout components (sidebar, header)
├── navigation/           # Navigation components
├── error/                # Error boundary components
└── ui/                   # Reusable UI components (Radix wrappers)
```

### State Management (Zustand)

Each feature domain has its own store:

```typescript
// src/stores/accountStore.ts
interface AccountStore {
  accounts: Account[]
  currentAccount: Account | null
  fetchAccounts: () => Promise<void>
  // ...
}

// src/stores/dnsStore.ts
interface DnsStore {
  records: DnsRecord[]
  currentPage: number
  totalPages: number
  searchQuery: string
  filterType: RecordType | 'ALL'
  // ...
}
```

## Backend Architecture

### Tauri Application State

```rust
// src-tauri/src/lib.rs
pub struct AppState {
    pub registry: ProviderRegistry,                    // Provider instances
    pub credential_store: Arc<dyn CredentialStore>,    // System keychain
    pub accounts: RwLock<Vec<Account>>,                // Account metadata
    pub app_handle: tauri::AppHandle,                  // Tauri handle
}
```

### Provider Registry (Simplified)

The `ProviderRegistry` now only manages provider instances, delegating creation to the library:

```rust
// src-tauri/src/providers/mod.rs
pub use dns_orchestrator_provider::{create_provider, get_all_provider_metadata, DnsProvider};

pub struct ProviderRegistry {
    providers: Arc<RwLock<HashMap<String, Arc<dyn DnsProvider>>>>,
}

impl ProviderRegistry {
    pub async fn register(&self, account_id: String, provider: Arc<dyn DnsProvider>);
    pub async fn unregister(&self, account_id: &str);
    pub async fn get(&self, account_id: &str) -> Option<Arc<dyn DnsProvider>>;
}
```

### Platform-Specific Dependencies

```toml
# src-tauri/Cargo.toml

# Desktop (macOS, Windows, Linux)
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
dns-orchestrator-provider = { path = "../dns-orchestrator-provider", default-features = false, features = ["all-providers", "native-tls"] }
keyring = { version = "3", features = ["apple-native", "windows-native", "sync-secret-service"] }

# Android
[target."cfg(target_os = \"android\")".dependencies]
dns-orchestrator-provider = { path = "../dns-orchestrator-provider", default-features = false, features = ["all-providers", "rustls"] }
tauri-plugin-stronghold = "2"  # Secure storage for Android
```

## Provider Library

### Design Goals

1. **Reusability**: Same provider code works in Tauri and actix-web backends
2. **Feature Flags**: Enable providers and TLS backends selectively
3. **Unified Error Handling**: `ProviderError` maps all provider-specific errors

### DnsProvider Trait

```rust
// dns-orchestrator-provider/src/traits.rs
#[async_trait]
pub trait DnsProvider: Send + Sync {
    async fn validate_credentials(&self) -> Result<()>;
    async fn list_domains(&self, params: &PaginationParams) -> Result<PaginatedResponse<Domain>>;
    async fn get_domain(&self, domain_id: &str) -> Result<Domain>;
    async fn list_records(&self, domain_id: &str, params: &RecordQueryParams) -> Result<PaginatedResponse<DnsRecord>>;
    async fn create_record(&self, req: &CreateDnsRecordRequest) -> Result<DnsRecord>;
    async fn update_record(&self, record_id: &str, req: &UpdateDnsRecordRequest) -> Result<DnsRecord>;
    async fn delete_record(&self, record_id: &str, domain_id: &str) -> Result<()>;
}
```

### Feature Flags

```toml
# dns-orchestrator-provider/Cargo.toml
[features]
default = ["native-tls", "all-providers"]

# TLS backend (choose one)
native-tls = ["reqwest/native-tls"]     # Desktop default
rustls = ["reqwest/rustls-tls"]          # Android (avoids OpenSSL cross-compile)

# Providers (enable individually or all)
cloudflare = []
aliyun = []
dnspod = []
huaweicloud = []
all-providers = ["cloudflare", "aliyun", "dnspod", "huaweicloud"]
```

### Error Handling

```rust
// dns-orchestrator-provider/src/error.rs
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "code")]
pub enum ProviderError {
    NetworkError { provider: String, detail: String },
    InvalidCredentials { provider: String },
    RecordExists { provider: String, record_name: String, raw_message: Option<String> },
    RecordNotFound { provider: String, record_id: String, raw_message: Option<String> },
    InvalidParameter { provider: String, param: String, detail: String },
    QuotaExceeded { provider: String, raw_message: Option<String> },
    DomainNotFound { provider: String, domain: String },
    ParseError { provider: String, detail: String },
    Unknown { provider: String, raw_code: Option<String>, raw_message: String },
}
```

## Security Architecture

### Credential Storage by Platform

| Platform | Storage Mechanism |
|----------|-------------------|
| **macOS** | Keychain via `keyring` crate |
| **Windows** | Credential Manager via `keyring` crate |
| **Linux** | Secret Service (GNOME Keyring/KWallet) via `keyring` crate |
| **Android** | Stronghold via `tauri-plugin-stronghold` |

### Account Import/Export Encryption

```rust
// AES-GCM encryption with PBKDF2 key derivation
pub fn encrypt_data(data: &str, password: &str) -> Result<String>
pub fn decrypt_data(encrypted: &str, password: &str) -> Result<String>
```

## Performance Optimizations

1. **Pagination**: Server-side pagination with 20 records per page
2. **Search Debouncing**: 300ms debounce on search input
3. **Infinite Scroll**: IntersectionObserver-based loading
4. **Rust Async**: Tokio async runtime for non-blocking I/O
5. **Feature Flags**: Only compile enabled providers

## Data Flow

### DNS Record Query Flow

```
1. User types in search box (debounced 300ms)
2. Store calls dnsService.listRecords()
3. Service uses transport.invoke('list_dns_records', args)
4. Transport routes to:
   - Tauri: IPC to Rust command
   - Web: HTTP POST to actix-web
5. Backend gets provider from registry
6. Provider makes HTTPS request to DNS API
7. Response flows back through layers
8. Store updates, UI re-renders
```

## Design Decisions

### Why Separate Provider Library?

| Benefit | Description |
|---------|-------------|
| **Reusability** | Same code for Tauri and actix-web backends |
| **Testability** | Unit test providers independently |
| **Feature Flags** | Compile only needed providers |
| **TLS Flexibility** | Switch between native-tls and rustls per platform |

### Why Transport Abstraction?

| Benefit | Description |
|---------|-------------|
| **Multi-Platform** | Same frontend code for desktop, mobile, and web |
| **Type Safety** | CommandMap enforces correct args/return types |
| **Testability** | Mock transport for frontend testing |

### Why actix-web for Web Backend?

| Criterion | actix-web | axum |
|-----------|-----------|------|
| **Performance** | Fastest Rust web framework | Very fast |
| **Maturity** | Battle-tested in production | Newer |
| **Ecosystem** | Large plugin ecosystem | Growing |

---

This architecture balances simplicity, security, and performance while supporting multiple platforms with shared code.
