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
        assert!(config.active.opencode.is_none());
    }

    #[test]
    fn test_opencode_config_creation() {
        let config = OpenCodeConfig::new();
        assert_eq!(config.version, "3.0.0");
        assert!(config.providers.is_empty());
    }
}
