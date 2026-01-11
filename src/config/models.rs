// 配置数据结构模型
// 统一使用 snake_case 命名风格

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 全局配置 (config.json)
// ============================================================================

/// 全局配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub version: String,
    pub active: ActiveConfigs,
    #[serde(default)]
    pub metadata: ConfigMetadata,
}

/// 当前激活的配置引用
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveConfigs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<OpenCodeActiveReference>,
}

/// 配置元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        Self {
            created_at: default_timestamp(),
            updated_at: default_timestamp(),
        }
    }
}

fn default_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ============================================================================
// 辅助实现
// ============================================================================

impl GlobalConfig {
    /// 创建新的全局配置
    pub fn new() -> Self {
        Self {
            version: "3.0.0".to_string(),
            active: ActiveConfigs::default(),
            metadata: ConfigMetadata::default(),
        }
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.metadata.updated_at = default_timestamp();
    }
}

// ============================================================================
// OpenCode 配置 (opencode.json)
// ============================================================================

/// OpenCode 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeConfig {
    #[serde(default = "default_opencode_version")]
    pub version: String,
    #[serde(default)]
    pub providers: HashMap<String, OpenCodeProvider>,
}

fn default_opencode_version() -> String {
    "3.0.0".to_string()
}

/// OpenCode Provider 配置 (匹配真实 opencode.json 格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeProvider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>, // 如: "@ai-sdk/openai-compatible"
    pub name: String,
    pub options: OpenCodeProviderOptions,
    pub models: HashMap<String, OpenCodeModelInfo>,
    // 内部元数据 (不同步到 opencode.json)
    #[serde(skip)]
    pub metadata: ProviderMetadata,
    // 站点检测结果 (持久化缓存，不同步到 opencode.json)
    #[serde(skip)]
    #[allow(dead_code)]
    pub site_detection: Option<SiteDetectionResult>,
}

/// Provider 选项配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeProviderOptions {
    #[serde(rename = "baseURL")]
    pub base_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
}

/// Provider 元数据 (仅用于内部管理)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProviderMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

/// 模型信息 (匹配真实格式)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeModelInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<OpenCodeModelLimit>,
    // 模型检测结果 (持久化缓存，不同步到 opencode.json)
    #[serde(skip)]
    #[allow(dead_code)]
    pub model_detection: Option<ModelDetectionResult>,
}

/// 模型限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeModelLimit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,
}

/// OpenCode 激活配置引用 (存储在 config.json 的 active.opencode)
/// 简化设计: 只需要记录当前激活的 Provider 名称即可
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeActiveReference {
    pub provider: String, // 当前激活的 Provider 名称
}

/// 完整激活配置 (运行时从引用+provider数据构建)
#[derive(Debug, Clone)]
pub struct OpenCodeActiveConfig {
    pub provider: String,
    #[allow(dead_code)]
    pub provider_description: Option<String>,
    pub base_url: String,
    #[allow(dead_code)]
    pub api_key: String,
    pub models: std::collections::HashMap<String, OpenCodeModelInfo>,
}

// ============================================================================
// OpenCode 实现方法
// ============================================================================

impl OpenCodeConfig {
    /// 创建新的 OpenCode 配置
    pub fn new() -> Self {
        Self {
            version: "3.0.0".to_string(),
            providers: HashMap::new(),
        }
    }

    /// 获取 Provider
    pub fn get_provider(&self, provider_name: &str) -> Option<&OpenCodeProvider> {
        self.providers.get(provider_name)
    }

    /// 获取可变 Provider
    pub fn get_provider_mut(&mut self, provider_name: &str) -> Option<&mut OpenCodeProvider> {
        self.providers.get_mut(provider_name)
    }

    /// 添加 Provider
    pub fn add_provider(&mut self, provider_name: String, provider: OpenCodeProvider) {
        self.providers.insert(provider_name, provider);
    }

    /// 删除 Provider
    pub fn remove_provider(&mut self, provider_name: &str) -> Option<OpenCodeProvider> {
        self.providers.remove(provider_name)
    }
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenCodeProvider {
    /// 创建新的 Provider
    pub fn new(
        name: String,
        base_url: String,
        api_key: String,
        npm: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            npm,
            name,
            options: OpenCodeProviderOptions { base_url, api_key },
            models: HashMap::new(),
            metadata: ProviderMetadata {
                description,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
            site_detection: None,
        }
    }

    /// 更新 API Key
    pub fn set_api_key(&mut self, api_key: String) {
        self.options.api_key = api_key;
        self.update_timestamp();
    }

    /// 更新 Base URL
    pub fn set_base_url(&mut self, base_url: String) {
        self.options.base_url = base_url;
        self.update_timestamp();
    }

    /// 获取模型
    pub fn get_model(&self, model_id: &str) -> Option<&OpenCodeModelInfo> {
        self.models.get(model_id)
    }

    /// 添加模型
    pub fn add_model(&mut self, model_id: String, model_info: OpenCodeModelInfo) {
        self.models.insert(model_id, model_info);
        self.update_timestamp();
    }

    /// 删除模型
    pub fn remove_model(&mut self, model_id: &str) -> Option<OpenCodeModelInfo> {
        let result = self.models.remove(model_id);
        self.update_timestamp();
        result
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.metadata.updated_at = default_timestamp();
    }
}

impl OpenCodeActiveConfig {
    /// 从引用和 Provider 配置创建完整运行时配置
    pub fn from_reference(
        reference: &OpenCodeActiveReference,
        config: &OpenCodeConfig,
    ) -> Result<Self, String> {
        let provider = config
            .get_provider(&reference.provider)
            .ok_or_else(|| format!("Provider '{}' not found", reference.provider))?;

        Ok(Self {
            provider: reference.provider.clone(),
            provider_description: provider.metadata.description.clone(),
            base_url: provider.options.base_url.clone(),
            api_key: provider.options.api_key.clone(),
            models: provider.models.clone(),
        })
    }
}

// ============================================================================
// 站点检测和模型检测数据结构
// ============================================================================

/// 站点检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteDetectionResult {
    /// 检测时间
    pub detected_at: String,

    /// 站点是否可用
    pub is_available: bool,

    /// API Key是否有效
    pub api_key_valid: bool,

    /// 检测到的模型列表
    pub available_models: Vec<String>,

    /// 站点响应时间(毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_time_ms: Option<f64>,

    /// 错误信息(如果检测失败)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

/// 模型检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDetectionResult {
    /// 检测时间
    pub detected_at: String,

    /// 模型ID
    pub model_id: String,

    /// 模型是否可用
    pub is_available: bool,

    /// 首次响应时间(TTFB, 毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_token_time_ms: Option<f64>,

    /// Token生成速度(tokens/秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_per_second: Option<f64>,

    /// 总响应时间(毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_response_time_ms: Option<f64>,

    /// 流式输出是否正常
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_available: Option<bool>,

    /// 错误信息(如果检测失败)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

// ============================================================================
// MCP 配置 (mcp.json)
// ============================================================================

/// MCP 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(default = "default_mcp_version")]
    pub version: String,
    #[serde(default)]
    pub servers: HashMap<String, McpServer>,
    // TODO: 后续迭代添加工具和代理工具配置
    // pub tools: HashMap<String, bool>,
    // pub agent_tools: HashMap<String, HashMap<String, bool>>,
}

fn default_mcp_version() -> String {
    "1.0.0".to_string()
}

/// MCP 服务器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerType {
    Local,
    Remote,
}

impl Default for McpServerType {
    fn default() -> Self {
        McpServerType::Local
    }
}

impl std::fmt::Display for McpServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpServerType::Local => write!(f, "local"),
            McpServerType::Remote => write!(f, "remote"),
        }
    }
}

/// MCP 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    /// 服务器类型
    #[serde(rename = "type")]
    pub server_type: McpServerType,

    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// 请求超时时间(毫秒)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,

    // === 本地服务器字段 ===
    /// 启动命令
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,

    /// 环境变量
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,

    // === 远程服务器字段 ===
    /// 远程 URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// HTTP Headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// OAuth 配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth: Option<McpOAuthConfig>,

    // === 内部元数据 ===
    #[serde(skip)]
    pub metadata: McpServerMetadata,
}

fn default_enabled() -> bool {
    true
}

/// MCP OAuth 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpOAuthConfig {
    /// Client ID (支持 {env:VAR} 格式)
    #[serde(rename = "clientId", skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// Client Secret (支持 {env:VAR} 格式)
    #[serde(rename = "clientSecret", skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// OAuth Scope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// MCP 服务器元数据 (仅用于内部管理)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServerMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

// ============================================================================
// MCP 实现方法
// ============================================================================

impl McpConfig {
    /// 创建新的 MCP 配置
    pub fn new() -> Self {
        Self {
            version: default_mcp_version(),
            servers: HashMap::new(),
        }
    }

    /// 获取服务器
    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.servers.get(name)
    }

    /// 添加服务器
    pub fn add_server(&mut self, name: String, server: McpServer) {
        self.servers.insert(name, server);
    }

    /// 获取按名称排序的服务器列表
    pub fn get_sorted_server_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.servers.keys().cloned().collect();
        names.sort();
        names
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl McpServer {
    /// 从 JSON Value 解析创建 McpServer
    /// 支持两种格式：
    /// 1. 本地服务器: { "command": "npx", "args": [...], "env": {...} }
    /// 2. 远程服务器: { "url": "https://...", "headers": {...}, "oauth": {...} }
    pub fn from_json(json: &serde_json::Value) -> Result<Self, String> {
        // 判断是本地还是远程服务器
        let is_local = json.get("command").is_some() || json.get("args").is_some();
        let is_remote = json.get("url").is_some();

        if is_local {
            Self::parse_local_from_json(json)
        } else if is_remote {
            Self::parse_remote_from_json(json)
        } else {
            // 无法识别的格式，创建空本地服务器
            Ok(Self::new_local(Vec::new(), HashMap::new()))
        }
    }

    /// 解析本地服务器配置
    fn parse_local_from_json(json: &serde_json::Value) -> Result<Self, String> {
        let mut command = Vec::new();
        if let Some(cmd) = json.get("command").and_then(|v| v.as_str()) {
            command.push(cmd.to_string());
        }
        if let Some(args) = json.get("args").and_then(|v| v.as_array()) {
            for arg in args {
                if let Some(s) = arg.as_str() {
                    command.push(s.to_string());
                }
            }
        }

        let mut environment = HashMap::new();
        if let Some(env) = json.get("env").and_then(|v| v.as_object()) {
            for (k, v) in env {
                if let Some(s) = v.as_str() {
                    environment.insert(k.clone(), s.to_string());
                }
            }
        }

        let timeout = json
            .get("timeout")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let enabled = json
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(Self {
            server_type: McpServerType::Local,
            enabled,
            timeout,
            command: Some(command),
            environment,
            url: None,
            headers: HashMap::new(),
            oauth: None,
            metadata: McpServerMetadata::default(),
        })
    }

    /// 解析远程服务器配置
    fn parse_remote_from_json(json: &serde_json::Value) -> Result<Self, String> {
        let url = json
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let mut headers = HashMap::new();
        if let Some(h) = json.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in h {
                if let Some(s) = v.as_str() {
                    headers.insert(k.clone(), s.to_string());
                }
            }
        }

        let oauth = if let Some(o) = json.get("oauth").and_then(|v| v.as_object()) {
            let client_id = o
                .get("clientId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let client_secret = o
                .get("clientSecret")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let scope = o
                .get("scope")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            if client_id.is_some() || client_secret.is_some() || scope.is_some() {
                Some(McpOAuthConfig {
                    client_id,
                    client_secret,
                    scope,
                })
            } else {
                None
            }
        } else {
            None
        };

        let timeout = json
            .get("timeout")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let enabled = json
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        Ok(Self {
            server_type: McpServerType::Remote,
            enabled,
            timeout,
            command: None,
            environment: HashMap::new(),
            url: Some(url),
            headers,
            oauth,
            metadata: McpServerMetadata::default(),
        })
    }

    /// 创建本地 MCP 服务器
    pub fn new_local(command: Vec<String>, environment: HashMap<String, String>) -> Self {
        Self {
            server_type: McpServerType::Local,
            enabled: true,
            timeout: None,
            command: Some(command),
            environment,
            url: None,
            headers: HashMap::new(),
            oauth: None,
            metadata: McpServerMetadata {
                description: None,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
        }
    }

    /// 创建远程 MCP 服务器
    pub fn new_remote(
        url: String,
        headers: HashMap<String, String>,
        oauth: Option<McpOAuthConfig>,
    ) -> Self {
        Self {
            server_type: McpServerType::Remote,
            enabled: true,
            timeout: None,
            command: None,
            environment: HashMap::new(),
            url: Some(url),
            headers,
            oauth,
            metadata: McpServerMetadata {
                description: None,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
        }
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.metadata.updated_at = default_timestamp();
    }

    /// 获取显示用的类型名称（本地/远程）
    pub fn type_display(&self) -> &'static str {
        match self.server_type {
            McpServerType::Local => "本地",
            McpServerType::Remote => "远程",
        }
    }
}

impl McpOAuthConfig {
    /// 检查是否为空配置
    pub fn is_empty(&self) -> bool {
        self.client_id.is_none() && self.client_secret.is_none() && self.scope.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_creation() {
        let config = GlobalConfig::new();
        assert_eq!(config.version, "3.0.0");
        assert!(config.active.opencode.is_none());
    }

    #[test]
    fn test_opencode_config_creation() {
        let config = OpenCodeConfig::new();
        assert_eq!(config.version, "3.0.0");
        assert!(config.providers.is_empty());
    }

    #[test]
    fn test_mcp_config_creation() {
        let config = McpConfig::new();
        assert_eq!(config.version, "1.0.0");
        assert!(config.servers.is_empty());
    }

    #[test]
    fn test_mcp_local_server() {
        let server = McpServer::new_local(
            vec![
                "npx".to_string(),
                "-y".to_string(),
                "test-server".to_string(),
            ],
            HashMap::new(),
        );
        assert_eq!(server.server_type, McpServerType::Local);
        assert!(server.enabled);
        assert!(server.command.is_some());
        assert!(server.url.is_none());
    }

    #[test]
    fn test_mcp_remote_server() {
        let server =
            McpServer::new_remote("https://mcp.example.com".to_string(), HashMap::new(), None);
        assert_eq!(server.server_type, McpServerType::Remote);
        assert!(server.enabled);
        assert!(server.url.is_some());
        assert!(server.command.is_none());
    }
}
