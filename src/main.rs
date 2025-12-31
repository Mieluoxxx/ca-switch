mod cli;
mod commands;
mod config;
mod error;
mod tui;
mod utils;

use clap::Parser;
use cli::{Cli, Commands, ExportType};
use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
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
            // 没有子命令时，启动 TUI 界面
            if let Err(e) = tui::run() {
                eprintln!("TUI 错误: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// 显示状态
fn show_status() -> Result<()> {
    use console::style;
    use config::ConfigManager;
    use utils::show_info;

    println!("\n{}", style("📊 当前配置状态").cyan().bold());
    println!("{}", style("═".repeat(40)).dim());

    let config_manager = ConfigManager::new()?;

    println!("\n{}", style("🚀 OpenCode 配置:").white().bold());
    match config_manager.get_active_opencode_config()? {
        Some(config) => {
            println!("  {} {}", style("Provider:").white(), style(&config.provider).cyan());
            println!("  {} {}", style("Base URL:").white(), style(&config.base_url).dim());
            let model_list: Vec<&str> = config.models.keys().map(|s| s.as_str()).collect();
            println!("  {} {}", style("可用模型:").white(), style(model_list.join(", ")).yellow());
        }
        None => {
            show_info("未配置 OpenCode");
        }
    }

    println!();
    Ok(())
}

/// 导出 OpenCode 配置到当前目录
fn export_opencode_config() -> Result<()> {
    use console::style;
    use utils::{show_error, show_info, show_success};

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
