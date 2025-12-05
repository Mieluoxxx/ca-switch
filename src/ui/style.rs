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

/// 显示启动 Banner
pub fn show_banner(version: &str, has_update: bool) {
    let banner = r#"
   ___  ___   ___ _    ___
  / __|/ __| / __| |  |_ _|
 | (__| (__  | (__| |__ | |
  \___|\___|  \___|____|___|
"#;

    let version_text = if has_update {
        format!("{} {}",
            style(format!("v{version}")).dim(),
            style("(有更新)").yellow()
        )
    } else {
        format!("{} {}",
            style(format!("v{version}")).dim(),
            style("(最新)").green()
        )
    };

    println!("\n{}", style(banner).cyan().bold());
    println!("  {}", style("Claude Code配置管理CLI工具").white());
    println!("  {version_text}\n");
}

/// 主菜单选项
#[derive(Debug, Clone, Copy)]
pub enum MainMenuChoice {
    Api,
    CodexApi,
    GeminiApi,
    OpenCodeApi,
    Backup,
    Exit,
}

impl fmt::Display for MainMenuChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MainMenuChoice::Api => write!(f, "📡 ClaudeCode"),
            MainMenuChoice::CodexApi => write!(f, "💻 Codex"),
            MainMenuChoice::GeminiApi => write!(f, "🌟 Gemini-cli"),
            MainMenuChoice::OpenCodeApi => write!(f, "🚀 OpenCode"),
            MainMenuChoice::Backup => write!(f, "🔄 Backup"),
            MainMenuChoice::Exit => write!(f, "🚪 Exit"),
        }
    }
}

/// 显示主菜单
pub fn show_main_menu() -> crate::error::Result<MainMenuChoice> {
    let choices = [
        MainMenuChoice::OpenCodeApi,
        MainMenuChoice::Api,
        MainMenuChoice::CodexApi,
        MainMenuChoice::GeminiApi,
        MainMenuChoice::Backup,
        MainMenuChoice::Exit,
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择功能模块")
        .items(&choices)
        .default(0)
        .interact()
        .map_err(|_| crate::error::CliError::UserCancelled)?;

    Ok(choices[selection])
}

/// API 菜单选项
#[derive(Debug, Clone, Copy)]
pub enum ApiMenuChoice {
    Switch,
    List,
    Apply,
    Add,
    Edit,
    Delete,
    DetectSite,
    DetectModel,
    Back,
}

impl fmt::Display for ApiMenuChoice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiMenuChoice::Switch => write!(f, "🔄 切换配置 - 切换API配置"),
            ApiMenuChoice::List => write!(f, "📋 查看配置 - 列出所有配置"),
            ApiMenuChoice::Apply => write!(f, "🚀 应用配置 - 应用到项目或全局"),
            ApiMenuChoice::Add => write!(f, "➕ 添加配置 - 添加新的API配置"),
            ApiMenuChoice::Edit => write!(f, "📝 编辑配置 - 修改现有配置"),
            ApiMenuChoice::Delete => write!(f, "❌ 删除配置 - 删除API配置"),
            ApiMenuChoice::DetectSite => write!(f, "🌐 站点检测 - 检测站点并获取模型列表"),
            ApiMenuChoice::DetectModel => write!(f, "🤖 模型检测 - 测试模型性能和可用性"),
            ApiMenuChoice::Back => write!(f, "⬅️  返回上一级菜单"),
        }
    }
}

/// 显示 API 菜单
pub fn show_api_menu(title: &str) -> crate::error::Result<ApiMenuChoice> {
    println!("\n{}", style(title).cyan().bold());
    println!("{}", style("═".repeat(40)).dim());

    let choices = [
        ApiMenuChoice::Switch,
        ApiMenuChoice::List,
        ApiMenuChoice::Apply,
        ApiMenuChoice::Add,
        ApiMenuChoice::Edit,
        ApiMenuChoice::Delete,
        ApiMenuChoice::DetectSite,
        ApiMenuChoice::DetectModel,
        ApiMenuChoice::Back,
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择操作")
        .items(&choices)
        .default(0)
        .interact()
        .map_err(|_| crate::error::CliError::UserCancelled)?;

    Ok(choices[selection])
}

/// 确认操作
pub fn confirm(message: &str, default: bool) -> crate::error::Result<bool> {
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .default(default)
        .interact()
        .map_err(|_| crate::error::CliError::UserCancelled)
}

/// 等待返回确认
#[allow(dead_code)]
pub fn wait_for_back_confirm(message: &str) -> crate::error::Result<()> {
    let items = vec!["⬅️  返回上一级菜单"];
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(message)
        .items(&items)
        .default(0)
        .interact()
        .map_err(|_| crate::error::CliError::UserCancelled)?;
    Ok(())
}

/// OpenCode 菜单选项 (去除 Switch 和 List)
#[derive(Debug, Clone, Copy)]
pub enum OpenCodeMenuChoice {
    Apply,
    Add,
    Edit,
    Delete,
    DetectSite,
    DetectModel,
    Back,
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
            OpenCodeMenuChoice::Back => write!(f, "⬅️  返回上一级菜单"),
        }
    }
}

/// 显示 OpenCode 专用菜单
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
        OpenCodeMenuChoice::Back,
    ];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("请选择操作")
        .items(&choices)
        .default(0)
        .interact()
        .map_err(|_| crate::error::CliError::UserCancelled)?;

    Ok(choices[selection])
}

/// 获取地区图标
#[allow(dead_code)]
pub fn get_region_icon(region_name: &str) -> &'static str {
    let lower_name = region_name.to_lowercase();
    if lower_name.contains("日本") || lower_name.contains("japan") {
        "🇯🇵"
    } else if lower_name.contains("新加坡") || lower_name.contains("singapore") {
        "🇸🇬"
    } else if lower_name.contains("美国") || lower_name.contains("usa") {
        "🇺🇸"
    } else if lower_name.contains("香港") || lower_name.contains("hongkong") {
        "🇭🇰"
    } else if lower_name.contains("大陆") || lower_name.contains("china") {
        "🇨🇳"
    } else {
        "🌍"
    }
}
