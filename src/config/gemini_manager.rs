// Gemini 配置管理器
// 负责管理 ~/.cc-cli/gemini.json 和同步到 ~/.gemini/

use crate::config::models::{GeminiActiveConfig, GeminiConfig, GeminiSite};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Gemini 配置管理器
pub struct GeminiConfigManager {
    config_dir: PathBuf,          // ~/.cc-cli
    gemini_config_file: PathBuf,  // ~/.cc-cli/gemini.json
    gemini_dir: PathBuf,           // ~/.gemini
    gemini_env_file: PathBuf,      // ~/.gemini/.env
    gemini_settings_file: PathBuf, // ~/.gemini/settings.json
}

impl GeminiConfigManager {
    /// 创建新的 Gemini 配置管理器
    pub fn new(config_dir: PathBuf) -> Result<Self, String> {
        // 确保 ~/.cc-cli 目录存在
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let gemini_config_file = config_dir.join("gemini.json");

        // Gemini 官方配置目录
        let gemini_dir = dirs::home_dir()
            .ok_or("无法获取用户主目录")?
            .join(".gemini");

        let gemini_env_file = gemini_dir.join(".env");
        let gemini_settings_file = gemini_dir.join("settings.json");

        Ok(Self {
            config_dir,
            gemini_config_file,
            gemini_dir,
            gemini_env_file,
            gemini_settings_file,
        })
    }

    // ========================================================================
    // 配置文件读写
    // ========================================================================

    /// 读取 gemini.json 配置
    pub fn read_config(&self) -> Result<GeminiConfig, String> {
        if !self.gemini_config_file.exists() {
            return Ok(GeminiConfig::new());
        }

        let content = fs::read_to_string(&self.gemini_config_file)
            .map_err(|e| format!("读取 gemini.json 失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析 gemini.json 失败: {}", e))
    }

    /// 写入 gemini.json 配置
    pub fn write_config(&self, config: &GeminiConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 gemini.json 失败: {}", e))?;

        fs::write(&self.gemini_config_file, content)
            .map_err(|e| format!("写入 gemini.json 失败: {}", e))
    }

    // ========================================================================
    // 站点管理
    // ========================================================================

    /// 获取站点
    pub fn get_site(&self, site_name: &str) -> Result<Option<GeminiSite>, String> {
        let config = self.read_config()?;
        Ok(config.get_site(site_name).cloned())
    }

    /// 获取所有站点
    pub fn get_all_sites(&self) -> Result<HashMap<String, GeminiSite>, String> {
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

        let site = GeminiSite::new(url, description);
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

    /// 更新站点配置
    pub fn update_site_config(
        &mut self,
        site_name: &str,
        base_url: Option<String>,
        model: Option<String>,
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

        site.update_timestamp();

        self.write_config(&config)
    }

    // ========================================================================
    // API Key 管理
    // ========================================================================

    /// 获取 API Keys
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
            return Err(format!(
                "API Key '{}' 已存在于站点 '{}'",
                key_name, site_name
            ));
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
            return Err(format!(
                "API Key '{}' 不存在于站点 '{}'",
                key_name, site_name
            ));
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
            return Err(format!(
                "API Key '{}' 不存在于站点 '{}'",
                key_name, site_name
            ));
        }

        self.write_config(&config)
    }

    // ========================================================================
    // 配置同步到 ~/.gemini/
    // ========================================================================

    /// 同步配置到 Gemini 官方配置文件
    pub fn sync_to_gemini(&self, active_config: &GeminiActiveConfig) -> Result<(), String> {
        // 确保 ~/.gemini 目录存在
        if !self.gemini_dir.exists() {
            fs::create_dir_all(&self.gemini_dir)
                .map_err(|e| format!("创建 .gemini 目录失败: {}", e))?;
        }

        // 同步到 .env
        self.sync_to_env(active_config)?;

        // settings.json 保留用户原有配置，这里不做修改

        Ok(())
    }

    /// 同步到 .env 文件
    fn sync_to_env(&self, active_config: &GeminiActiveConfig) -> Result<(), String> {
        let mut lines = Vec::new();

        // Base URL
        if let Some(ref base_url) = active_config.base_url {
            lines.push(format!("GOOGLE_GEMINI_BASE_URL={}", base_url));
        }

        // API Key
        lines.push(format!("GEMINI_API_KEY={}", active_config.api_key));

        // Model
        if let Some(ref model) = active_config.model {
            lines.push(format!("GEMINI_MODEL={}", model));
        }

        let content = lines.join("\n") + "\n";

        fs::write(&self.gemini_env_file, content)
            .map_err(|e| format!("写入 .env 失败: {}", e))
    }

    // ========================================================================
    // 辅助方法
    // ========================================================================

    /// 获取配置文件路径（用于备份等）
    pub fn get_config_file_path(&self) -> &PathBuf {
        &self.gemini_config_file
    }

    /// 获取 .env 路径
    pub fn get_env_file_path(&self) -> &PathBuf {
        &self.gemini_env_file
    }

    /// 获取 settings.json 路径
    pub fn get_settings_file_path(&self) -> &PathBuf {
        &self.gemini_settings_file
    }
}
