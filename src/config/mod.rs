// Configuration module
// 配置管理模块

pub mod detector;
pub mod manager;
pub mod mcp_manager;
pub mod models;
pub mod opencode_manager;

pub use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("文件读写错误: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON 解析错误: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },

    #[error("配置项不存在: {name}")]
    NotFound { name: String },

    #[error("配置项已存在: {name}")]
    AlreadyExists { name: String },

    #[error("配置验证失败: {message}")]
    Validation { message: String },

    #[error("未知错误: {message}")]
    Unknown { message: String },
}

impl From<String> for ConfigError {
    fn from(message: String) -> Self {
        ConfigError::Unknown { message }
    }
}

impl From<&str> for ConfigError {
    fn from(message: &str) -> Self {
        ConfigError::Unknown {
            message: message.to_string(),
        }
    }
}

// Re-export commonly used items
pub use detector::*;
pub use manager::*;
pub use models::*;
// McpConfigManager 通过 manager.rs 使用
