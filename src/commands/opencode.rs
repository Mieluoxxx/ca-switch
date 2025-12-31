// OpenCode 配置管理命令
// 采用新架构:Provider与模型分离,支持跨Provider选择

use crate::config::{ConfigManager, OpenCodeModelInfo, OpenCodeModelLimit, OpenCodeProvider};
use crate::ui::style::{show_error, show_info, show_opencode_menu, show_success};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, MultiSelect, Select};
use std::collections::HashMap;

/// OpenCode 管理命令
pub struct OpenCodeCommand {
    config_manager: ConfigManager,
}

impl OpenCodeCommand {
    /// 创建新的命令实例
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            config_manager: ConfigManager::new()?,
        })
    }

    /// 执行命令 (原有方法，内部循环)
    pub fn execute(&mut self) -> Result<(), String> {
        loop {
            match self.execute_with_menu()? {
                crate::ui::style::OpenCodeMenuChoice::Exit => break,
                crate::ui::style::OpenCodeMenuChoice::Backup => {
                    // 备份功能由外部 Menu 处理
                    // 这里只是为了保持兼容性
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 执行单次菜单交互并返回用户选择
    /// 供外部 Menu 调用以处理 Backup 和 Exit
    pub fn execute_with_menu(&mut self) -> Result<crate::ui::style::OpenCodeMenuChoice, String> {
        let choice =
            show_opencode_menu("🚀 OpenCode配置管理").map_err(|e| e.to_string())?;

        use crate::ui::style::OpenCodeMenuChoice;
        match choice {
            OpenCodeMenuChoice::Apply => {
                if let Err(e) = self.handle_apply() {
                    show_error(&format!("应用配置失败: {}", e));
                    self.wait_for_back();
                }
            }
            OpenCodeMenuChoice::Add => {
                if let Err(e) = self.handle_add() {
                    show_error(&format!("添加配置失败: {}", e));
                    self.wait_for_back();
                }
            }
            OpenCodeMenuChoice::Edit => {
                if let Err(e) = self.handle_edit() {
                    show_error(&format!("编辑配置失败: {}", e));
                    self.wait_for_back();
                }
            }
            OpenCodeMenuChoice::Delete => {
                if let Err(e) = self.handle_delete() {
                    show_error(&format!("删除配置失败: {}", e));
                    self.wait_for_back();
                }
            }
            OpenCodeMenuChoice::DetectSite => {
                if let Err(e) = self.handle_detect_site() {
                    show_error(&format!("站点检测失败: {}", e));
                    self.wait_for_back();
                }
            }
            OpenCodeMenuChoice::DetectModel => {
                if let Err(e) = self.handle_detect_model() {
                    show_error(&format!("模型检测失败: {}", e));
                    self.wait_for_back();
                }
            }
            // Backup 和 Exit 由调用方处理
            OpenCodeMenuChoice::Backup | OpenCodeMenuChoice::Exit => {}
        }

        Ok(choice)
    }

    // ========================================================================
    // 核心处理器
    // ========================================================================

    /// 处理应用配置(支持多选Provider和多选应用范围)
    fn handle_apply(&mut self) -> Result<(), String> {
        println!("\n{}", style("🚀 应用 OpenCode 配置").cyan().bold());
        println!("{}", style("选择要应用的 Provider 配置 (可多选)").dim());
        println!();

        // 读取所有 Provider
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_error("没有可用的 Provider 配置");
            show_info("请先使用「添加配置」功能添加 Provider");
            return Ok(());
        }

        // 多选 Provider
        let provider_names: Vec<String> = all_providers.keys().cloned().collect();
        let provider_items: Vec<String> = provider_names
            .iter()
            .map(|name| {
                let provider = all_providers.get(name).unwrap();
                format!(
                    "🔌 {} ({})",
                    name,
                    provider.metadata.description.as_deref().unwrap_or("")
                )
            })
            .collect();

        let provider_selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要应用的 Provider (空格选择,回车确认)")
            .items(&provider_items)
            .interact()
            .map_err(|_| "用户取消操作")?;

        if provider_selections.is_empty() {
            show_info("未选择任何 Provider");
            return Ok(());
        }

        let selected_providers: Vec<String> = provider_selections
            .iter()
            .map(|&idx| provider_names[idx].clone())
            .collect();

        // 显示配置预览
        println!("\n{}", style("📋 配置预览：").white().bold());
        println!();
        for provider_name in &selected_providers {
            let provider = all_providers.get(provider_name).unwrap();
            println!("{}", style(provider_name).cyan().bold());
            println!(
                "  {} {}",
                style("Base URL:").white(),
                style(&provider.options.base_url).dim()
            );

            if let Some(ref desc) = provider.metadata.description {
                println!("  {} {}", style("描述:").white(), style(desc).yellow());
            }

            let model_list: Vec<&str> = provider.models.keys().map(|s| s.as_str()).collect();
            println!(
                "  {} {}",
                style("可用模型:").white(),
                style(model_list.join(", ")).yellow()
            );
            println!();
        }

        // 多选应用范围
        println!("{}", style("📍 选择应用范围 (可多选):").white().bold());
        let scope_choices = vec!["🌍 全局 - 应用到全局配置", "📁 项目 - 应用到当前项目"];

        // 循环验证：确保用户选择至少一个应用范围
        let scope_selections = loop {
            let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("选择应用范围 (空格选择,回车确认) - ⚠️ 必须至少选择一项")
                .items(&scope_choices)
                .interact()
                .map_err(|_| "用户取消操作")?;

            if selections.is_empty() {
                show_error("❌ 必须至少选择一个应用范围！");
                
                // 提供继续或取消的选项
                if !self.confirm("是否继续选择应用范围？", true)? {
                    show_info("用户取消应用配置");
                    return Ok(());
                }
                continue; // 重新显示选择界面
            }
            
            // 成功选择，返回结果
            break selections;
        };

        let apply_to_global = scope_selections.contains(&0);
        let apply_to_project = scope_selections.contains(&1);

        // 显示确认信息
        println!();
        println!(
            "{}",
            style(format!("✓ 将应用 {} 个 Provider", selected_providers.len())).green()
        );
        if apply_to_global {
            println!("{}", style("✓ 将应用到全局配置").green());
        }
        if apply_to_project {
            println!("{}", style("✓ 将应用到当前项目").green());
        }
        println!();

        if !self.confirm("确认应用此配置", true)? {
            show_info("用户取消应用");
            return Ok(());
        }

        // 执行应用
        if apply_to_global {
            println!();
            println!(
                "{}",
                style("正在应用 Provider 到全局配置...")
                    .cyan()
                    .bold()
            );
            
            self.config_manager
                .apply_multiple_opencode_to_global(&selected_providers)?;
            show_success("✨ 已应用到全局配置！");
            println!(
                "{}",
                style("  配置文件: ~/.opencode/opencode.json").dim()
            );
        }

        if apply_to_project {
            println!();
            println!(
                "{}",
                style("正在应用 Provider 到当前项目...")
                    .cyan()
                    .bold()
            );
            
            self.config_manager
                .apply_multiple_opencode_to_project(&selected_providers)?;
            show_success("✨ 已应用到当前项目！");

            // 获取当前目录并显示配置路径
            if let Ok(current_dir) = std::env::current_dir() {
                println!(
                    "{}",
                    style(format!(
                        "  配置文件: {}/.opencode/opencode.json",
                        current_dir.display()
                    ))
                    .dim()
                );
            }
        }

        println!();
        show_success(&format!(
            "🎉 成功应用 {} 个 Provider 配置！",
            selected_providers.len()
        ));

        self.wait_for_back();

        Ok(())
    }

    /// 处理添加配置
    fn handle_add(&mut self) -> Result<(), String> {
        let choices = vec![
            "➕ 添加新 Provider",
            "🤖 向已有 Provider 添加模型",
            "⬅️  返回上一级菜单",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择操作")
            .items(&choices)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        match selection {
            0 => self.add_new_provider()?,
            1 => self.add_model_to_provider_interactive()?,
            _ => {}
        }

        Ok(())
    }

    /// 添加新 Provider
    fn add_new_provider(&mut self) -> Result<(), String> {
        println!("\n{}", style("➕ 添加新 Provider").cyan().bold());
        println!();

        // Provider 名称
        let provider_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Provider 名称 (如: MyProvider, CustomAI)")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Provider 名称不能为空")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // Base URL
        let base_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Base URL")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("Base URL 不能为空")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // API Key
        let api_key: String = dialoguer::Password::with_theme(&ColorfulTheme::default())
            .with_prompt("API Key")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("API Key 不能为空")
                } else if input.len() < 10 {
                    Err("API Key 长度不能少于10个字符")
                } else {
                    Ok(())
                }
            })
            .interact()
            .map_err(|_| "用户取消操作")?;

        // NPM 包
        let npm: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("NPM 包 (如: @ai-sdk/openai-compatible, 可选)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let npm = if npm.trim().is_empty() {
            None
        } else {
            Some(npm)
        };

        // 描述
        let description: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("描述 (可选)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let description = if description.trim().is_empty() {
            None
        } else {
            Some(description)
        };

        // 添加 Provider
        self.config_manager.opencode_mut().add_provider(
            provider_name.clone(),
            base_url,
            api_key,
            npm,
            description,
        )?;

        show_success(&format!("✅ Provider '{}' 添加成功！", provider_name));
        show_info("接下来请前往编辑配置中添加模型");

        self.wait_for_back();

        Ok(())
    }

    /// 向已有 Provider 添加模型(交互式)
    fn add_model_to_provider_interactive(&mut self) -> Result<(), String> {
        // 选择 Provider
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_error("没有可用的 Provider");
            show_info("请先添加 Provider");
            return Ok(());
        }

        let provider_name = self.select_provider(&all_providers)?;

        // 循环添加模型，允许用户连续添加多个模型到同一个 Provider
        loop {
            println!("\n{}", style("🤖 添加模型").cyan().bold());
            println!();

            let choices = vec!["➕ 添加新模型", "⬅️  返回上一级菜单"];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("请选择操作")
                .items(&choices)
                .default(0)
                .interact()
                .map_err(|_| "用户取消操作")?;

            match selection {
                0 => self.add_model_to_provider(&provider_name)?,
                _ => break,
            }
        }

        Ok(())
    }

    /// 添加模型到指定 Provider
    fn add_model_to_provider(&mut self, provider_name: &str) -> Result<(), String> {
        // 模型 ID
        let model_id: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("模型 ID (如: gpt-4, model-name)")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.trim().is_empty() {
                    Err("模型 ID 不能为空")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // 模型名称
        let model_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("模型显示名称")
            .default(model_id.clone())
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // Context Limit
        let context_limit_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Context Limit (留空则不设置)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let context_limit = if context_limit_str.is_empty() {
            None
        } else {
            Some(context_limit_str.parse::<u64>().map_err(|_| "无效的数字")?)
        };

        // Output Limit
        let output_limit_str: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Output Limit (留空则不设置)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let output_limit = if output_limit_str.is_empty() {
            None
        } else {
            Some(output_limit_str.parse::<u64>().map_err(|_| "无效的数字")?)
        };

        // 构建嵌套的 limit 结构
        let limit = if context_limit.is_some() || output_limit.is_some() {
            Some(OpenCodeModelLimit {
                context: context_limit,
                output: output_limit,
            })
        } else {
            None
        };

        let model_info = OpenCodeModelInfo {
            name: model_name,
            limit,
            model_detection: None,
        };

        self.config_manager.opencode_mut().add_model(
            provider_name,
            model_id.clone(),
            model_info,
        )?;

        show_success(&format!(
            "✅ 模型 '{}' 已添加到 Provider '{}'",
            model_id, provider_name
        ));

        Ok(())
    }

    /// 处理编辑配置
    fn handle_edit(&mut self) -> Result<(), String> {
        println!("\n{}", style("📝 编辑配置").cyan().bold());
        println!();

        // 选择 Provider
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_error("没有可用的 Provider");
            return Ok(());
        }

        let provider_name = self.select_provider(&all_providers)?;

        // 选择编辑类型
        let choices = vec![
            "📝 编辑 Provider 元数据",
            "🤖 管理模型",
            "⬅️  返回上一级菜单",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择操作")
            .items(&choices)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        match selection {
            0 => self.edit_provider_metadata(&provider_name)?,
            1 => self.edit_models(&provider_name)?,
            _ => {}
        }

        Ok(())
    }

    /// 编辑 Provider 元数据
    fn edit_provider_metadata(&mut self, provider_name: &str) -> Result<(), String> {
        println!("\n{}", style("📝 编辑 Provider 元数据").cyan().bold());
        println!();

        let provider = self
            .config_manager
            .opencode()
            .get_provider(provider_name)?
            .ok_or_else(|| format!("Provider '{}' 不存在", provider_name))?;

        // Base URL (留空保持不变)
        let base_url_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Base URL (留空保持不变)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_base_url = if base_url_input.trim().is_empty() {
            None
        } else {
            Some(base_url_input)
        };

        // API Key
        let new_api_key: String = dialoguer::Password::with_theme(&ColorfulTheme::default())
            .with_prompt("API Key (留空保持不变)")
            .allow_empty_password(true)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let new_api_key = if new_api_key.trim().is_empty() {
            None
        } else {
            Some(new_api_key)
        };

        // NPM 包
        let new_npm: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("NPM 包 (留空则不设置)")
            .default(provider.npm.clone().unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_npm = if new_npm.trim().is_empty() {
            None
        } else {
            Some(new_npm)
        };

        // 描述
        let new_description: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("描述 (留空则不设置)")
            .default(provider.metadata.description.clone().unwrap_or_default())
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_description = if new_description.trim().is_empty() {
            None
        } else {
            Some(new_description)
        };

        self.config_manager
            .opencode_mut()
            .update_provider_metadata(
                provider_name,
                new_base_url,
                new_api_key,
                new_npm,
                new_description,
            )?;

        show_success(&format!("✅ Provider '{}' 元数据已更新", provider_name));

        self.wait_for_back();

        Ok(())
    }

    /// 管理模型
    fn edit_models(&mut self, provider_name: &str) -> Result<(), String> {
        loop {
            println!("\n{}", style("🤖 管理模型").cyan().bold());
            println!();

            let choices = vec!["➕ 添加新模型", "🗑️  删除模型", "⬅️  返回上一级菜单"];

            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("请选择操作")
                .items(&choices)
                .default(0)
                .interact()
                .map_err(|_| "用户取消操作")?;

            match selection {
                0 => self.add_model_to_provider(provider_name)?,
                1 => self.delete_model_from_provider(provider_name)?,
                _ => break,
            }
        }

        Ok(())
    }

    /// 删除模型
    fn delete_model_from_provider(&mut self, provider_name: &str) -> Result<(), String> {
        let models = self.config_manager.opencode().get_models(provider_name)?;

        if models.is_empty() {
            show_error("该 Provider 没有模型");
            return Ok(());
        }

        let model_ids: Vec<String> = models.keys().cloned().collect();
        let model_items: Vec<String> = model_ids
            .iter()
            .map(|id| {
                let model_info = models.get(id).unwrap();
                format!("🤖 {} ({})", id, model_info.name)
            })
            .collect();

        let model_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要删除的模型")
            .items(&model_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let model_id = &model_ids[model_idx];

        if !self.confirm(&format!("确认删除模型 '{}'?", model_id), false)? {
            show_info("取消删除");
            return Ok(());
        }

        self.config_manager
            .opencode_mut()
            .delete_model(provider_name, model_id)?;

        show_success(&format!("✅ 模型 '{}' 已删除", model_id));

        Ok(())
    }

    /// 处理删除配置
    fn handle_delete(&mut self) -> Result<(), String> {
        println!("\n{}", style("🗑️  删除配置").red().bold());
        println!();

        // 选择 Provider
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_error("没有可用的 Provider");
            return Ok(());
        }

        let provider_name = self.select_provider(&all_providers)?;

        println!(
            "\n{}",
            style("⚠️  警告: 此操作将删除整个 Provider 及其所有配置").yellow()
        );
        println!();

        if !self.confirm(&format!("确认删除 Provider '{}'?", provider_name), false)? {
            show_info("取消删除");
            return Ok(());
        }

        self.config_manager
            .opencode_mut()
            .delete_provider(&provider_name)?;

        show_success(&format!("✅ Provider '{}' 已删除", provider_name));

        self.wait_for_back();

        Ok(())
    }

    // ========================================================================
    // 辅助方法
    // ========================================================================

    /// 选择 Provider
    fn select_provider(
        &self,
        all_providers: &HashMap<String, OpenCodeProvider>,
    ) -> Result<String, String> {
        let provider_names: Vec<String> = all_providers.keys().cloned().collect();
        let provider_items: Vec<String> = provider_names
            .iter()
            .map(|name| {
                let provider = all_providers.get(name).unwrap();
                format!(
                    "🔌 {} ({})",
                    name,
                    provider.metadata.description.as_deref().unwrap_or("")
                )
            })
            .collect();

        let provider_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择 Provider")
            .items(&provider_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        Ok(provider_names[provider_idx].clone())
    }

    /// 确认对话框
    fn confirm(&self, prompt: &str, default: bool) -> Result<bool, String> {
        dialoguer::Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(default)
            .interact()
            .map_err(|_| "用户取消操作".to_string())
    }

    // ========================================================================
    // 站点检测和模型检测
    // ========================================================================

    /// 处理站点检测
    fn handle_detect_site(&mut self) -> Result<(), String> {
        println!("\n{}", style("🌐 站点检测").cyan().bold());

        // 1. 获取所有Providers
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_error("没有可用的Provider");
            show_info("请先使用「添加配置」功能添加 Provider");
            return Ok(());
        }

        // 2. 选择要诊断的Provider
        let provider_name = self.select_provider(&all_providers)?;
        let provider = all_providers
            .get(&provider_name)
            .ok_or("Provider不存在")?;

        println!("\n{}", style(format!("Provider: {}", provider_name)).white());
        println!(
            "{}",
            style(format!("Base URL: {}", provider.options.base_url)).dim()
        );

        // 3. 执行检测
        show_info("正在检测站点...");

        use crate::config::Detector;

        let detector = Detector::new();
        let base_url = provider.options.base_url.clone();
        let api_key = provider.options.api_key.clone();

        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                detector.detect_site(&base_url, &api_key).await
            })
        });

        // 4. 显示结果
        self.show_site_detection_report(&result);

        // 5. 批量导入模型(如果检测成功)
        if result.is_available && !result.available_models.is_empty() {
            if self.confirm("是否批量导入检测到的模型?", true)? {
                self.batch_import_models(&provider_name, &result.available_models)?;
            }
        }

        // 6. 保存检测结果
        if self.confirm("是否保存检测结果到配置?", true)? {
            self.save_site_detection(&provider_name, result)?;
            show_success("检测结果已保存");
        }

        Ok(())
    }

    /// 显示站点检测报告
    fn show_site_detection_report(&self, result: &crate::config::models::SiteDetectionResult) {
        println!("\n{}", style("═".repeat(60)).dim());
        println!("{}", style("📊 站点检测报告").cyan().bold());
        println!("{}", style("═".repeat(60)).dim());

        // 站点状态
        if result.is_available {
            println!("\n{} {}", "✅", style("站点状态: 可用").green().bold());
            println!("{} {}", "🔑", style("API Key: 有效").green());

            if let Some(time) = result.response_time_ms {
                println!(
                    "{} {} ms",
                    "⚡",
                    style(format!("响应时间: {:.0}", time)).yellow()
                );
            }

            println!(
                "\n{} {} 个",
                "🤖",
                style(format!(
                    "检测到模型: {}",
                    result.available_models.len()
                ))
                .cyan()
                .bold()
            );

            for (i, model) in result.available_models.iter().enumerate() {
                println!("  {}. {}", i + 1, style(model).white());
            }
        } else {
            println!(
                "\n{} {}",
                "❌",
                style("站点状态: 不可用").red().bold()
            );

            if let Some(err) = &result.error_message {
                println!("{} {}", "⚠️ ", style(format!("错误: {}", err)).yellow());
            }
        }

        println!(
            "\n{}",
            style(format!("检测时间: {}", result.detected_at)).dim()
        );
        println!("{}", style("═".repeat(60)).dim());
    }

    /// 批量导入模型
    fn batch_import_models(&mut self, provider_name: &str, models: &[String]) -> Result<(), String> {
        let mut imported = 0;

        for model_id in models {
            // 检查模型是否已存在
            if self
                .config_manager
                .opencode()
                .get_models(provider_name)?
                .contains_key(model_id)
            {
                continue; // 跳过已存在的
            }

            // 添加模型
            let new_model_info = OpenCodeModelInfo {
                name: model_id.clone(),
                limit: None,
                model_detection: None,
            };

            self.config_manager.opencode_mut().add_model(
                provider_name,
                model_id.clone(),
                new_model_info,
            )?;

            imported += 1;
        }

        show_success(&format!("成功导入 {} 个新模型", imported));
        Ok(())
    }

    /// 保存站点检测结果
    fn save_site_detection(
        &mut self,
        provider_name: &str,
        result: crate::config::models::SiteDetectionResult,
    ) -> Result<(), String> {
        let mut config = self.config_manager.opencode().read_config()?;

        if let Some(provider) = config.providers.get_mut(provider_name) {
            provider.site_detection = Some(result);
        } else {
            return Err("Provider不存在".to_string());
        }

        self.config_manager.opencode().write_config(&config)?;
        Ok(())
    }

    /// 处理模型检测
    fn handle_detect_model(&mut self) -> Result<(), String> {
        println!("\n{}", style("🤖 模型检测").cyan().bold());

        // 1. 选择Provider
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_error("没有可用的Provider");
            show_info("请先使用「添加配置」功能添加 Provider");
            return Ok(());
        }

        let provider_name = self.select_provider(&all_providers)?;
        let provider = all_providers
            .get(&provider_name)
            .ok_or("Provider不存在")?;

        // 2. 选择模型
        let models = self.config_manager.opencode().get_models(&provider_name)?;

        if models.is_empty() {
            show_error("该Provider没有配置模型");
            show_info("请先添加模型或使用站点检测功能批量导入");
            return Ok(());
        }

        let model_id = self.select_model_from_list(&models)?;

        println!(
            "\n{}",
            style(format!("Provider: {}", provider_name)).white()
        );
        println!("{}", style(format!("Model: {}", model_id)).white());

        // 3. 询问是否测试流式输出
        let test_stream = self.confirm("是否测试流式输出功能?", false)?;

        // 4. 执行检测
        show_info("正在检测模型...");

        use crate::config::Detector;

        let detector = Detector::new();
        let base_url = provider.options.base_url.clone();
        let api_key = provider.options.api_key.clone();

        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                detector
                    .detect_model(&base_url, &api_key, &model_id, test_stream)
                    .await
            })
        });

        // 5. 显示结果
        self.show_model_detection_report(&result);

        // 6. 保存检测结果
        if result.is_available {
            if self.confirm("是否保存检测结果到配置?", true)? {
                self.save_model_detection(&provider_name, &model_id, result)?;
                show_success("检测结果已保存");
            }
        }

        Ok(())
    }

    /// 显示模型检测报告
    fn show_model_detection_report(&self, result: &crate::config::models::ModelDetectionResult) {
        println!("\n{}", style("═".repeat(60)).dim());
        println!(
            "{}",
            style(format!("📊 模型检测报告: {}", result.model_id))
                .cyan()
                .bold()
        );
        println!("{}", style("═".repeat(60)).dim());

        if result.is_available {
            println!("\n{} {}", "✅", style("模型状态: 可用").green().bold());

            if let Some(time) = result.first_token_time_ms {
                println!(
                    "{} {} ms",
                    "⚡",
                    style(format!("首次响应时间: {:.0}", time)).yellow()
                );
            }

            if let Some(time) = result.total_response_time_ms {
                println!(
                    "{} {} ms",
                    "⏱️ ",
                    style(format!("总响应时间: {:.0}", time)).yellow()
                );
            }

            if let Some(tps) = result.tokens_per_second {
                println!(
                    "{} {} tokens/s",
                    "🚀",
                    style(format!("Token速度: {:.2}", tps)).cyan().bold()
                );
            }

            if let Some(stream) = result.stream_available {
                if stream {
                    println!(
                        "{} {}",
                        "✅",
                        style("流式输出: 支持").green()
                    );
                } else {
                    println!(
                        "{} {}",
                        "❌",
                        style("流式输出: 不支持").red()
                    );
                }
            }
        } else {
            println!(
                "\n{} {}",
                "❌",
                style("模型状态: 不可用").red().bold()
            );

            if let Some(err) = &result.error_message {
                println!("{} {}", "⚠️ ", style(format!("错误: {}", err)).yellow());
            }
        }

        println!(
            "\n{}",
            style(format!("检测时间: {}", result.detected_at)).dim()
        );
        println!("{}", style("═".repeat(60)).dim());
    }

    /// 保存模型检测结果
    fn save_model_detection(
        &mut self,
        provider_name: &str,
        model_id: &str,
        result: crate::config::models::ModelDetectionResult,
    ) -> Result<(), String> {
        let mut config = self.config_manager.opencode().read_config()?;

        if let Some(provider) = config.providers.get_mut(provider_name) {
            if let Some(model_info) = provider.models.get_mut(model_id) {
                model_info.model_detection = Some(result);
            } else {
                return Err("模型不存在".to_string());
            }
        } else {
            return Err("Provider不存在".to_string());
        }

        self.config_manager.opencode().write_config(&config)?;
        Ok(())
    }

    /// 从模型列表中选择模型
    fn select_model_from_list(
        &self,
        models: &HashMap<String, OpenCodeModelInfo>,
    ) -> Result<String, String> {
        let mut model_list: Vec<_> = models.iter().collect();
        model_list.sort_by(|a, b| a.0.cmp(b.0));

        let model_names: Vec<String> = model_list.iter().map(|(id, _)| (*id).clone()).collect();

        let selection_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择模型")
            .items(&model_names)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        Ok(model_names[selection_idx].clone())
    }

    /// 等待用户返回
    fn wait_for_back(&self) {
        let items = vec!["⬅️  返回上一级菜单"];
        let _ = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("操作完成")
            .items(&items)
            .default(0)
            .interact();
    }
}

impl Default for OpenCodeCommand {
    fn default() -> Self {
        Self::new().expect("Failed to create OpenCodeCommand")
    }
}
