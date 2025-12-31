use crate::commands::{BackupCommand, OpenCodeCommand};
use crate::error::Result;
use crate::ui::{show_banner, OpenCodeMenuChoice};

/// 菜单管理器
pub struct Menu;

impl Menu {
    pub fn new() -> Self {
        Self
    }

    /// 运行交互式菜单
    /// 直接进入 OpenCode 配置管理，无需先选择功能模块
    pub async fn run(&mut self) -> Result<()> {
        // 显示 Banner
        show_banner(env!("CARGO_PKG_VERSION"), false);

        // 直接进入 OpenCode 配置管理
        loop {
            let mut cmd = OpenCodeCommand::new()?;
            match cmd.execute_with_menu()? {
                OpenCodeMenuChoice::Backup => {
                    let mut backup_cmd = BackupCommand::new()?;
                    backup_cmd.execute().await?;
                }
                OpenCodeMenuChoice::Exit => {
                    println!("\n👋 再见喵～");
                    break;
                }
                _ => {}
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
