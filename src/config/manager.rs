// 核心配置管理器
// 负责管理全局 config.json 和协调各供应商配置管理器

use crate::config::opencode_manager::OpenCodeConfigManager;
use crate::config::models::{
    OpenCodeActiveConfig, OpenCodeActiveReference,
    GlobalConfig,
};
use std::fs;
use std::path::PathBuf;

/// 核心配置管理器
pub struct ConfigManager {
    global_config_file: PathBuf, // ~/.ca-switch/config.json
    opencode_manager: OpenCodeConfigManager,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, String> {
        let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
        let config_dir = home_dir.join(".ca-switch");
        let global_config_file = config_dir.join("config.json");

        // 确保配置目录存在
        fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

        // 初始化 OpenCode 配置管理器
        let opencode_manager = OpenCodeConfigManager::new(config_dir.clone())?;

        Ok(Self {
            global_config_file,
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

    /// 应用多个 OpenCode Provider 配置到全局
    pub fn apply_multiple_opencode_to_global(&mut self, provider_names: &[String]) -> Result<(), String> {
        // 1. 验证所有 Provider 是否存在
        let opencode_config = self.opencode_manager.read_config()?;

        for provider_name in provider_names {
            if opencode_config.get_provider(provider_name).is_none() {
                return Err(format!("Provider '{}' 不存在", provider_name));
            }
        }

        // 2. 更新全局配置（记录第一个provider为激活状态）
        if let Some(first_provider) = provider_names.first() {
            let reference = OpenCodeActiveReference {
                provider: first_provider.clone(),
            };

            let mut global_config = self.read_global_config()?;
            global_config.active.opencode = Some(reference);
            global_config.update_timestamp();
            self.write_global_config(&global_config)?;
        }

        // 3. 同步所有Provider到 ~/.opencode/
        self.opencode_manager.sync_multiple_providers_to_opencode(provider_names)?;

        Ok(())
    }

    /// 应用多个 OpenCode Provider 配置到项目级
    pub fn apply_multiple_opencode_to_project(&mut self, provider_names: &[String]) -> Result<(), String> {
        // 1. 验证所有 Provider 是否存在
        let opencode_config = self.opencode_manager.read_config()?;

        for provider_name in provider_names {
            if opencode_config.get_provider(provider_name).is_none() {
                return Err(format!("Provider '{}' 不存在", provider_name));
            }
        }

        // 2. 同步所有Provider到项目 .opencode/
        self.opencode_manager.sync_multiple_providers_to_project(provider_names)?;

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
