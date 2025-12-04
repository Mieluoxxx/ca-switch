// Codex 配置管理器
// 负责管理 ~/.cc-cli/codex.json 和同步到 ~/.codex/

use crate::config::models::{CodexActiveConfig, CodexConfig, CodexSite};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Codex 配置管理器
pub struct CodexConfigManager {
    #[allow(dead_code)]
    config_dir: PathBuf,         // ~/.cc-cli
    codex_config_file: PathBuf,  // ~/.cc-cli/codex.json
    codex_dir: PathBuf,          // ~/.codex
    codex_config_toml: PathBuf,  // ~/.codex/config.toml
    codex_auth_json: PathBuf,    // ~/.codex/auth.json
}

impl CodexConfigManager {
    /// 创建新的 Codex 配置管理器
    pub fn new(config_dir: PathBuf) -> Result<Self, String> {
        // 确保 ~/.cc-cli 目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let codex_config_file = config_dir.join("codex.json");

        // Codex 官方配置目录
        let codex_dir = dirs::home_dir()
            .ok_or("无法获取用户主目录")?
            .join(".codex");

        let codex_config_toml = codex_dir.join("config.toml");
        let codex_auth_json = codex_dir.join("auth.json");

        Ok(Self {
            config_dir,
            codex_config_file,
            codex_dir,
            codex_config_toml,
            codex_auth_json,
        })
    }

    // ========================================================================
    // 配置文件读写
    // ========================================================================

    /// 读取 codex.json 配置
    pub fn read_config(&self) -> Result<CodexConfig, String> {
        if !self.codex_config_file.exists() {
            return Ok(CodexConfig::new());
        }

        let content = fs::read_to_string(&self.codex_config_file)
            .map_err(|e| format!("读取 codex.json 失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析 codex.json 失败: {}", e))
    }

    /// 写入 codex.json 配置
    pub fn write_config(&self, config: &CodexConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 codex.json 失败: {}", e))?;

        fs::write(&self.codex_config_file, content)
            .map_err(|e| format!("写入 codex.json 失败: {}", e))
    }

    // ========================================================================
    // 站点管理
    // ========================================================================

    /// 获取站点
    pub fn get_site(&self, site_name: &str) -> Result<Option<CodexSite>, String> {
        let config = self.read_config()?;
        Ok(config.get_site(site_name).cloned())
    }

    /// 获取所有站点
    pub fn get_all_sites(&self) -> Result<HashMap<String, CodexSite>, String> {
        let config = self.read_config()?;
        Ok(config.sites.clone())
    }

    /// 添加站点
    pub fn add_site(
        &mut self,
        site_name: String,
        url: String,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        // 检查站点是否已存在
        if config.get_site(&site_name).is_some() {
            return Err(format!("站点 '{}' 已存在", site_name));
        }

        let site = CodexSite::new(url, description);
        config.add_site(site_name, site);

        self.write_config(&config)
    }

    /// 更新站点元数据
    pub fn update_site_metadata(
        &mut self,
        site_name: &str,
        url: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if let Some(url) = url {
            site.metadata.url = url;
        }
        if description.is_some() {
            site.metadata.description = description;
        }

        site.update_timestamp();

        self.write_config(&config)
    }

    /// 删除站点
    pub fn delete_site(&mut self, site_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.remove_site(site_name).is_none() {
            return Err(format!("站点 '{}' 不存在", site_name));
        }

        self.write_config(&config)
    }

    /// 删除站点（兼容接口）
    #[allow(dead_code)]
    pub fn remove_site(&self, site_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.remove_site(site_name).is_none() {
            return Err(format!("站点 '{}' 不存在", site_name));
        }

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("序列化 codex.json 失败: {}", e))?;

        fs::write(&self.codex_config_file, content)
            .map_err(|e| format!("写入 codex.json 失败: {}", e))
    }

    /// 更新站点配置
    pub fn update_site_config(
        &mut self,
        site_name: &str,
        base_url: Option<String>,
        model: Option<String>,
        model_reasoning_effort: Option<String>,
        model_provider: Option<String>,
        network_access: Option<String>,
        disable_response_storage: Option<bool>,
        wire_api: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        // 更新配置
        if let Some(base_url) = base_url {
            site.config.base_url = Some(base_url);
        }
        if let Some(model) = model {
            site.config.model = Some(model);
        }
        if let Some(model_reasoning_effort) = model_reasoning_effort {
            site.config.model_reasoning_effort = Some(model_reasoning_effort);
        }
        if let Some(model_provider) = model_provider {
            site.config.model_provider = Some(model_provider);
        }
        if let Some(network_access) = network_access {
            site.config.network_access = Some(network_access);
        }
        if let Some(disable_response_storage) = disable_response_storage {
            site.config.disable_response_storage = Some(disable_response_storage);
        }
        if let Some(wire_api) = wire_api {
            site.config.wire_api = Some(wire_api);
        }

        site.update_timestamp();

        self.write_config(&config)
    }

    // ========================================================================
    // API Key 管理
    // ========================================================================

    /// 获取 API Keys
    #[allow(dead_code)]
    pub fn get_api_keys(&self, site_name: &str) -> Result<HashMap<String, String>, String> {
        let config = self.read_config()?;
        let site = config
            .get_site(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        Ok(site.api_keys.clone())
    }

    /// 添加 API Key
    pub fn add_api_key(
        &mut self,
        site_name: &str,
        key_name: String,
        api_key: String,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        // 检查 key 是否已存在
        if site.get_api_key(&key_name).is_some() {
            return Err(format!("API Key '{}' 已存在于站点 '{}'", key_name, site_name));
        }

        site.add_api_key(key_name, api_key);

        self.write_config(&config)
    }

    /// 更新 API Key
    pub fn update_api_key(
        &mut self,
        site_name: &str,
        key_name: &str,
        new_api_key: String,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if site.get_api_key(key_name).is_none() {
            return Err(format!("API Key '{}' 不存在于站点 '{}'", key_name, site_name));
        }

        site.add_api_key(key_name.to_string(), new_api_key);

        self.write_config(&config)
    }

    /// 删除 API Key
    pub fn delete_api_key(&mut self, site_name: &str, key_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if site.remove_api_key(key_name).is_none() {
            return Err(format!("API Key '{}' 不存在于站点 '{}'", key_name, site_name));
        }

        self.write_config(&config)
    }

    // ========================================================================
    // 配置同步到 ~/.codex/
    // ========================================================================

    /// 同步配置到 Codex 官方配置文件
    pub fn sync_to_codex(&self, active_config: &CodexActiveConfig) -> Result<(), String> {
        // 确保 ~/.codex 目录存在
        if !self.codex_dir.exists() {
            fs::create_dir_all(&self.codex_dir)
                .map_err(|e| format!("创建 .codex 目录失败: {}", e))?;
        }

        // 同步到 auth.json
        self.sync_to_auth_json(active_config)?;

        // 同步到 config.toml
        self.sync_to_config_toml(active_config)?;

        Ok(())
    }

    /// 同步到 auth.json
    fn sync_to_auth_json(&self, active_config: &CodexActiveConfig) -> Result<(), String> {
        let auth_data = serde_json::json!({
            "OPENAI_API_KEY": active_config.api_key,
        });

        let content = serde_json::to_string_pretty(&auth_data)
            .map_err(|e| format!("序列化 auth.json 失败: {}", e))?;

        fs::write(&self.codex_auth_json, content)
            .map_err(|e| format!("写入 auth.json 失败: {}", e))
    }

    /// 同步到 config.toml
    fn sync_to_config_toml(&self, active_config: &CodexActiveConfig) -> Result<(), String> {
        let mut lines = Vec::new();

        // Model Provider（如果不填则默认使用站点名）
        let provider_name = active_config
            .model_provider
            .as_ref()
            .unwrap_or(&active_config.site);
        lines.push(format!("model_provider = \"{}\"", provider_name));

        // Model
        if let Some(ref model) = active_config.model {
            lines.push(format!("model = \"{}\"", model));
        }

        // Model Reasoning Effort
        if let Some(ref reasoning_effort) = active_config.model_reasoning_effort {
            lines.push(format!("model_reasoning_effort = \"{}\"", reasoning_effort));
        }

        // Network Access
        if let Some(ref network_access) = active_config.network_access {
            lines.push(format!("network_access = \"{}\"", network_access));
        }

        // Disable Response Storage
        if let Some(disable_response_storage) = active_config.disable_response_storage {
            lines.push(format!("disable_response_storage = {}", disable_response_storage));
        }

        // 添加空行
        lines.push(String::new());

        // [model_providers.xxx] 配置块
        lines.push(format!("[model_providers.{}]", provider_name));
        lines.push(format!("name = \"{}\"", provider_name));

        // Base URL（必需）
        if let Some(ref base_url) = active_config.base_url {
            lines.push(format!("base_url = \"{}\"", base_url));
        }

        // Wire API
        if let Some(ref wire_api) = active_config.wire_api {
            lines.push(format!("wire_api = \"{}\"", wire_api));
        }

        // Requires OpenAI Auth（默认为 true）
        lines.push("requires_openai_auth = true".to_string());

        let content = lines.join("\n") + "\n";

        fs::write(&self.codex_config_toml, content)
            .map_err(|e| format!("写入 config.toml 失败: {}", e))
    }

    // ========================================================================
    // 辅助方法
    // ========================================================================

    /// 获取配置文件路径（用于备份等）
    pub fn get_config_file_path(&self) -> &PathBuf {
        &self.codex_config_file
    }

    /// 获取 config.toml 路径
    pub fn get_config_toml_path(&self) -> &PathBuf {
        &self.codex_config_toml
    }

    /// 获取 auth.json 路径
    pub fn get_auth_json_path(&self) -> &PathBuf {
        &self.codex_auth_json
    }
}
