use clap::{Parser, Subcommand};

/// Claude Code配置管理CLI工具
#[derive(Parser)]
#[command(name = "cc")]
#[command(author = "cjh0 <your-email@example.com>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Claude Code配置管理CLI工具", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Claude API 配置管理
    Api,

    /// Codex API 配置管理
    Codex,

    /// Gemini CLI 配置管理
    Gemini,

    /// OpenCode 配置管理
    OpenCode,

    /// 备份与恢复
    Backup,

    /// 查看当前状态
    Status,
}
