use clap::{Parser, Subcommand, ValueEnum};

/// Coding Agent 配置管理CLI工具
#[derive(Parser)]
#[command(name = "opcd")]
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
        #[arg(value_enum)]
        config_type: ExportType,
    },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ExportType {
    #[value(name = "opencode")]
    OpenCode,
}
