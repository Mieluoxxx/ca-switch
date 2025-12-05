// 核心配置管理器
// 负责管理全局 config.json 和协调各供应商配置管理器

use crate::config::claude_manager::ClaudeConfigManager;
use crate::config::codex_manager::CodexConfigManager;
use crate::config::gemini_manager::GeminiConfigManager;
use crate::config::opencode_manager::OpenCodeConfigManager;
use crate::config::models::{
    ClaudeActiveConfig, ClaudeActiveReference, CodexActiveConfig, CodexActiveReference,
    GeminiActiveConfig, GeminiActiveReference, OpenCodeActiveConfig, OpenCodeActiveReference,
    GlobalConfig,
};
use std::fs;
use std::path::PathBuf;

/// 核心配置管理器
pub struct ConfigManager {
    global_config_file: PathBuf, // ~/.cc-cli/config.json
    claude_manager: ClaudeConfigManager,
    codex_manager: CodexConfigManager,
    gemini_manager: GeminiConfigManager,
    opencode_manager: OpenCodeConfigManager,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, String> {
        let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let config_dir = home_dir.join(".cc-cli");
        let global_config_file = config_dir.join("config.json");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

        // 初始化供应商配置管理器
        let claude_manager = ClaudeConfigManager::new()?;
        let codex_manager = CodexConfigManager::new(config_dir.clone())?;
        let gemini_manager = GeminiConfigManager::new(config_dir.clone())?;
        let opencode_manager = OpenCodeConfigManager::new(config_dir.clone())?;

        Ok(Self {
            global_config_file,
            claude_manager,
            codex_manager,
            gemini_manager,
            opencode_manager,
        })
    }

    // ========================================================================
    // 全局配置管理 (config.json)
    // ========================================================================

    /// 读取全局配置
    pub fn read_global_config(&self) -> Result<GlobalConfig, String> {
        if !self.global_config_file.exists() {
            // 如果文件不存在，返回新配置
            return Ok(GlobalConfig::new());
        }

        let content = fs::read_to_string(&self.global_config_file)
            .map_err(|e| format!("读取全局配置失败: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("解析全局配置失败: {}", e))
    }

    /// 写入全局配置
    pub fn write_global_config(&self, config: &GlobalConfig) -> Result<(), String> {
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("序列化全局配置失败: {}", e))?;

        fs::write(&self.global_config_file, content)
            .map_err(|e| format!("写入全局配置失败: {}", e))
    }

    // ========================================================================
    // Claude 配置管理
    // ========================================================================

    /// 获取 Claude 配置管理器的引用
    pub fn claude(&self) -> &ClaudeConfigManager {
        &self.claude_manager
    }

    /// 获取 Claude 配置管理器的可变引用
    pub fn claude_mut(&mut self) -> &mut ClaudeConfigManager {
        &mut self.claude_manager
    }

    /// 获取当前激活的 Claude 配置（完整配置）
    pub fn get_active_claude_config(&self) -> Result<Option<ClaudeActiveConfig>, String> {
        // 1. 读取全局配置获取引用
        let global_config = self.read_global_config()?;

        let reference = match global_config.active.claude {
            Some(ref r) => r,
            None => return Ok(None),
        };

        // 2. 从 claude.json 获取完整站点配置
        let site = match self.claude_manager.get_site(&reference.site)? {
            Some(s) => s,
            None => return Err(format!("站点 '{}' 不存在", reference.site)),
        };

        // 3. 构建完整的激活配置
        let active_config = ClaudeActiveConfig::from_reference(reference, &site)?;

        Ok(Some(active_config))
    }

    /// 切换 Claude 配置
    pub fn switch_claude_config(
        &mut self,
        site_name: &str,
        token_name: &str,
    ) -> Result<(), String> {
        // 1. 验证站点和 token 是否存在
        let site = self
            .claude_manager
            .get_site(site_name)?
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if !site.tokens.contains_key(token_name) {
            return Err(format!(
                "Token '{}' 不存在于站点 '{}'",
                token_name, site_name
            ));
        }

        // 2. 更新全局配置中的引用
        let mut global_config = self.read_global_config()?;

        global_config.active.claude = Some(ClaudeActiveReference {
            site: site_name.to_string(),
            token_name: token_name.to_string(),
        });

        global_config.update_timestamp();
        self.write_global_config(&global_config)?;

        // 3. 同步到 ~/.claude/settings.json
        let active_config = self
            .get_active_claude_config()?
            .ok_or("无法获取激活的 Claude 配置")?;

        self.claude_manager.sync_to_settings(&active_config)?;

        Ok(())
    }

    // ========================================================================
    // Codex 配置管理
    // ========================================================================

    /// 获取 Codex 配置管理器（不可变）
    pub fn codex(&self) -> &CodexConfigManager {
        &self.codex_manager
    }

    /// 获取 Codex 配置管理器（可变）
    pub fn codex_mut(&mut self) -> &mut CodexConfigManager {
        &mut self.codex_manager
    }

    /// 获取当前激活的 Codex 配置
    pub fn get_active_codex_config(&self) -> Result<Option<CodexActiveConfig>, String> {
        let global_config = self.read_global_config()?;

        // 获取 Codex 引用
        let reference = match global_config.active.codex {
            Some(ref r) => r,
            None => return Ok(None),
        };

        // 从 codex.json 获取站点信息
        let site = self.codex_manager.get_site(&reference.site)?;
        let site = match site {
            Some(s) => s,
            None => {
                return Err(format!(
                    "引用的 Codex 站点 '{}' 不存在于 codex.json 中",
                    reference.site
                ))
            }
        };

        // 构建完整的运行时配置
        CodexActiveConfig::from_reference(reference, &site).map(Some)
    }

    /// 切换 Codex 配置
    pub fn switch_codex_config(
        &mut self,
        site_name: &str,
        api_key_name: &str,
    ) -> Result<(), String> {
        // 1. 验证站点和 API Key 是否存在
        let site = self
            .codex_manager
            .get_site(site_name)?
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if site.get_api_key(api_key_name).is_none() {
            return Err(format!(
                "API Key '{}' 不存在于站点 '{}'",
                api_key_name, site_name
            ));
        }

        // 2. 更新全局配置中的引用
        let mut global_config = self.read_global_config()?;

        global_config.active.codex = Some(CodexActiveReference {
            site: site_name.to_string(),
            api_key_name: api_key_name.to_string(),
        });

        global_config.update_timestamp();
        self.write_global_config(&global_config)?;

        // 3. 同步到 ~/.codex/
        let active_config = self
            .get_active_codex_config()?
            .ok_or("无法获取激活的 Codex 配置")?;

        self.codex_manager.sync_to_codex(&active_config)?;

        Ok(())
    }

    // ========================================================================
    // Gemini 配置管理
    // ========================================================================

    /// 获取 Gemini 配置管理器引用
    pub fn gemini(&self) -> &GeminiConfigManager {
        &self.gemini_manager
    }

    /// 获取 Gemini 配置管理器可变引用
    pub fn gemini_mut(&mut self) -> &mut GeminiConfigManager {
        &mut self.gemini_manager
    }

    /// 获取当前激活的 Gemini 配置
    pub fn get_active_gemini_config(&self) -> Result<Option<GeminiActiveConfig>, String> {
        let global_config = self.read_global_config()?;

        if let Some(ref reference) = global_config.active.gemini {
            // 从 gemini.json 读取站点配置
            let gemini_config = self.gemini_manager.read_config()?;
            let site = gemini_config
                .get_site(&reference.site)
                .ok_or_else(|| format!("站点 '{}' 不存在于 gemini.json", reference.site))?;

            // 构建完整配置
            let active_config = GeminiActiveConfig::from_reference(reference, site)?;
            Ok(Some(active_config))
        } else {
            Ok(None)
        }
    }

    /// 切换 Gemini 配置
    pub fn switch_gemini_config(
        &mut self,
        site_name: &str,
        api_key_name: &str,
    ) -> Result<(), String> {
        // 验证站点和 API Key 是否存在
        let gemini_config = self.gemini_manager.read_config()?;
        let site = gemini_config
            .get_site(site_name)
            .ok_or_else(|| format!("站点 '{}' 不存在", site_name))?;

        if site.get_api_key(api_key_name).is_none() {
            return Err(format!(
                "API Key '{}' 不存在于站点 '{}'",
                api_key_name, site_name
            ));
        }

        // 创建激活引用
        let reference = GeminiActiveReference {
            site: site_name.to_string(),
            api_key_name: api_key_name.to_string(),
        };

        // 更新全局配置
        let mut global_config = self.read_global_config()?;
        global_config.active.gemini = Some(reference.clone());
        global_config.update_timestamp();
        self.write_global_config(&global_config)?;

        // 构建完整配置并同步到 ~/.gemini/
        let active_config = GeminiActiveConfig::from_reference(&reference, site)?;
        self.gemini_manager.sync_to_gemini(&active_config)?;

        Ok(())
    }

    // ========================================================================
    // OpenCode 配置管理
    // ========================================================================

    /// 获取 OpenCode 配置管理器引用
    pub fn opencode(&self) -> &OpenCodeConfigManager {
        &self.opencode_manager
    }

    /// 获取 OpenCode 配置管理器可变引用
    pub fn opencode_mut(&mut self) -> &mut OpenCodeConfigManager {
        &mut self.opencode_manager
    }

    /// 获取当前激活的 OpenCode 配置
    pub fn get_active_opencode_config(&self) -> Result<Option<OpenCodeActiveConfig>, String> {
        let global_config = self.read_global_config()?;

        if let Some(ref reference) = global_config.active.opencode {
            // 从 opencode.json 读取 Provider 配置
            let opencode_config = self.opencode_manager.read_config()?;

            // 构建完整配置
            let active_config = OpenCodeActiveConfig::from_reference(reference, &opencode_config)?;
            Ok(Some(active_config))
        } else {
            Ok(None)
        }
    }

    /// 切换 OpenCode 配置(简化版:只需指定Provider)
    pub fn switch_opencode_config(&mut self, provider: &str) -> Result<(), String> {
        // 1. 验证 Provider 是否存在
        let opencode_config = self.opencode_manager.read_config()?;

        if opencode_config.get_provider(provider).is_none() {
            return Err(format!("Provider '{}' 不存在", provider));
        }

        // 2. 创建激活引用
        let reference = OpenCodeActiveReference {
            provider: provider.to_string(),
        };

        // 3. 更新全局配置
        let mut global_config = self.read_global_config()?;
        global_config.active.opencode = Some(reference.clone());
        global_config.update_timestamp();
        self.write_global_config(&global_config)?;

        // 4. 构建完整配置并同步到 ~/.opencode/
        let active_config = OpenCodeActiveConfig::from_reference(&reference, &opencode_config)?;
        self.opencode_manager.sync_to_opencode(&active_config)?;

        Ok(())
    }

    /// 应用 OpenCode 配置到项目级
    pub fn apply_opencode_to_project(&mut self, provider: &str) -> Result<(), String> {
        // 1. 验证 Provider 是否存在
        let opencode_config = self.opencode_manager.read_config()?;

        if opencode_config.get_provider(provider).is_none() {
            return Err(format!("Provider '{}' 不存在", provider));
        }

        // 2. 创建激活引用
        let reference = OpenCodeActiveReference {
            provider: provider.to_string(),
        };

        // 3. 构建完整配置并同步到项目 .opencode/
        let active_config = OpenCodeActiveConfig::from_reference(&reference, &opencode_config)?;
        self.opencode_manager.sync_to_project(&active_config)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_manager_creation() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_global_config_read_write() {
        let manager = ConfigManager::new().unwrap();
        let config = GlobalConfig::new();

        let result = manager.write_global_config(&config);
        assert!(result.is_ok());

        let read_config = manager.read_global_config().unwrap();
        assert_eq!(read_config.version, "3.0.0");
    }
}
