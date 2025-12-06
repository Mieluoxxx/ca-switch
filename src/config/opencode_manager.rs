// OpenCode 配置管理器
// 负责管理 ~/.ca-switch/opencode.json 和同步到 ~/.opencode/opencode.json

use crate::config::models::{
    OpenCodeActiveConfig, OpenCodeConfig, OpenCodeModelInfo, OpenCodeProvider,
};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// OpenCode 配置管理器
pub struct OpenCodeConfigManager {
    opencode_config_file: PathBuf,  // ~/.ca-switch/opencode.json
    opencode_dir: PathBuf,          // ~/.opencode
    opencode_json: PathBuf,         // ~/.opencode/opencode.json
}

impl OpenCodeConfigManager {
    /// 创建新的 OpenCode 配置管理器
    pub fn new(config_dir: PathBuf) -> Result<Self, String> {
        // 确保 ~/.ca-switch 目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let opencode_config_file = config_dir.join("opencode.json");

        // OpenCode 官方配置目录
        let opencode_dir = dirs::home_dir()
            .ok_or("无法获取用户主目录")?
            .join(".opencode");

        let opencode_json = opencode_dir.join("opencode.json");

        Ok(Self {
            opencode_config_file,
            opencode_dir,
            opencode_json,
        })
    }

    // ========================================================================
    // 配置文件读写
    // ========================================================================

    /// 读取 opencode.json 配置
    pub fn read_config(&self) -> Result<OpenCodeConfig, String> {
        if !self.opencode_config_file.exists() {
            return Ok(OpenCodeConfig::new());
        }

        let content = fs::read_to_string(&self.opencode_config_file)
            .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析 opencode.json 失败: {}", e))
    }

    /// 写入 opencode.json 配置
    pub fn write_config(&self, config: &OpenCodeConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

        fs::write(&self.opencode_config_file, content)
            .map_err(|e| format!("写入 opencode.json 失败: {}", e))
    }

    // ========================================================================
    // Provider 管理
    // ========================================================================

    /// 获取 Provider
    pub fn get_provider(&self, provider_name: &str) -> Result<Option<OpenCodeProvider>, String> {
        let config = self.read_config()?;
        Ok(config.get_provider(provider_name).cloned())
    }

    /// 获取所有 Provider
    pub fn get_all_providers(&self) -> Result<HashMap<String, OpenCodeProvider>, String> {
        let config = self.read_config()?;
        Ok(config.providers.clone())
    }

    /// 添加 Provider
    pub fn add_provider(
        &mut self,
        provider_name: String,
        base_url: String,
        api_key: String,
        npm: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        // 检查 Provider 是否已存在
        if config.get_provider(&provider_name).is_some() {
            return Err(format!("Provider '{}' 已存在", provider_name));
        }

        let provider = OpenCodeProvider::new(provider_name.clone(), base_url, api_key, npm, description);
        config.add_provider(provider_name, provider);

        self.write_config(&config)
    }

    /// 更新 Provider 元数据
    pub fn update_provider_metadata(
        &mut self,
        provider_name: &str,
        base_url: Option<String>,
        api_key: Option<String>,
        npm: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        if let Some(url) = base_url {
            provider.set_base_url(url);
        }
        if let Some(key) = api_key {
            provider.set_api_key(key);
        }
        if npm.is_some() {
            provider.npm = npm;
            provider.update_timestamp();
        }
        if description.is_some() {
            provider.metadata.description = description;
            provider.update_timestamp();
        }

        self.write_config(&config)
    }

    /// 删除 Provider
    pub fn delete_provider(&mut self, provider_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.remove_provider(provider_name).is_none() {
            return Err(format!("Provider '{}' 不存在", provider_name));
        }

        self.write_config(&config)
    }

    // ========================================================================
    // 模型管理
    // ========================================================================

    /// 获取模型
    pub fn get_models(&self, provider_name: &str) -> Result<HashMap<String, OpenCodeModelInfo>, String> {
        let config = self.read_config()?;
        let provider = config
            .get_provider(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        Ok(provider.models.clone())
    }

    /// 添加模型
    pub fn add_model(
        &mut self,
        provider_name: &str,
        model_id: String,
        model_info: OpenCodeModelInfo,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        // 检查模型是否已存在
        if provider.get_model(&model_id).is_some() {
            return Err(format!(
                "模型 '{}' 已存在于 Provider '{}'",
                model_id, provider_name
            ));
        }

        provider.add_model(model_id, model_info);

        self.write_config(&config)
    }

    /// 删除模型
    pub fn delete_model(&mut self, provider_name: &str, model_id: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        let provider = config
            .get_provider_mut(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        if provider.remove_model(model_id).is_none() {
            return Err(format!(
                "模型 '{}' 不存在于 Provider '{}'",
                model_id, provider_name
            ));
        }

        self.write_config(&config)
    }

    // ========================================================================
    // 配置同步到 ~/.opencode/opencode.json
    // ========================================================================

    /// 同步配置到 OpenCode 官方配置文件 (生成完整的 opencode.json)
    pub fn sync_to_opencode(&self, active_config: &OpenCodeActiveConfig) -> Result<(), String> {
        // 确保 ~/.opencode 目录存在
        if !self.opencode_dir.exists() {
            fs::create_dir_all(&self.opencode_dir)
                .map_err(|e| format!("创建 .opencode 目录失败: {}", e))?;
        }

        // 读取完整的 provider 配置
        let opencode_config = self.read_config()?;

        // 构建 provider 对象（只包含当前激活的 provider）
        let mut providers_map = serde_json::Map::new();

        if let Some(provider) = opencode_config.get_provider(&active_config.provider) {
            providers_map.insert(
                active_config.provider.clone(),
                serde_json::to_value(provider)
                    .map_err(|e| format!("序列化 Provider 失败: {}", e))?,
            );
        }

        // 构建完整的 opencode.json 结构
        // 注意: 不再设置 model 和 small_model,让 opencode 自己选择
        let sync_data = serde_json::json!({
            "$schema": "https://opencode.ai/config.json",
            "theme": "tokyonight",
            "autoupdate": false,
            "provider": providers_map,
            "tools": {
                "get-current-session-id": true,
                "webfetch": true
            },
            "agent": {},
            "mcp": {}
        });

        let content = serde_json::to_string_pretty(&sync_data)
            .map_err(|e| format!("序列化同步数据失败: {}", e))?;

        fs::write(&self.opencode_json, content)
            .map_err(|e| format!("写入 ~/.opencode/opencode.json 失败: {}", e))
    }

    /// 同步配置到项目级 .opencode/opencode.json
    pub fn sync_to_project(&self, active_config: &OpenCodeActiveConfig) -> Result<(), String> {
        // 获取当前工作目录
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("获取当前目录失败: {}", e))?;

        let project_opencode_dir = current_dir.join(".opencode");
        let project_opencode_json = project_opencode_dir.join("opencode.json");

        // 确保 .opencode 目录存在
        if !project_opencode_dir.exists() {
            fs::create_dir_all(&project_opencode_dir)
                .map_err(|e| format!("创建项目 .opencode 目录失败: {}", e))?;
        }

        // 读取完整的 provider 配置
        let opencode_config = self.read_config()?;

        // 构建 provider 对象（只包含当前激活的 provider）
        let mut providers_map = serde_json::Map::new();

        if let Some(provider) = opencode_config.get_provider(&active_config.provider) {
            providers_map.insert(
                active_config.provider.clone(),
                serde_json::to_value(provider)
                    .map_err(|e| format!("序列化 Provider 失败: {}", e))?,
            );
        }

        // 构建完整的 opencode.json 结构
        let sync_data = serde_json::json!({
            "$schema": "https://opencode.ai/config.json",
            "theme": "tokyonight",
            "autoupdate": false,
            "provider": providers_map,
            "tools": {
                "get-current-session-id": true,
                "webfetch": true
            },
            "agent": {},
            "mcp": {}
        });

        let content = serde_json::to_string_pretty(&sync_data)
            .map_err(|e| format!("序列化同步数据失败: {}", e))?;

        fs::write(&project_opencode_json, content)
            .map_err(|e| format!("写入 .opencode/opencode.json 失败: {}", e))
    }
}
