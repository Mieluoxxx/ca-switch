use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use std::fmt;

/// 显示成功消息
pub fn show_success(message: &str) {
    println!("{} {}", style("✨").green(), style(message).green());
}

/// 显示警告消息
pub fn show_warning(message: &str) {
    println!("{} {}", style("⚠️ ").yellow(), style(message).yellow());
}

/// 显示错误消息
pub fn show_error(message: &str) {
    println!("{} {}", style("❌").red(), style(message).red());
}

/// 显示信息消息
pub fn show_info(message: &str) {
    println!("{} {}", style("ℹ️ ").blue(), style(message).blue());
}

/// 确认操作
pub fn confirm(message: &str, default: bool) -> crate::error::Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(default)
        .interact()
        .map_err(|_| crate::error::CliError::UserCancelled)
}

/// OpenCode 菜单选项 (直接作为主菜单使用)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenCodeMenuChoice {
    Apply,
    Add,
    Edit,
    Delete,
    DetectSite,
    DetectModel,
    Backup,
    Exit,
}

impl fmt::Display for OpenCodeMenuChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpenCodeMenuChoice::Apply => write!(f, "🚀 应用配置 - 应用到项目或全局"),
            OpenCodeMenuChoice::Add => write!(f, "➕ 添加配置 - 添加新的API配置"),
            OpenCodeMenuChoice::Edit => write!(f, "📝 编辑配置 - 修改现有配置"),
            OpenCodeMenuChoice::Delete => write!(f, "❌ 删除配置 - 删除API配置"),
            OpenCodeMenuChoice::DetectSite => write!(f, "🌐 站点检测 - 检测站点并获取模型列表"),
            OpenCodeMenuChoice::DetectModel => write!(f, "🤖 模型检测 - 测试模型性能和可用性"),
            OpenCodeMenuChoice::Backup => write!(f, "🔄 备份管理 - 管理配置备份"),
            OpenCodeMenuChoice::Exit => write!(f, "🚪 退出程序"),
        }
    }
}

/// 显示 OpenCode 专用菜单 (现在作为主菜单使用)
pub fn show_opencode_menu(title: &str) -> crate::error::Result<OpenCodeMenuChoice> {
    println!("\n{}", style(title).cyan().bold());
    println!("{}", style("═".repeat(40)).dim());

    let choices = [
        OpenCodeMenuChoice::Apply,
        OpenCodeMenuChoice::Add,
        OpenCodeMenuChoice::Edit,
        OpenCodeMenuChoice::Delete,
        OpenCodeMenuChoice::DetectSite,
        OpenCodeMenuChoice::DetectModel,
        OpenCodeMenuChoice::Backup,
        OpenCodeMenuChoice::Exit,
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择操作")
        .items(&choices)
        .default(0)
        .interact()
        .map_err(|_| crate::error::CliError::UserCancelled)?;

    Ok(choices[selection])
}
