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
                MainMenuChoice::Exit => {
                    println!("\n👋 再见喵～");
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}
