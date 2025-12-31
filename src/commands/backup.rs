use crate::error::Result;
use crate::config::file_manager::FileManager;
use crate::ui::{show_error, show_info, show_success, show_warning};
use crate::config::webdav::WebDAVClient;
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect, Select};

/// 备份类别
#[derive(Debug, Clone)]
pub struct BackupCategory {
    pub name: String,
    pub value: String,
    pub checked: bool,
}

impl BackupCategory {
    fn new(name: impl Into<String>, value: impl Into<String>, checked: bool) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            checked,
        }
    }
}

/// 备份命令
pub struct BackupCommand {
    file_manager: FileManager,
    webdav_client: WebDAVClient,
}

impl BackupCommand {
    pub fn new() -> Result<Self> {
        Ok(Self {
            file_manager: FileManager::new()?,
            webdav_client: WebDAVClient::new()?,
        })
    }

    /// 执行备份命令
    pub async fn execute(&mut self) -> Result<()> {
        loop {
            let choice = self.show_backup_menu()?;

            match choice.as_str() {
                "backup" => {
                    if let Err(e) = self.handle_backup().await {
                        show_error(&format!("备份失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                "restore" => {
                    if let Err(e) = self.handle_restore().await {
                        show_error(&format!("恢复失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                "status" => {
                    if let Err(e) = self.handle_status().await {
                        show_error(&format!("获取状态失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                "config" => {
                    if let Err(e) = self.handle_config().await {
                        show_error(&format!("配置失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                "back" => break,
                _ => {}
            }
        }

        Ok(())
    }

    /// 显示备份菜单
    fn show_backup_menu(&self) -> Result<String> {
        println!("\n{}", style("🔄 备份与恢复").cyan().bold());
        println!("{}", style("═".repeat(40)).dim());

        let items = vec![
            "📤 手动备份 - 选择配置进行备份",
            "📥 恢复数据 - 从云端存储恢复配置",
            "📊 备份状态 - 查看备份历史和状态",
            "⚙️  WebDAV配置 - 配置云端存储",
            "⬅️  返回上一级菜单",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择操作")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| crate::error::CliError::UserCancelled)?;

        let choice = match selection {
            0 => "backup",
            1 => "restore",
            2 => "status",
            3 => "config",
            4 => "back",
            _ => "back",
        };

        Ok(choice.to_string())
    }

    /// 处理手动备份
    async fn handle_backup(&mut self) -> Result<()> {
        println!("\n{}", style("📤 配置备份向导").cyan().bold());
        println!();

        // 选择备份类别
        let categories = self.select_backup_categories()?;

        if categories.is_empty() {
            show_info("未选择任何配置类别，备份已取消");
            return Ok(());
        }

        // 确认备份
        let confirmed = self.confirm_backup(&categories)?;

        if !confirmed {
            show_info("用户取消备份");
            return Ok(());
        }

        // 初始化 WebDAV 客户端
        show_info("🔌 初始化 WebDAV 连接...");
        self.webdav_client.initialize().await?;

        println!();
        show_info(&format!("📦 开始备份 {} 个配置类别...", categories.len()));
        println!();

        let mut success_count = 0;
        let mut fail_count = 0;

        // 执行备份
        for category in &categories {
            match self.backup_category(category).await {
                Ok(_) => success_count += 1,
                Err(e) => {
                    show_error(&format!("备份 {category} 失败: {e}"));
                    fail_count += 1;
                }
            }
        }

        println!();
        println!("{}", style("═".repeat(40)).dim());
        println!("{}", style("📊 备份完成统计").white().bold());
        println!();
        println!("  {} {} 个配置类别", style("✅ 成功:").green(), success_count);
        if fail_count > 0 {
            println!("  {} {} 个配置类别", style("❌ 失败:").red(), fail_count);
        }
        println!();

        self.wait_for_back()?;

        Ok(())
    }

    /// 备份单个类别
    async fn backup_category(&mut self, category: &str) -> Result<()> {
        let category_name = match category {
            "ccCli" => "CA-Switch配置",
            "opencode" => "OpenCode配置",
            _ => category,
        };

        show_info(&format!("📦 正在收集 {category_name} 的文件..."));

        // 收集备份数据
        let backup_data = self.file_manager.collect_backup_data(category).await?;

        // 生成文件名
        let file_name = format!(
            "{}-{}.json",
            category,
            chrono::Local::now().format("%Y-%m-%d-%H-%M-%S")
        );

        // 序列化为 JSON
        let json_data = serde_json::to_value(&backup_data)?;

        // 上传到 WebDAV
        self.webdav_client.upload_backup(&file_name, &json_data).await?;

        show_success(&format!(
            "✅ {} 备份成功 ({} 个文件, {})",
            category_name,
            backup_data.metadata.total_files,
            self.file_manager.format_file_size(backup_data.metadata.total_size)
        ));

        Ok(())
    }

    /// 选择备份类别
    fn select_backup_categories(&self) -> Result<Vec<String>> {
        let categories = vec![
            BackupCategory::new(
                "🔧 CA-Switch配置 (.ca-switch/)",
                "ccCli",
                true,
            ),
            BackupCategory::new(
                "🚀 OpenCode配置 (opencode.json)",
                "opencode",
                false,
            ),
        ];

        let items: Vec<String> = categories.iter().map(|c| c.name.clone()).collect();
        let defaults: Vec<bool> = categories.iter().map(|c| c.checked).collect();

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择要备份的配置类别（空格选择，回车确认）")
            .items(&items)
            .defaults(&defaults)
            .interact()
            .map_err(|_| crate::error::CliError::UserCancelled)?;

        let selected: Vec<String> = selections
            .into_iter()
            .map(|i| categories[i].value.clone())
            .collect();

        Ok(selected)
    }

    /// 确认备份
    fn confirm_backup(&self, categories: &[String]) -> Result<bool> {
        println!("\n{}", style("📋 备份信息确认").white());
        println!("{}", style("─".repeat(40)).dim());

        for category in categories {
            let display = match category.as_str() {
                "ccCli" => "🔧 CA-Switch配置",
                "opencode" => "🚀 OpenCode配置",
                _ => category,
            };
            println!("  ✓ {display}");
        }

        println!();

        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("确认执行备份")
            .default(true)
            .interact()
            .map_err(|_| crate::error::CliError::UserCancelled)
    }

    /// 处理恢复数据
    async fn handle_restore(&mut self) -> Result<()> {
        println!("\n{}", style("📥 数据恢复向导").cyan().bold());
        println!();

        // 初始化 WebDAV 客户端
        show_info("🔌 连接到 WebDAV 服务器...");
        self.webdav_client.initialize().await?;

        // 获取备份列表
        let backups = self.webdav_client.list_backups().await?;

        if backups.is_empty() {
            show_warning("云端没有找到任何备份文件");
            self.wait_for_back()?;
            return Ok(());
        }

        println!();
        show_info(&format!("找到 {} 个备份文件", backups.len()));

        // TODO: 实现备份文件选择和恢复逻辑
        show_info("完整的恢复功能正在开发中...");
        show_info("当前已支持：列出远程备份文件");

        self.wait_for_back()?;

        Ok(())
    }

    /// 处理备份状态
    async fn handle_status(&mut self) -> Result<()> {
        println!("\n{}", style("📊 备份状态报告").cyan().bold());
        println!();

        // 显示本地配置文件状态
        println!("{}", style("🔍 本地配置文件状态：").white().bold());
        println!();

        let categories = vec!["ccCli", "opencode"];

        for category in categories {
            match self.file_manager.check_category_files(category).await {
                Ok(result) => {
                    let status_icon = if result.total_exists == result.total_count {
                        "✅"
                    } else if result.total_exists > 0 {
                        "⚠️"
                    } else {
                        "❌"
                    };

                    println!(
                        "{} {} ({}/{})",
                        status_icon,
                        style(&result.name).white(),
                        result.total_exists,
                        result.total_count
                    );

                    // 显示文件详情
                    for (name, info) in &result.files {
                        let icon = if info.exists { "📄" } else { "❌" };
                        let size = if info.exists {
                            self.file_manager.format_file_size(info.size)
                        } else {
                            "不存在".to_string()
                        };
                        println!("  {} {} ({})", icon, style(name).dim(), style(size).dim());
                    }

                    // 显示目录详情
                    for (name, info) in &result.directories {
                        let icon = if info.exists { "📁" } else { "❌" };
                        let count = if info.exists {
                            format!("{} 个文件", info.file_count)
                        } else {
                            "不存在".to_string()
                        };
                        println!("  {} {}/ ({})", icon, style(name).dim(), style(count).dim());
                    }

                    println!();
                }
                Err(e) => {
                    println!("❌ {category} 检查失败: {e}");
                }
            }
        }

        // 显示云端存储状态
        println!("{}", style("☁️  云端存储状态：").white().bold());
        println!();

        if let Some((url, username, server_type)) = self.webdav_client.get_server_info() {
            println!("  {} {}", style("类型:").dim(), style(server_type).white());
            println!("  {} {}", style("地址:").dim(), style(url).white());
            println!("  {} {}", style("用户:").dim(), style(username).white());

            match self.webdav_client.test_connection().await {
                Ok(_) => {
                    println!("  {} {}", style("状态:").dim(), style("✅ 已连接").green());
                }
                Err(_) => {
                    println!("  {} {}", style("状态:").dim(), style("❌ 连接失败").red());
                }
            }
        } else {
            show_info("未配置 WebDAV");
            println!("  {} 使用 '⚙️  WebDAV配置' 菜单配置云端存储", style("提示:").dim());
        }

        println!();
        self.wait_for_back()?;

        Ok(())
    }

    /// 处理 WebDAV 配置
    async fn handle_config(&mut self) -> Result<()> {
        println!("\n{}", style("⚙️  WebDAV 配置管理").cyan().bold());
        println!();

        let items = vec![
            "1. 🔧 重新配置 WebDAV",
            "2. 🧪 测试连接",
            "3. 🗑️  清除配置",
            "4. ⬅️  返回上一级菜单",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择操作")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| crate::error::CliError::UserCancelled)?;

        match selection {
            0 => {
                // 重新配置
                self.webdav_client.initialize().await?;
            }
            1 => {
                // 测试连接
                show_info("🧪 测试 WebDAV 连接...");
                match self.webdav_client.test_connection().await {
                    Ok(_) => show_success("✅ WebDAV 连接正常"),
                    Err(e) => show_error(&format!("❌ WebDAV 连接失败: {e}")),
                }
            }
            2 => {
                // 清除配置
                if Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("确认清除 WebDAV 配置？")
                    .default(false)
                    .interact()?
                {
                    self.webdav_client.clear_config().await?;
                }
            }
            _ => {}
        }

        self.wait_for_back()?;

        Ok(())
    }

    /// 等待用户返回
    fn wait_for_back(&self) -> Result<()> {
        let items = vec!["⬅️  返回上一级菜单"];
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt("操作完成")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| crate::error::CliError::UserCancelled)?;
        Ok(())
    }
}

impl Default for BackupCommand {
    fn default() -> Self {
        Self::new().expect("Failed to create BackupCommand")
    }
}
