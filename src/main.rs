mod cli;
mod config;
mod error;
mod tui;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use console::style;
use error::Result;

use crate::config::ConfigManager;
use crate::utils::{
    display_paths, print_section, show_error, show_info, show_success, show_warning,
};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Status) => show_status()?,
        Some(Commands::Export { .. }) => {
            export_opencode_config()?;
        }
        None => {
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
    print_section("📊 当前配置状态");

    let config_manager = ConfigManager::new()?;

    println!("\n{}", style("🚀 OpenCode 配置:").white().bold());
    match config_manager.get_active_opencode_config()? {
        Some(config) => print_config_details(&config),
        None => show_info("未配置 OpenCode"),
    }

    println!();
    Ok(())
}

fn print_config_details(config: &crate::config::OpenCodeActiveConfig) {
    println!(
        "  {} {}",
        style("Provider:").white(),
        style(&config.provider).cyan()
    );
    println!(
        "  {} {}",
        style("Base URL:").white(),
        style(&config.base_url).dim()
    );
    let model_list: Vec<&str> = config.models.keys().map(|s| s.as_str()).collect();
    println!(
        "  {} {}",
        style("可用模型:").white(),
        style(model_list.join(", ")).yellow()
    );
}

/// 导出 OpenCode 配置到当前目录
fn export_opencode_config() -> Result<()> {
    print_section("📤 导出 OpenCode 配置");

    let home_dir = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    let source_path = home_dir.join(".opencode").join("opencode.json");

    if !source_path.exists() {
        show_error("源配置文件不存在");
        show_info("请先切换配置以生成 ~/.opencode/opencode.json");
        return Ok(());
    }

    let current_dir = std::env::current_dir()?;
    let target_dir = current_dir.join(".opencode");
    let target_path = target_dir.join("opencode.json");

    display_paths("源文件:", &source_path);
    println!();
    display_paths("目标文件:", &target_path);
    println!();

    if target_path.exists() {
        show_warning("目标文件已存在，将被覆盖");
        println!();
    }

    std::fs::create_dir_all(&target_dir)?;
    std::fs::copy(&source_path, &target_path)?;

    show_success("配置已成功导出到当前目录！");
    println!();
    show_info(&format!("目标路径: {}", target_path.display()));
    println!();

    Ok(())
}
