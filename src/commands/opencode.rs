// OpenCode 配置管理命令
// 采用新架构:Provider与模型分离,支持跨Provider选择

use crate::config::{ConfigManager, OpenCodeModelInfo, OpenCodeModelLimit, OpenCodeProvider};
use crate::ui::style::{show_error, show_info, show_success};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
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

    /// 执行命令
    pub fn execute(&mut self) -> Result<(), String> {
        loop {
            let choice =
                crate::ui::show_api_menu("🚀 OpenCode配置管理").map_err(|e| e.to_string())?;

            use crate::ui::style::ApiMenuChoice;
            match choice {
                ApiMenuChoice::Switch => {
                    if let Err(e) = self.handle_switch() {
                        show_error(&format!("切换配置失败: {}", e));
                        self.wait_for_back();
                    }
                }
                ApiMenuChoice::List => {
                    if let Err(e) = self.handle_list() {
                        show_error(&format!("查看配置失败: {}", e));
                        self.wait_for_back();
                    }
                }
                ApiMenuChoice::Add => {
                    if let Err(e) = self.handle_add() {
                        show_error(&format!("添加配置失败: {}", e));
                        self.wait_for_back();
                    }
                }
                ApiMenuChoice::Edit => {
                    if let Err(e) = self.handle_edit() {
                        show_error(&format!("编辑配置失败: {}", e));
                        self.wait_for_back();
                    }
                }
                ApiMenuChoice::Delete => {
                    if let Err(e) = self.handle_delete() {
                        show_error(&format!("删除配置失败: {}", e));
                        self.wait_for_back();
                    }
                }
                ApiMenuChoice::Back => break,
            }
        }

        Ok(())
    }

    // ========================================================================
    // 核心处理器
    // ========================================================================

    /// 处理切换配置(支持跨Provider模型选择)
    fn handle_switch(&mut self) -> Result<(), String> {
        println!("\n{}", style("🔄 切换 OpenCode 配置").cyan().bold());
        println!("{}", style("支持主模型和轻量模型来自不同Provider").dim());
        println!();

        // 读取所有 Provider
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_error("没有可用的 Provider 配置");
            show_info("请先使用「添加配置」功能添加 Provider");
            return Ok(());
        }

        // ===== 第1步：选择主模型 =====
        println!(
            "{}",
            style("📝 第1步: 选择主模型 (复杂任务使用)").white().bold()
        );
        let (main_provider, main_model) = self.select_model(&all_providers, "主模型")?;

        // ===== 第2步：选择轻量模型 =====
        println!(
            "\n{}",
            style("📝 第2步: 选择轻量模型 (简单任务使用)")
                .white()
                .bold()
        );
        let (small_provider, small_model) = self.select_model(&all_providers, "轻量模型")?;

        // ===== 第3步：确认配置 =====
        println!("\n{}", style("📋 配置预览：").white().bold());
        println!();

        println!("{}", style("主模型配置:").green());
        println!(
            "  {} {}",
            style("Provider:").white(),
            style(&main_provider).cyan()
        );
        println!("  {} {}", style("模型:").white(), style(&main_model).cyan());

        println!();
        println!("{}", style("轻量模型配置:").green());
        println!(
            "  {} {}",
            style("Provider:").white(),
            style(&small_provider).cyan()
        );
        println!(
            "  {} {}",
            style("模型:").white(),
            style(&small_model).cyan()
        );
        println!();

        if !self.confirm("确认切换配置", true)? {
            show_info("用户取消切换");
            return Ok(());
        }

        // ===== 第4步：执行切换 =====
        self.config_manager.switch_opencode_config(
            &main_provider,
            &main_model,
            &small_provider,
            &small_model,
        )?;

        show_success("✨ OpenCode 配置切换成功！");
        self.wait_for_back();

        Ok(())
    }

    /// 选择模型(Provider + Model)
    fn select_model(
        &self,
        all_providers: &HashMap<String, OpenCodeProvider>,
        model_type: &str,
    ) -> Result<(String, String), String> {
        // 第1步：选择 Provider
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
            .with_prompt(format!("选择 {} 的 Provider", model_type))
            .items(&provider_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let provider_name = provider_names[provider_idx].clone();
        let provider = all_providers.get(&provider_name).unwrap();

        println!(
            "{} {}",
            style("✓ 已选择 Provider:").green(),
            style(&provider_name).cyan()
        );

        // 第2步：选择模型
        if provider.models.is_empty() {
            return Err(format!("Provider '{}' 没有可用的模型", provider_name));
        }

        let model_ids: Vec<String> = provider.models.keys().cloned().collect();
        let model_items: Vec<String> = model_ids
            .iter()
            .map(|id| {
                let model_info = provider.models.get(id).unwrap();
                format!("🤖 {} ({})", id, model_info.name)
            })
            .collect();

        let model_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("选择 {} 的模型", model_type))
            .items(&model_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let model_id = model_ids[model_idx].clone();

        println!(
            "{} {}",
            style("✓ 已选择模型:").green(),
            style(&model_id).cyan()
        );

        Ok((provider_name, model_id))
    }

    /// 处理查看配置
    fn handle_list(&self) -> Result<(), String> {
        println!("\n{}", style("📋 所有 OpenCode 配置").cyan().bold());
        println!();

        // 显示当前激活的配置
        if let Some(active) = self.config_manager.get_active_opencode_config()? {
            println!("{}", style("🎯 当前使用的配置:").green().bold());
            println!();

            println!("{}", style("主模型配置:").white().bold());
            println!(
                "  {} {}",
                style("Provider:").white(),
                style(&active.main.provider).cyan()
            );
            println!(
                "  {} {}",
                style("Base URL:").white(),
                style(&active.main.base_url).dim()
            );
            println!(
                "  {} {}",
                style("模型:").white(),
                style(&active.main.model).cyan()
            );
            println!();

            println!("{}", style("轻量模型配置:").white().bold());
            println!(
                "  {} {}",
                style("Provider:").white(),
                style(&active.small.provider).cyan()
            );
            println!(
                "  {} {}",
                style("Base URL:").white(),
                style(&active.small.base_url).dim()
            );
            println!(
                "  {} {}",
                style("模型:").white(),
                style(&active.small.model).cyan()
            );
            println!();
        } else {
            show_info("当前没有激活的 OpenCode 配置");
            println!();
        }

        // 显示所有 Provider
        let all_providers = self.config_manager.opencode().get_all_providers()?;

        if all_providers.is_empty() {
            show_info("没有找到任何 Provider 配置");
        } else {
            println!("{}", style("🌐 所有可用 Provider:").white().bold());

            for (provider_name, provider) in &all_providers {
                println!();
                println!(
                    "  {} {}",
                    style("Provider:").white(),
                    style(provider_name).cyan()
                );
                println!(
                    "  {} {}",
                    style("Base URL:").white(),
                    style(&provider.options.base_url).dim()
                );

                if let Some(ref npm) = provider.npm {
                    println!("  {} {}", style("NPM:").white(), style(npm).dim());
                }

                if let Some(ref desc) = provider.metadata.description {
                    println!("  {} {}", style("描述:").white(), style(desc).yellow());
                }

                let model_list: Vec<&str> = provider.models.keys().map(|s| s.as_str()).collect();
                println!(
                    "  {} {}",
                    style("可用模型:").white(),
                    style(model_list.join(", ")).yellow()
                );
            }
        }

        println!();
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
            .with_prompt("Provider 名称 (如: AutoCore, Elysia)")
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
            .with_prompt("模型 ID (如: claude-sonnet-4-5)")
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

        // Base URL
        let new_base_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Base URL")
            .default(provider.options.base_url.clone())
            .interact_text()
            .map_err(|_| "用户取消操作")?;

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
                Some(new_base_url),
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
