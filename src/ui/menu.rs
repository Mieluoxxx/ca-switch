use crate::commands::{BackupCommand, ClaudeCommand, CodexCommand, GeminiCommand, OpenCodeCommand};
use crate::error::Result;
use crate::ui::{show_banner, show_main_menu, MainMenuChoice};

/// 菜单管理器
pub struct Menu;

impl Menu {
    pub fn new() -> Self {
        Self
    }

    /// 运行交互式菜单
    pub async fn run(&mut self) -> Result<()> {
        // 显示 Banner
        show_banner(env!("CARGO_PKG_VERSION"), false);

        loop {
            match show_main_menu()? {
                MainMenuChoice::Api => {
                    let mut cmd = ClaudeCommand::new()?;
                    cmd.execute()?;
                }
                MainMenuChoice::CodexApi => {
                    let mut cmd = CodexCommand::new()?;
                    cmd.execute()?;
                }
                MainMenuChoice::GeminiApi => {
                    let mut cmd = GeminiCommand::new()?;
                    cmd.execute()?;
                }
                MainMenuChoice::OpenCodeApi => {
                    let mut cmd = OpenCodeCommand::new()?;
                    cmd.execute()?;
                }
                MainMenuChoice::Backup => {
                    let mut cmd = BackupCommand::new()?;
                    cmd.execute().await?;
                }
                MainMenuChoice::Status => {
                    self.show_status()?;
                }
                MainMenuChoice::Help => {
                    self.show_help()?;
                }
                MainMenuChoice::Exit => {
                    println!("\n👋 再见喵～");
                    break;
                }
            }
        }

        Ok(())
    }

    /// 显示状态
    fn show_status(&self) -> Result<()> {
        use crate::config::ConfigManager;
        use crate::ui::{show_info, wait_for_back_confirm};
        use console::style;

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
        wait_for_back_confirm("查看完成")?;

        Ok(())
    }

    /// 显示帮助
    fn show_help(&self) -> Result<()> {
        use crate::ui::wait_for_back_confirm;
        use console::style;

        println!("\n{}", style("❓ 帮助文档").cyan().bold());
        println!("{}", style("═".repeat(40)).dim());

        println!("\n{}", style("📡 Claude API 管理:").white().bold());
        println!("  • 切换不同的 API 配置");
        println!("  • 查看所有可用配置");
        println!("  • 添加、编辑、删除配置");
        println!("  • 管理通知和 YOLO 模式");

        println!("\n{}", style("💻 Codex API 管理:").white().bold());
        println!("  • 管理 Codex API 配置");
        println!("  • 支持多种 AI 提供商");

        println!("\n{}", style("🌟 Gemini API 管理:").white().bold());
        println!("  • 管理 Gemini CLI 配置");
        println!("  • 支持多个 API Key");

        println!("\n{}", style("🚀 OpenCode API 管理:").white().bold());
        println!("  • 管理 OpenCode 配置");
        println!("  • 支持多个 Provider");
        println!("  • 支持主模型和轻量模型配置");

        println!("\n{}", style("🔄 备份与恢复:").white().bold());
        println!("  • 备份配置到云端 (WebDAV)");
        println!("  • 从云端恢复配置");
        println!("  • 查看备份状态");

        println!("\n{}", style("📊 状态查看:").white().bold());
        println!("  • 查看当前 API 配置状态");

        println!("\n{}", style("💡 提示:").yellow().bold());
        println!("  • 使用方向键和回车键进行选择");
        println!("  • 按 Ctrl+C 可以退出");

        println!();
        wait_for_back_confirm("查看完成")?;

        Ok(())
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
