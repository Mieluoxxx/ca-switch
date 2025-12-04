// Claude 配置管理器
// 负责管理 ~/.cc-cli/claude.json 和同步到 ~/.claude/settings.json

use crate::config::models::{
    ClaudeActiveConfig, ClaudeConfig, ClaudeSite, VertexConfig,
};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Claude 配置管理器
pub struct ClaudeConfigManager {
    #[allow(dead_code)]
    config_dir: PathBuf,       // ~/.cc-cli
    claude_config_file: PathBuf, // ~/.cc-cli/claude.json
    #[allow(dead_code)]
    claude_dir: PathBuf,       // ~/.claude
    settings_file: PathBuf,    // ~/.claude/settings.json
}

impl ClaudeConfigManager {
    /// 创建新的 Claude 配置管理器
    pub fn new() -> Result<Self, String> {
        let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;

        let config_dir = home_dir.join(".cc-cli");
        let claude_config_file = config_dir.join("claude.json");
        let claude_dir = home_dir.join(".claude");
        let settings_file = claude_dir.join("settings.json");

        // 确保目录存在
        fs::create_dir_all(&config_dir).map_err(|e| format!("创建目录失败: {}", e))?;
        fs::create_dir_all(&claude_dir).map_err(|e| format!("创建目录失败: {}", e))?;

        Ok(Self {
            config_dir,
            claude_config_file,
            claude_dir,
            settings_file,
        })
    }

    // ========================================================================
    // 配置文件读写
    // ========================================================================

    /// 读取 claude.json
    pub fn read_config(&self) -> Result<ClaudeConfig, String> {
        if !self.claude_config_file.exists() {
            // 如果文件不存在，返回空配置
            return Ok(ClaudeConfig::new());
        }

        let content = fs::read_to_string(&self.claude_config_file)
            .map_err(|e| format!("读取配置文件失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析配置文件失败: {}", e))
    }

    /// 写入 claude.json
    pub fn write_config(&self, config: &ClaudeConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;

        fs::write(&self.claude_config_file, content)
            .map_err(|e| format!("写入配置文件失败: {}", e))
    }

    // ========================================================================
    // 站点管理
    // ========================================================================

    /// 获取所有站点
    pub fn get_all_sites(&self) -> Result<HashMap<String, ClaudeSite>, String> {
        let config = self.read_config()?;
        Ok(config.sites)
    }

    /// 获取单个站点
    pub fn get_site(&self, site_name: &str) -> Result<Option<ClaudeSite>, String> {
        let config = self.read_config()?;
        Ok(config.get_site(site_name).cloned())
    }

    /// 添加站点
    pub fn add_site(
        &self,
        site_name: String,
        url: String,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        // 检查站点是否已存在
        if config.sites.contains_key(&site_name) {
            return Err(format!("站点 '{}' 已存在", site_name));
        }

        let site = ClaudeSite::new(url, description);
        config.add_site(site_name, site);
        self.write_config(&config)
    }

    /// 更新站点元数据
    pub fn update_site_metadata(
        &self,
        site_name: &str,
        url: Option<String>,
        description: Option<String>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if let Some(new_url) = url {
            site.metadata.url = new_url;
        }

        if let Some(new_desc) = description {
            site.metadata.description = Some(new_desc);
        }

        site.update_timestamp();
        self.write_config(&config)
    }

    /// 更新站点配置
    pub fn update_site_config(
        &self,
        site_name: &str,
        base_url: Option<String>,
        model: Option<String>,
        vertex: Option<VertexConfig>,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if let Some(url) = base_url {
            site.config.base_url = Some(url);
        }

        if let Some(m) = model {
            site.config.model = Some(m);
        }

        if let Some(v) = vertex {
            site.config.vertex = v;
        }

        site.update_timestamp();
        self.write_config(&config)
    }

    /// 删除站点
    pub fn remove_site(&self, site_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        config
            .remove_site(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        self.write_config(&config)
    }

    // ========================================================================
    // Token 管理
    // ========================================================================

    /// 添加 token
    pub fn add_token(
        &self,
        site_name: &str,
        token_name: String,
        token: String,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        // 检查 token 是否已存在
        if site.tokens.contains_key(&token_name) {
            return Err(format!("Token '{}' 已存在于站点 '{}'", token_name, site_name));
        }

        site.add_token(token_name, token);
        self.write_config(&config)
    }

    /// 更新 token
    pub fn update_token(
        &self,
        site_name: &str,
        token_name: &str,
        new_token: String,
    ) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if !site.tokens.contains_key(token_name) {
            return Err(format!("Token '{}' 不存在于站点 '{}'", token_name, site_name));
        }

        site.add_token(token_name.to_string(), new_token);
        self.write_config(&config)
    }

    /// 删除 token
    pub fn remove_token(&self, site_name: &str, token_name: &str) -> Result<(), String> {
        let mut config = self.read_config()?;

        let site = config
            .get_site_mut(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        site.remove_token(token_name)
            .ok_or_else(|| format!("Token '{}' 不存在于站点 '{}'", token_name, site_name))?;

        self.write_config(&config)
    }

    /// 获取站点的所有 tokens
    #[allow(dead_code)]
    pub fn get_tokens(&self, site_name: &str) -> Result<HashMap<String, String>, String> {
        let config = self.read_config()?;

        let site = config
            .get_site(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        Ok(site.tokens.clone())
    }

    // ========================================================================
    // 配置同步到 ~/.claude/settings.json
    // ========================================================================

    /// 同步配置到 Claude Code 官方配置文件
    pub fn sync_to_settings(&self, active_config: &ClaudeActiveConfig) -> Result<(), String> {
        // 读取现有 settings.json（如果存在）
        let mut settings = if self.settings_file.exists() {
            let content = fs::read_to_string(&self.settings_file)
                .map_err(|e| format!("读取 settings.json 失败: {}", e))?;

            serde_json::from_str::<serde_json::Value>(&content)
                .unwrap_or_else(|_| serde_json::json!({}))
        } else {
            serde_json::json!({})
        };

        // 清理外层旧的认证字段（避免冲突）
        if let Some(obj) = settings.as_object_mut() {
            obj.remove("ANTHROPIC_AUTH_TOKEN");
            obj.remove("ANTHROPIC_BASE_URL");
            obj.remove("ANTHROPIC_VERTEX_BASE_URL");
            obj.remove("ANTHROPIC_VERTEX_PROJECT_ID");
            obj.remove("CLAUDE_CODE_USE_VERTEX");
            obj.remove("CLAUDE_CODE_SKIP_VERTEX_AUTH");
        }

        // 确保 env 对象存在
        if !settings.get("env").is_some() {
            settings["env"] = serde_json::json!({});
        }

        // 清理 env 内部的旧字段（避免模式切换时残留）
        if let Some(env_obj) = settings.get_mut("env").and_then(|v| v.as_object_mut()) {
            env_obj.remove("ANTHROPIC_AUTH_TOKEN");
            env_obj.remove("ANTHROPIC_BASE_URL");
            env_obj.remove("ANTHROPIC_VERTEX_BASE_URL");
            env_obj.remove("ANTHROPIC_VERTEX_PROJECT_ID");
            env_obj.remove("CLAUDE_CODE_USE_VERTEX");
            env_obj.remove("CLAUDE_CODE_SKIP_VERTEX_AUTH");
            env_obj.remove("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC");
        }

        // 构建新的 env 配置
        let mut new_env = serde_json::json!({
            "ANTHROPIC_AUTH_TOKEN": active_config.token,
        });

        // 根据是否启用 Vertex 决定使用哪个 Base URL
        if active_config.vertex.enabled {
            // Vertex 模式：只使用 ANTHROPIC_VERTEX_BASE_URL
            new_env["CLAUDE_CODE_USE_VERTEX"] = serde_json::json!("1");

            if let Some(ref project_id) = active_config.vertex.project_id {
                new_env["ANTHROPIC_VERTEX_PROJECT_ID"] = serde_json::json!(project_id);
            }

            if let Some(ref vertex_url) = active_config.vertex.base_url {
                new_env["ANTHROPIC_VERTEX_BASE_URL"] = serde_json::json!(vertex_url);
            }

            if active_config.vertex.skip_auth {
                new_env["CLAUDE_CODE_SKIP_VERTEX_AUTH"] = serde_json::json!("1");
            }

            // 添加 CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC
            new_env["CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"] = serde_json::json!("1");
        } else {
            // 普通模式：使用 ANTHROPIC_BASE_URL
            if let Some(ref base_url) = active_config.base_url {
                new_env["ANTHROPIC_BASE_URL"] = serde_json::json!(base_url);
            }
        }

        // 深度合并到 env 对象
        if let Some(env_obj) = settings.get_mut("env") {
            self.deep_merge(env_obj, &new_env);
        }

        // 写入文件
        let content = serde_json::to_string_pretty(&settings)
            .map_err(|e| format!("序列化 settings.json 失败: {}", e))?;

        fs::write(&self.settings_file, content)
            .map_err(|e| format!("写入 settings.json 失败: {}", e))
    }

    /// 深度合并 JSON 对象
    fn deep_merge(&self, target: &mut serde_json::Value, source: &serde_json::Value) {
        if let (Some(target_obj), Some(source_obj)) = (target.as_object_mut(), source.as_object()) {
            for (key, value) in source_obj {
                if let Some(target_value) = target_obj.get_mut(key) {
                    if target_value.is_object() && value.is_object() {
                        self.deep_merge(target_value, value);
                    } else {
                        *target_value = value.clone();
                    }
                } else {
                    target_obj.insert(key.clone(), value.clone());
                }
            }
        }
    }

    // ========================================================================
    // 辅助方法
    // ========================================================================

    /// 获取配置文件路径（用于备份等）
    #[allow(dead_code)]
    pub fn get_config_file_path(&self) -> &PathBuf {
        &self.claude_config_file
    }

    /// 获取 settings.json 路径
    #[allow(dead_code)]
    pub fn get_settings_file_path(&self) -> &PathBuf {
        &self.settings_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_merge() {
        let manager = ClaudeConfigManager::new().unwrap();

        let mut target = serde_json::json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": 3
            }
        });

        let source = serde_json::json!({
            "b": {
                "c": 20,
                "e": 4
            },
            "f": 5
        });

        manager.deep_merge(&mut target, &source);

        assert_eq!(target["a"], 1);
        assert_eq!(target["b"]["c"], 20);
        assert_eq!(target["b"]["d"], 3);
        assert_eq!(target["b"]["e"], 4);
        assert_eq!(target["f"], 5);
    }
}
