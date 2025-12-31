use clap::{Parser, Subcommand};

/// Coding Agent 配置管理CLI工具
#[derive(Parser)]
#[command(name = "ca-switch")]
#[command(author = "moguw <weiyiding0@gmail.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Coding Agent 配置管理CLI工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 查看当前状态
    Status,

    /// 导出配置
    Export {
        /// 要导出的配置类型
        #[arg(value_name = "TYPE")]
        config_type: ExportType,
    },
}

#[derive(Clone, Debug)]
pub enum ExportType {
    OpenCode,
}

impl std::str::FromStr for ExportType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "opencode" => Ok(ExportType::OpenCode),
            _ => Err(format!("不支持的配置类型: {}", s)),
        }
    }
}

impl std::fmt::Display for ExportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportType::OpenCode => write!(f, "opencode"),
        }
    }
}
