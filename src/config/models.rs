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
    pub claude: Option<ClaudeActiveReference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex: Option<CodexActiveReference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gemini: Option<GeminiActiveReference>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencode: Option<OpenCodeActiveReference>,
}

/// Claude 激活配置引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeActiveReference {
    pub site: String,
    pub token_name: String,
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
// Claude 配置 (claude.json)
// ============================================================================

/// Claude 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    pub version: String,
    pub sites: HashMap<String, ClaudeSite>,
}

/// Claude 站点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSite {
    pub metadata: SiteMetadata,
    pub tokens: HashMap<String, String>,
    pub config: ClaudeSiteConfig,
}

/// 站点元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMetadata {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

/// Claude 站点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeSiteConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(default)]
    pub vertex: VertexConfig,
}

/// Vertex AI 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VertexConfig {
    #[serde(default)]
    pub enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(default)]
    pub skip_auth: bool,
}

// ============================================================================
// 运行时配置（从引用解析出的完整配置）
// ============================================================================

/// Claude 运行时激活配置（完整信息）
#[derive(Debug, Clone)]
pub struct ClaudeActiveConfig {
    pub site: String,
    pub site_url: String,
    #[allow(dead_code)]
    pub site_description: Option<String>,
    pub token_name: String,
    pub token: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub vertex: VertexConfig,
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

impl ClaudeConfig {
    /// 创建新的 Claude 配置
    pub fn new() -> Self {
        Self {
            version: "3.0.0".to_string(),
            sites: HashMap::new(),
        }
    }

    /// 获取站点
    pub fn get_site(&self, site_name: &str) -> Option<&ClaudeSite> {
        self.sites.get(site_name)
    }

    /// 获取站点（可变）
    pub fn get_site_mut(&mut self, site_name: &str) -> Option<&mut ClaudeSite> {
        self.sites.get_mut(site_name)
    }

    /// 添加站点
    pub fn add_site(&mut self, site_name: String, site: ClaudeSite) {
        self.sites.insert(site_name, site);
    }

    /// 删除站点
    pub fn remove_site(&mut self, site_name: &str) -> Option<ClaudeSite> {
        self.sites.remove(site_name)
    }
}

impl ClaudeSite {
    /// 创建新站点
    pub fn new(url: String, description: Option<String>) -> Self {
        Self {
            metadata: SiteMetadata {
                url,
                description,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
            tokens: HashMap::new(),
            config: ClaudeSiteConfig::default(),
        }
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.metadata.updated_at = default_timestamp();
    }

    /// 获取 token
    pub fn get_token(&self, token_name: &str) -> Option<&String> {
        self.tokens.get(token_name)
    }

    /// 添加 token
    pub fn add_token(&mut self, token_name: String, token: String) {
        self.tokens.insert(token_name, token);
        self.update_timestamp();
    }

    /// 删除 token
    pub fn remove_token(&mut self, token_name: &str) -> Option<String> {
        let result = self.tokens.remove(token_name);
        if result.is_some() {
            self.update_timestamp();
        }
        result
    }
}

impl Default for ClaudeSiteConfig {
    fn default() -> Self {
        Self {
            base_url: None,
            model: None,
            vertex: VertexConfig::default(),
        }
    }
}

impl ClaudeActiveConfig {
    /// 从引用和站点配置创建运行时配置
    pub fn from_reference(
        reference: &ClaudeActiveReference,
        site: &ClaudeSite,
    ) -> Result<Self, String> {
        let token = site
            .get_token(&reference.token_name)
            .ok_or_else(|| format!("Token '{}' not found in site '{}'", reference.token_name, reference.site))?;

        Ok(Self {
            site: reference.site.clone(),
            site_url: site.metadata.url.clone(),
            site_description: site.metadata.description.clone(),
            token_name: reference.token_name.clone(),
            token: token.clone(),
            base_url: site.config.base_url.clone(),
            model: site.config.model.clone(),
            vertex: site.config.vertex.clone(),
        })
    }
}

// ============================================================================
// Codex 配置 (codex.json)
// ============================================================================

/// Codex 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    pub version: String,
    pub sites: HashMap<String, CodexSite>,
}

/// Codex 站点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexSite {
    pub metadata: SiteMetadata,
    pub api_keys: HashMap<String, String>, // key_name -> api_key
    pub config: CodexSiteConfig,
}

/// Codex 站点级配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexSiteConfig {
    // 基础配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_reasoning_effort: Option<String>, // 正确的字段名

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_provider: Option<String>, // 如果不填则默认继承站点名

    // 额外配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_access: Option<String>, // "enabled" or "disabled"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_response_storage: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub wire_api: Option<String>, // "responses" 等
}

/// Codex 激活配置引用（存储在 config.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexActiveReference {
    pub site: String,
    pub api_key_name: String,
}

/// Codex 运行时激活配置（完整信息）
#[derive(Debug, Clone)]
pub struct CodexActiveConfig {
    pub site: String,
    #[allow(dead_code)]
    pub site_url: String,
    #[allow(dead_code)]
    pub site_description: Option<String>,
    pub api_key_name: String,
    pub api_key: String,

    // 基础配置
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub model_reasoning_effort: Option<String>, // 正确的字段名
    pub model_provider: Option<String>, // 如果不填则默认继承站点名

    // 额外配置
    pub network_access: Option<String>,
    pub disable_response_storage: Option<bool>,
    pub wire_api: Option<String>,
}

// ============================================================================
// Codex 辅助实现
// ============================================================================

impl CodexConfig {
    /// 创建新的 Codex 配置
    pub fn new() -> Self {
        Self {
            version: "3.0.0".to_string(),
            sites: HashMap::new(),
        }
    }

    /// 获取站点
    pub fn get_site(&self, site_name: &str) -> Option<&CodexSite> {
        self.sites.get(site_name)
    }

    /// 获取站点（可变）
    pub fn get_site_mut(&mut self, site_name: &str) -> Option<&mut CodexSite> {
        self.sites.get_mut(site_name)
    }

    /// 添加站点
    pub fn add_site(&mut self, site_name: String, site: CodexSite) {
        self.sites.insert(site_name, site);
    }

    /// 删除站点
    pub fn remove_site(&mut self, site_name: &str) -> Option<CodexSite> {
        self.sites.remove(site_name)
    }
}

impl CodexSite {
    /// 创建新站点
    pub fn new(url: String, description: Option<String>) -> Self {
        Self {
            metadata: SiteMetadata {
                url,
                description,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
            api_keys: HashMap::new(),
            config: CodexSiteConfig::default(),
        }
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.metadata.updated_at = default_timestamp();
    }

    /// 获取 API key
    pub fn get_api_key(&self, key_name: &str) -> Option<&String> {
        self.api_keys.get(key_name)
    }

    /// 添加 API key
    pub fn add_api_key(&mut self, key_name: String, api_key: String) {
        self.api_keys.insert(key_name, api_key);
        self.update_timestamp();
    }

    /// 删除 API key
    pub fn remove_api_key(&mut self, key_name: &str) -> Option<String> {
        let result = self.api_keys.remove(key_name);
        self.update_timestamp();
        result
    }
}

impl CodexActiveConfig {
    /// 从引用和站点配置创建运行时配置
    pub fn from_reference(
        reference: &CodexActiveReference,
        site: &CodexSite,
    ) -> Result<Self, String> {
        let api_key = site
            .get_api_key(&reference.api_key_name)
            .ok_or_else(|| format!("API Key '{}' not found in site '{}'", reference.api_key_name, reference.site))?;

        Ok(Self {
            site: reference.site.clone(),
            site_url: site.metadata.url.clone(),
            site_description: site.metadata.description.clone(),
            api_key_name: reference.api_key_name.clone(),
            api_key: api_key.clone(),
            base_url: site.config.base_url.clone(),
            model: site.config.model.clone(),
            model_reasoning_effort: site.config.model_reasoning_effort.clone(),
            model_provider: site.config.model_provider.clone(),
            network_access: site.config.network_access.clone(),
            disable_response_storage: site.config.disable_response_storage,
            wire_api: site.config.wire_api.clone(),
        })
    }
}

// ============================================================================
// Gemini 配置 (gemini.json)
// ============================================================================

/// Gemini 配置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub version: String,
    pub sites: HashMap<String, GeminiSite>,
}

/// Gemini 站点配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiSite {
    pub metadata: SiteMetadata,
    pub api_keys: HashMap<String, String>, // key_name -> api_key
    pub config: GeminiSiteConfig,
}

/// Gemini 站点级配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeminiSiteConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Gemini 激活配置引用（存储在 config.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiActiveReference {
    pub site: String,
    pub api_key_name: String,
}

/// Gemini 运行时激活配置（完整信息）
#[derive(Debug, Clone)]
pub struct GeminiActiveConfig {
    pub site: String,
    #[allow(dead_code)]
    pub site_url: String,
    #[allow(dead_code)]
    pub site_description: Option<String>,
    pub api_key_name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: Option<String>,
}

// ============================================================================
// Gemini 辅助实现
// ============================================================================

impl GeminiConfig {
    /// 创建新的 Gemini 配置
    pub fn new() -> Self {
        Self {
            version: "3.0.0".to_string(),
            sites: HashMap::new(),
        }
    }

    /// 获取站点
    pub fn get_site(&self, site_name: &str) -> Option<&GeminiSite> {
        self.sites.get(site_name)
    }

    /// 获取可变站点
    pub fn get_site_mut(&mut self, site_name: &str) -> Option<&mut GeminiSite> {
        self.sites.get_mut(site_name)
    }

    /// 添加站点
    pub fn add_site(&mut self, site_name: String, site: GeminiSite) {
        self.sites.insert(site_name, site);
    }

    /// 删除站点
    pub fn remove_site(&mut self, site_name: &str) -> Option<GeminiSite> {
        self.sites.remove(site_name)
    }
}

impl GeminiSite {
    /// 创建新站点
    pub fn new(url: String, description: Option<String>) -> Self {
        Self {
            metadata: SiteMetadata {
                url,
                description,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
            api_keys: HashMap::new(),
            config: GeminiSiteConfig::default(),
        }
    }

    /// 更新时间戳
    pub fn update_timestamp(&mut self) {
        self.metadata.updated_at = default_timestamp();
    }

    /// 获取 API key
    pub fn get_api_key(&self, key_name: &str) -> Option<&String> {
        self.api_keys.get(key_name)
    }

    /// 添加 API key
    pub fn add_api_key(&mut self, key_name: String, api_key: String) {
        self.api_keys.insert(key_name, api_key);
        self.update_timestamp();
    }

    /// 删除 API key
    pub fn remove_api_key(&mut self, key_name: &str) -> Option<String> {
        let result = self.api_keys.remove(key_name);
        self.update_timestamp();
        result
    }
}

impl GeminiActiveConfig {
    /// 从引用和站点配置创建运行时配置
    pub fn from_reference(
        reference: &GeminiActiveReference,
        site: &GeminiSite,
    ) -> Result<Self, String> {
        let api_key = site
            .get_api_key(&reference.api_key_name)
            .ok_or_else(|| format!("API Key '{}' not found in site '{}'", reference.api_key_name, reference.site))?;

        Ok(Self {
            site: reference.site.clone(),
            site_url: site.metadata.url.clone(),
            site_description: site.metadata.description.clone(),
            api_key_name: reference.api_key_name.clone(),
            api_key: api_key.clone(),
            base_url: site.config.base_url.clone(),
            model: site.config.model.clone(),
        })
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
    pub fn new(name: String, base_url: String, api_key: String, npm: Option<String>, description: Option<String>) -> Self {
        Self {
            npm,
            name,
            options: OpenCodeProviderOptions {
                base_url,
                api_key,
            },
            models: HashMap::new(),
            metadata: ProviderMetadata {
                description,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
            site_detection: None,
        }
    }

    /// 获取 API Key
    #[allow(dead_code)]
    pub fn get_api_key(&self) -> &String {
        &self.options.api_key
    }

    /// 更新 API Key
    pub fn set_api_key(&mut self, api_key: String) {
        self.options.api_key = api_key;
        self.update_timestamp();
    }

    /// 获取 Base URL
    #[allow(dead_code)]
    pub fn get_base_url(&self) -> &String {
        &self.options.base_url
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
            .ok_or_else(|| {
                format!("Provider '{}' not found", reference.provider)
            })?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config_creation() {
        let config = GlobalConfig::new();
        assert_eq!(config.version, "3.0.0");
        assert!(config.active.claude.is_none());
    }

    #[test]
    fn test_claude_config_creation() {
        let config = ClaudeConfig::new();
        assert_eq!(config.version, "3.0.0");
        assert!(config.sites.is_empty());
    }

    #[test]
    fn test_claude_site_operations() {
        let mut site = ClaudeSite::new(
            "https://api.example.com".to_string(),
            Some("Test Site".to_string()),
        );

        // 测试添加 token
        site.add_token("main".to_string(), "sk-xxx".to_string());
        assert_eq!(site.get_token("main"), Some(&"sk-xxx".to_string()));

        // 测试删除 token
        let removed = site.remove_token("main");
        assert_eq!(removed, Some("sk-xxx".to_string()));
        assert_eq!(site.get_token("main"), None);
    }
}
