mod cli;
mod commands;
mod config;
mod error;
mod ui;

use clap::Parser;
use cli::{Cli, Commands, ExportType};
use error::Result;
use ui::Menu;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Claude) => {
            let mut cmd = commands::ClaudeCommand::new()?;
            cmd.execute()?;
        }
        Some(Commands::Codex) => {
            let mut cmd = commands::CodexCommand::new()?;
            cmd.execute()?;
        }
        Some(Commands::Gemini) => {
            let mut cmd = commands::GeminiCommand::new()?;
            cmd.execute()?;
        }
        Some(Commands::OpenCode) => {
            let mut cmd = commands::OpenCodeCommand::new()?;
            cmd.execute()?;
        }
        Some(Commands::Backup) => {
            let mut cmd = commands::BackupCommand::new()?;
            cmd.execute().await?;
        }
        Some(Commands::Status) => {
            show_status()?;
        }
        Some(Commands::Export { config_type }) => {
            match config_type {
                ExportType::OpenCode => {
                    export_opencode_config()?;
                }
            }
        }
        None => {
            // 没有子命令时，显示交互式菜单
            let mut menu = Menu::new();
            menu.run().await?;
        }
    }

    Ok(())
}

/// 显示状态
fn show_status() -> Result<()> {
    use console::style;
    use config::ConfigManager;
    use ui::show_info;

    println!("\n{}", style("📊 当前配置状态").cyan().bold());
    println!("{}", style("═".repeat(40)).dim());

    let config_manager = ConfigManager::new()?;

    // 显示 Claude 配置
    println!("\n{}", style("🤖 Claude 配置:").white().bold());
    match config_manager.get_active_claude_config()? {
        Some(config) => {
            println!("  {} {}", style("站点:").white(), style(&config.site).cyan());
            println!("  {} {}", style("URL:").white(), style(&config.site_url).dim());
            println!("  {} {}", style("Token:").white(), style(&config.token_name).cyan());
            if let Some(ref base_url) = config.base_url {
                println!("  {} {}", style("Base URL:").white(), style(base_url).dim());
            }
            if let Some(ref model) = config.model {
                println!("  {} {}", style("Model:").white(), style(model).yellow());
            }
        }
        None => {
            show_info("未配置 Claude API");
        }
    }

    // 显示 Codex 配置
    println!("\n{}", style("💻 Codex 配置:").white().bold());
    match config_manager.get_active_codex_config()? {
        Some(config) => {
            println!("  {} {}", style("站点:").white(), style(&config.site).cyan());
            if let Some(ref base_url) = config.base_url {
                println!("  {} {}", style("Base URL:").white(), style(base_url).dim());
            }
            println!("  {} {}", style("API Key:").white(), style(&config.api_key_name).cyan());
            if let Some(ref model) = config.model {
                println!("  {} {}", style("Model:").white(), style(model).yellow());
            }
            if let Some(ref provider) = config.model_provider {
                println!("  {} {}", style("Model Provider:").white(), style(provider).green());
            }
        }
        None => {
            show_info("未配置 Codex API");
        }
    }

    // 显示 Gemini 配置
    println!("\n{}", style("🌟 Gemini 配置:").white().bold());
    match config_manager.get_active_gemini_config()? {
        Some(config) => {
            println!("  {} {}", style("站点:").white(), style(&config.site).cyan());
            if let Some(ref base_url) = config.base_url {
                println!("  {} {}", style("Base URL:").white(), style(base_url).dim());
            }
            println!("  {} {}", style("API Key:").white(), style(&config.api_key_name).cyan());
            if let Some(ref model) = config.model {
                println!("  {} {}", style("Model:").white(), style(model).yellow());
            }
        }
        None => {
            show_info("未配置 Gemini API");
        }
    }

    println!("\n{}", style("🚀 OpenCode 配置:").white().bold());
    match config_manager.get_active_opencode_config()? {
        Some(config) => {
            println!("  {} {}", style("主模型Provider:").white(), style(&config.main.provider).cyan());
            println!("  {} {}", style("主模型:").white(), style(&config.main.model).yellow());
            println!("  {} {}", style("轻量模型Provider:").white(), style(&config.small.provider).cyan());
            println!("  {} {}", style("轻量模型:").white(), style(&config.small.model).yellow());
        }
        None => {
            show_info("未配置 OpenCode");
        }
    }

    println!();
    Ok(())
}

/// 显示帮助
#[allow(dead_code)]
fn show_help() -> Result<()> {
    use console::style;

    println!("\n{}", style("❓ 帮助文档").cyan().bold());
    println!("{}", style("═".repeat(40)).dim());

    println!("\n{}", style("使用方法:").white().bold());
    println!("  cc [COMMAND]");

    println!("\n{}", style("可用命令:").white().bold());
    println!("  claude   Claude API 配置管理");
    println!("  codex    Codex API 配置管理");
    println!("  backup   备份与恢复");
    println!("  status   查看当前状态");
    println!("  help     显示帮助信息");

    println!("\n{}", style("不带任何参数运行时将进入交互式菜单").dim());

    println!("\n{}", style("更多信息:").white().bold());
    println!("  使用 'cc --help' 查看详细帮助");
    println!("  使用 'cc <COMMAND> --help' 查看子命令帮助");

    println!();
    Ok(())
}

/// 导出 OpenCode 配置到当前目录
fn export_opencode_config() -> Result<()> {
    use console::style;
    use ui::{show_error, show_info, show_success};

    println!("\n{}", style("📤 导出 OpenCode 配置").cyan().bold());
    println!("{}", style("═".repeat(40)).dim());
    println!();

    // 获取源文件路径 ($HOME/.opencode/opencode.json)
    let home_dir = dirs::home_dir().ok_or("无法获取用户主目录")?;
    let source_path = home_dir.join(".opencode").join("opencode.json");

    // 检查源文件是否存在
    if !source_path.exists() {
        show_error("源配置文件不存在");
        show_info("请先切换配置以生成 ~/.opencode/opencode.json");
        return Ok(());
    }

    // 获取目标文件路径 (当前目录/.opencode/opencode.json)
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("无法获取当前目录: {}", e))?;
    let target_dir = current_dir.join(".opencode");
    let target_path = target_dir.join("opencode.json");

    // 显示路径信息
    println!("{}", style("源文件:").white());
    println!("  {}", style(source_path.display()).cyan());
    println!();
    println!("{}", style("目标文件:").white());
    println!("  {}", style(target_path.display()).cyan());
    println!();

    // 如果目标文件已存在，显示警告
    if target_path.exists() {
        println!("{}", style("⚠️  目标文件已存在，将被覆盖").yellow());
        println!();
    }

    // 创建目标目录
    std::fs::create_dir_all(&target_dir)
        .map_err(|e| format!("创建目标目录失败: {}", e))?;

    // 复制文件
    std::fs::copy(&source_path, &target_path)
        .map_err(|e| format!("复制文件失败: {}", e))?;

    show_success("✨ 配置已成功导出到当前目录！");
    println!();
    show_info(&format!("目标路径: {}", target_path.display()));
    println!();

    Ok(())
}
