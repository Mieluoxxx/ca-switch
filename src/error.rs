use thiserror::Error;

/// CLI 应用的错误类型
#[derive(Error, Debug)]
pub enum CliError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP 请求错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("交互式输入错误: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("WebDAV 错误: {0}")]
    WebDav(String),

    #[error("用户取消操作")]
    UserCancelled,

    #[error("{0}")]
    #[allow(dead_code)]
    Custom(String),
}

// 实现 From<String> 转换，方便新架构使用
impl From<String> for CliError {
    fn from(s: String) -> Self {
        CliError::Custom(s)
    }
}

// 实现 From<&str> 转换
impl From<&str> for CliError {
    fn from(s: &str) -> Self {
        CliError::Custom(s.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
