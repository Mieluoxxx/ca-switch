// OpenCode 配置管理器
// 负责管理 ~/.opcd/opencode.json 和同步到 ~/.opencode/opencode.json

use crate::config::models::{OpenCodeConfig, OpenCodeModelInfo, OpenCodeProvider};
use crate::config::ConfigError;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const SYNC_THEME: &str = "tokyonight";

pub struct OpenCodeConfigManager {
    config_dir: PathBuf,
    home_dir: PathBuf,
    home_json: PathBuf,
}

impl OpenCodeConfigManager {
    pub fn new(config_dir: PathBuf) -> Result<Self, ConfigError> {
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        let home_dir = dirs::home_dir()
            .ok_or_else(|| ConfigError::NotFound {
                name: "用户主目录".to_string(),
            })?
            .join(".opencode");

        let home_json = home_dir.join("opencode.json");

        Ok(Self {
            config_dir,
            home_dir,
            home_json,
        })
    }

    pub fn read_config(&self) -> Result<OpenCodeConfig, String> {
        let config_path = self.config_dir.join("opencode.json");
        if !config_path.exists() {
            return Ok(OpenCodeConfig::new());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取 opencode.json 失败: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("解析 opencode.json 失败: {}", e))
    }

    pub fn write_config(&self, config: &OpenCodeConfig) -> Result<(), String> {
        let config_path = self.config_dir.join("opencode.json");
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化 opencode.json 失败: {}", e))?;

        fs::write(&config_path, content).map_err(|e| format!("写入 opencode.json 失败: {}", e))
    }

    pub fn get_provider(&self, provider_name: &str) -> Result<Option<OpenCodeProvider>, String> {
        let config = self.read_config()?;
        Ok(config.get_provider(provider_name).cloned())
    }

    pub fn get_all_providers(&self) -> Result<HashMap<String, OpenCodeProvider>, String> {
        let config = self.read_config()?;
        Ok(config.providers)
    }

    pub fn add_provider(
        &mut self,
        provider_name: String,
        base_url: String,
        api_key: String,
        npm: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.get_provider(&provider_name).is_some() {
            return Err(format!("Provider '{}' 已存在", provider_name));
        }

        let provider =
            OpenCodeProvider::new(provider_name.clone(), base_url, api_key, npm, description);
        config.add_provider(provider_name, provider);

        self.write_config(&config)
    }

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
        if let Some(npm_val) = npm {
            provider.npm = Some(npm_val);
            provider.update_timestamp();
        }
        if let Some(desc) = description {
            provider.metadata.description = Some(desc);
            provider.update_timestamp();
        }

        self.write_config(&config)
    }

    pub fn delete_provider(&mut self, provider_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        if config.remove_provider(provider_name).is_none() {
            return Err(format!("Provider '{}' 不存在", provider_name));
        }

        self.write_config(&config)
    }

    pub fn get_models(
        &self,
        provider_name: &str,
    ) -> Result<HashMap<String, OpenCodeModelInfo>, String> {
        let config = self.read_config()?;
        let provider = config
            .get_provider(provider_name)
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        Ok(provider.models.clone())
    }

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

        if provider.get_model(&model_id).is_some() {
            return Err(format!(
                "模型 '{}' 已存在于 Provider '{}'",
                model_id, provider_name
            ));
        }

        provider.add_model(model_id, model_info);

        self.write_config(&config)
    }

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

    pub fn sync_multiple_providers_to_opencode(
        &self,
        provider_names: &[String],
    ) -> Result<(), String> {
        let config = self.read_config()?;
        sync_providers(&config, provider_names, &self.home_dir, &self.home_json)
    }

    pub fn sync_multiple_providers_to_project(
        &self,
        provider_names: &[String],
    ) -> Result<(), String> {
        let config = self.read_config()?;
        let (project_dir, project_json) = get_project_opencode_paths()?;
        sync_providers(&config, provider_names, &project_dir, &project_json)
    }
}

fn ensure_dir_exists(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(())
}

fn get_project_opencode_paths() -> Result<(PathBuf, PathBuf), String> {
    let project_dir = std::env::current_dir()
        .map_err(|e| format!("获取当前目录失败: {}", e))?
        .join(".opencode");
    let project_json = project_dir.join("opencode.json");
    Ok((project_dir, project_json))
}

fn sync_providers(
    config: &OpenCodeConfig,
    provider_names: &[String],
    dir: &PathBuf,
    json_path: &PathBuf,
) -> Result<(), String> {
    ensure_dir_exists(dir)?;
    sync_providers_to_file(config, provider_names, json_path)
}

fn sync_providers_to_file(
    config: &OpenCodeConfig,
    provider_names: &[String],
    target_path: &PathBuf,
) -> Result<(), String> {
    let providers_map: serde_json::Map<String, serde_json::Value> = provider_names
        .iter()
        .filter_map(|name| {
            config.get_provider(name).and_then(|provider| {
                serde_json::to_value(provider)
                    .ok()
                    .map(|value| (name.clone(), value))
            })
        })
        .collect();

    let sync_data = serde_json::json!({
        "$schema": "https://opencode.ai/config.json",
        "theme": SYNC_THEME,
        "autoupdate": false,
        "provider": providers_map,
        "tools": {
            "webfetch": true
        },
        "agent": {},
        "mcp": {}
    });

    let content = serde_json::to_string_pretty(&sync_data)
        .map_err(|e| format!("序列化同步数据失败: {}", e))?;

    backup_existing_file(target_path)?;

    fs::write(target_path, content).map_err(|e| format!("写入失败: {}", e))
}

fn backup_existing_file(target_path: &PathBuf) -> Result<(), String> {
    if target_path.exists() {
        let backup_path = target_path.with_extension("json.bak");
        fs::copy(target_path, &backup_path).map_err(|e| format!("备份文件失败: {}", e))?;
    }
    Ok(())
}
