// Codex 命令模块 - 基于新架构重构
// 支持新的配置文件结构：codex.json + config.json

use crate::config::{CodexSite, ConfigManager};
use crate::ui::{confirm, show_error, show_info, show_success, show_warning, ApiMenuChoice};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};

/// Codex API 管理命令
pub struct CodexCommand {
    config_manager: ConfigManager,
}

impl CodexCommand {
    /// 创建新的 Codex 命令实例
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            config_manager: ConfigManager::new()?,
        })
    }

    /// 执行 Codex API 管理命令
    pub fn execute(&mut self) -> Result<(), String> {
        loop {
            let choice = crate::ui::show_api_menu("💻 Codex配置管理").map_err(|e| e.to_string())?;

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
    // 切换配置
    // ========================================================================

    fn handle_switch(&mut self) -> Result<(), String> {
        println!("\n{}", style("🔄 切换 Codex API 配置").cyan().bold());
        println!();

        // 获取所有站点
        let sites = self.config_manager.codex().get_all_sites()?;

        if sites.is_empty() {
            show_error("没有可用的站点配置，请先添加站点");
            return Ok(());
        }

        // 选择站点
        let site_names: Vec<String> = sites.keys().cloned().collect();
        let site_items: Vec<String> = site_names
            .iter()
            .map(|name| {
                let site = sites.get(name).unwrap();
                format!("🌐 {} ({})", name, site.metadata.url)
            })
            .collect();

        let site_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择站点")
            .items(&site_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_site_name = &site_names[site_idx];
        let selected_site = sites.get(selected_site_name).unwrap();

        // 检查是否有 API Keys
        if selected_site.api_keys.is_empty() {
            show_error("该站点没有配置 API Key，请先添加 API Key");
            return Ok(());
        }

        // 选择 API Key
        let key_names: Vec<String> = selected_site.api_keys.keys().cloned().collect();
        let key_items: Vec<String> = key_names
            .iter()
            .map(|name| {
                let key = selected_site.api_keys.get(name).unwrap();
                let preview = if key.len() > 20 {
                    format!("{}...", &key[..20])
                } else {
                    key.clone()
                };
                format!("🔑 {} ({})", name, preview)
            })
            .collect();

        let key_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择 API Key")
            .items(&key_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_key_name = &key_names[key_idx];
        let selected_key = selected_site.api_keys.get(selected_key_name).unwrap();

        // 显示将要切换到的配置
        println!("\n{}", style("📋 即将切换到以下配置：").white());
        println!();
        println!("  {} {}", style("站点:").white(), style(selected_site_name).cyan());
        println!("  {} {}", style("URL:").white(), style(&selected_site.metadata.url).dim());
        if let Some(ref base_url) = selected_site.config.base_url {
            println!("  {} {}", style("Base URL:").white(), style(base_url).dim());
        }
        if let Some(ref model) = selected_site.config.model {
            println!("  {} {}", style("Model:").white(), style(model).yellow());
        }
        if let Some(ref provider) = selected_site.config.model_provider {
            println!("  {} {}", style("Model Provider:").white(), style(provider).green());
        }
        println!(
            "  {} {}",
            style("API Key:").white(),
            style(format!("{}...", &selected_key[..20.min(selected_key.len())])).cyan()
        );
        println!();

        // 确认切换
        let confirmed = confirm("确认切换配置", true).map_err(|e| e.to_string())?;

        if !confirmed {
            show_info("用户取消切换");
            return Ok(());
        }

        // 执行切换
        self.config_manager
            .switch_codex_config(selected_site_name, selected_key_name)?;

        show_success(&format!(
            "✨ 成功切换到配置: {} - {}",
            selected_site_name, selected_key_name
        ));
        self.wait_for_back();

        Ok(())
    }

    // ========================================================================
    // 查看配置
    // ========================================================================

    fn handle_list(&self) -> Result<(), String> {
        println!("\n{}", style("📋 Codex API 配置列表").cyan().bold());
        println!();

        // 显示当前激活的配置
        if let Some(active_config) = self.config_manager.get_active_codex_config()? {
            println!("{}", style("🎯 当前使用的配置:").green().bold());
            println!("  {} {}", style("站点:").white(), style(&active_config.site).cyan());
            println!(
                "  {} {}",
                style("URL:").white(),
                style(&active_config.site_url).dim()
            );
            println!(
                "  {} {}",
                style("API Key:").white(),
                style(&active_config.api_key_name).cyan()
            );
            if let Some(ref base_url) = active_config.base_url {
                println!("  {} {}", style("Base URL:").white(), style(base_url).dim());
            }
            if let Some(ref model) = active_config.model {
                println!("  {} {}", style("Model:").white(), style(model).yellow());
            }
            if let Some(ref reasoning_effort) = active_config.model_reasoning_effort {
                println!(
                    "  {} {}",
                    style("Model Reasoning Effort:").white(),
                    style(reasoning_effort).yellow()
                );
            }
            if let Some(ref provider) = active_config.model_provider {
                println!("  {} {}", style("Model Provider:").white(), style(provider).green());
            }
            if let Some(ref network_access) = active_config.network_access {
                println!("  {} {}", style("Network Access:").white(), style(network_access).cyan());
            }
            println!();
        } else {
            println!("{}", style("⚠️  当前没有激活的配置").yellow());
            println!();
        }

        // 显示所有站点
        let sites = self.config_manager.codex().get_all_sites()?;

        if sites.is_empty() {
            show_info("没有可用的站点配置");
            self.wait_for_back();
            return Ok(());
        }

        println!("{}", style("🌐 所有可用站点:").white().bold());
        println!();

        for (site_name, site) in &sites {
            println!("  {} {}", style("站点:").white(), style(site_name).cyan().bold());
            println!("  {} {}", style("URL:").white(), style(&site.metadata.url).dim());

            if let Some(ref desc) = site.metadata.description {
                println!("  {} {}", style("描述:").white(), style(desc).dim());
            }

            if let Some(ref base_url) = site.config.base_url {
                println!("  {} {}", style("Base URL:").white(), style(base_url).dim());
            }

            if let Some(ref model) = site.config.model {
                println!("  {} {}", style("Model:").white(), style(model).yellow());
            }

            if let Some(ref reasoning_effort) = site.config.model_reasoning_effort {
                println!(
                    "  {} {}",
                    style("Model Reasoning Effort:").white(),
                    style(reasoning_effort).yellow()
                );
            }

            if let Some(ref provider) = site.config.model_provider {
                println!("  {} {}", style("Model Provider:").white(), style(provider).green());
            }

            if let Some(ref network_access) = site.config.network_access {
                println!("  {} {}", style("Network Access:").white(), style(network_access).cyan());
            }

            println!("  {} {}", style("API Keys:").white(), style(site.api_keys.len()).yellow());
            for (key_name, key) in &site.api_keys {
                let preview = if key.len() > 20 {
                    format!("{}...", &key[..20])
                } else {
                    key.clone()
                };
                println!("    - {} ({})", style(key_name).cyan(), style(preview).dim());
            }

            println!();
        }

        self.wait_for_back();
        Ok(())
    }

    // ========================================================================
    // 添加配置
    // ========================================================================

    fn handle_add(&mut self) -> Result<(), String> {
        println!("\n{}", style("➕ 添加 Codex API 配置").cyan().bold());
        println!();

        // 选择操作类型
        let choices = vec!["添加新站点", "在已有站点中添加 API Key", "返回"];

        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择操作")
            .items(&choices)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        match choice {
            0 => self.add_new_site(),
            1 => self.add_key_to_existing_site(),
            2 => Ok(()),
            _ => Ok(()),
        }
    }

    /// 添加新站点
    fn add_new_site(&mut self) -> Result<(), String> {
        println!("\n{}", style("创建新站点").cyan().bold());
        println!();

        // 输入站点名称
        let site_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("站点名称")
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // 检查站点是否已存在
        if self
            .config_manager
            .codex()
            .get_site(&site_name)?
            .is_some()
        {
            return Err(format!("站点 '{}' 已存在", site_name));
        }

        // 输入 Base URL
        let base_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("API Base URL")
            .default("https://api.openai.com/v1".to_string())
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // 输入描述（可选）
        let description: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("站点描述（可选）")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let description = if description.is_empty() {
            None
        } else {
            Some(description)
        };

        // 输入 Model（可选）
        let model: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("默认模型（可选）")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let model = if model.is_empty() {
            None
        } else {
            Some(model)
        };

        // 输入 Reasoning Effort（可选）
        let reasoning_effort: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Reasoning Effort (low/medium/high, 可选)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let reasoning_effort = if reasoning_effort.is_empty() {
            None
        } else {
            Some(reasoning_effort)
        };

        // 输入 Provider（可选）
        let provider: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Provider（可选，默认使用站点名）")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let provider = if provider.is_empty() {
            None
        } else {
            Some(provider)
        };

        // 选择 Wire API (chat 或 responses)
        let wire_api_options = vec!["不设置", "chat", "responses"];
        let wire_api_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Wire API")
            .items(&wire_api_options)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let wire_api = match wire_api_idx {
            1 => Some("chat".to_string()),
            2 => Some("responses".to_string()),
            _ => None,
        };

        // 创建站点（使用 base_url 作为 url）
        self.config_manager
            .codex_mut()
            .add_site(site_name.clone(), base_url.clone(), description)?;

        // 更新站点配置（设置 base_url 和其他可选参数）
        self.config_manager.codex_mut().update_site_config(
            &site_name,
            Some(base_url),
            model,
            reasoning_effort,
            provider,
            None, // network_access
            None, // disable_response_storage
            wire_api,
        )?;

        show_success(&format!("成功创建站点: {}", site_name));

        // 询问是否立即添加 API Key
        println!();
        let add_key = confirm("是否立即添加 API Key", true).map_err(|e| e.to_string())?;

        if add_key {
            self.add_key_to_site(&site_name)?;
        }

        self.wait_for_back();
        Ok(())
    }

    /// 在已有站点中添加 API Key
    fn add_key_to_existing_site(&mut self) -> Result<(), String> {
        // 获取所有站点
        let sites = self.config_manager.codex().get_all_sites()?;

        if sites.is_empty() {
            show_error("没有可用的站点，请先添加站点");
            return Ok(());
        }

        // 选择站点
        let site_names: Vec<String> = sites.keys().cloned().collect();
        let site_items: Vec<String> = site_names.iter().map(|name| format!("🌐 {}", name)).collect();

        let site_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择站点")
            .items(&site_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_site = &site_names[site_idx];

        self.add_key_to_site(selected_site)?;
        self.wait_for_back();
        Ok(())
    }

    /// 添加 API Key 到指定站点
    fn add_key_to_site(&mut self, site_name: &str) -> Result<(), String> {
        println!("\n{}", style(format!("为站点 '{}' 添加 API Key", site_name)).cyan());
        println!();

        // 输入 API Key 名称
        let key_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("API Key 名称")
            .default("主账号".to_string())
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // 输入 API Key 值
        let api_key: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("API Key 值（输入不可见）")
            .interact()
            .map_err(|_| "用户取消操作")?;

        if api_key.is_empty() {
            return Err("API Key 值不能为空".to_string());
        }

        // 添加 API Key
        self.config_manager
            .codex_mut()
            .add_api_key(site_name, key_name.clone(), api_key)?;

        show_success(&format!("成功添加 API Key: {}", key_name));

        Ok(())
    }

    // ========================================================================
    // 编辑配置
    // ========================================================================

    fn handle_edit(&mut self) -> Result<(), String> {
        println!("\n{}", style("✏️  编辑 Codex API 配置").cyan().bold());
        println!();

        // 获取所有站点
        let sites = self.config_manager.codex().get_all_sites()?;

        if sites.is_empty() {
            show_error("没有可用的站点配置");
            return Ok(());
        }

        // 选择站点
        let site_names: Vec<String> = sites.keys().cloned().collect();
        let site_items: Vec<String> = site_names.iter().map(|name| format!("🌐 {}", name)).collect();

        let site_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要编辑的站点")
            .items(&site_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_site_name = &site_names[site_idx];
        let selected_site = sites.get(selected_site_name).unwrap();

        // 选择编辑类型
        let edit_choices = vec![
            "编辑站点元数据（URL、描述）",
            "编辑站点配置（Base URL、Model等）",
            "编辑 API Key",
            "返回",
        ];

        let edit_choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择编辑类型")
            .items(&edit_choices)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        match edit_choice {
            0 => self.edit_site_metadata(selected_site_name, selected_site)?,
            1 => self.edit_site_config(selected_site_name, selected_site)?,
            2 => self.edit_api_key(selected_site_name, selected_site)?,
            3 => return Ok(()),
            _ => return Ok(()),
        }

        self.wait_for_back();
        Ok(())
    }

    /// 编辑站点元数据
    fn edit_site_metadata(&mut self, site_name: &str, site: &CodexSite) -> Result<(), String> {
        println!("\n{}", style("编辑站点元数据").cyan());
        println!();

        // 编辑 URL
        let new_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("站点 URL")
            .default(site.metadata.url.clone())
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // 编辑描述
        let current_desc = site.metadata.description.clone().unwrap_or_default();
        let new_description: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("描述（可选）")
            .default(current_desc)
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_description = if new_description.is_empty() {
            None
        } else {
            Some(new_description)
        };

        // 更新站点元数据
        self.config_manager
            .codex_mut()
            .update_site_metadata(site_name, Some(new_url), new_description)?;

        show_success("成功更新站点元数据");

        Ok(())
    }

    /// 编辑站点配置
    fn edit_site_config(&mut self, site_name: &str, site: &CodexSite) -> Result<(), String> {
        println!("\n{}", style("编辑站点配置").cyan());
        println!();

        // 编辑 Base URL
        let current_base_url = site.config.base_url.clone().unwrap_or_default();
        let new_base_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Base URL（可选）")
            .default(current_base_url)
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_base_url = if new_base_url.is_empty() {
            None
        } else {
            Some(new_base_url)
        };

        // 编辑 Model
        let current_model = site.config.model.clone().unwrap_or_default();
        let new_model: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Model（可选）")
            .default(current_model)
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_model = if new_model.is_empty() {
            None
        } else {
            Some(new_model)
        };

        // 编辑 Model Reasoning Effort
        let current_reasoning_effort = site.config.model_reasoning_effort.clone().unwrap_or_default();
        let new_reasoning_effort: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Model Reasoning Effort (low/medium/high, 可选)")
            .default(current_reasoning_effort)
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_reasoning_effort = if new_reasoning_effort.is_empty() {
            None
        } else {
            Some(new_reasoning_effort)
        };

        // 编辑 Model Provider
        let current_provider = site.config.model_provider.clone().unwrap_or_default();
        let new_provider: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Model Provider（可选，默认使用站点名）")
            .default(current_provider)
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_provider = if new_provider.is_empty() {
            None
        } else {
            Some(new_provider)
        };

        // 编辑 Network Access
        let current_network_access = site.config.network_access.clone().unwrap_or_default();
        let new_network_access: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Network Access (enabled/disabled, 可选)")
            .default(current_network_access)
            .allow_empty(true)
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        let new_network_access = if new_network_access.is_empty() {
            None
        } else {
            Some(new_network_access)
        };

        // 编辑 Wire API (使用选择器)
        let wire_api_options = vec!["不设置", "chat", "responses"];
        let current_wire_api = site.config.wire_api.clone().unwrap_or_default();

        let default_idx = if current_wire_api.is_empty() {
            0
        } else if current_wire_api == "chat" {
            1
        } else if current_wire_api == "responses" {
            2
        } else {
            0
        };

        let wire_api_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Wire API")
            .items(&wire_api_options)
            .default(default_idx)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let new_wire_api = match wire_api_idx {
            1 => Some("chat".to_string()),
            2 => Some("responses".to_string()),
            _ => None,
        };

        // 更新站点配置
        self.config_manager.codex_mut().update_site_config(
            site_name,
            new_base_url,
            new_model,
            new_reasoning_effort,
            new_provider,
            new_network_access,
            None, // disable_response_storage (暂不在 UI 中编辑)
            new_wire_api,
        )?;

        show_success("成功更新站点配置");

        Ok(())
    }

    /// 编辑 API Key
    fn edit_api_key(&mut self, site_name: &str, site: &CodexSite) -> Result<(), String> {
        if site.api_keys.is_empty() {
            show_error("该站点没有 API Key");
            return Ok(());
        }

        println!("\n{}", style("编辑 API Key").cyan());
        println!();

        // 选择 API Key
        let key_names: Vec<String> = site.api_keys.keys().cloned().collect();
        let key_items: Vec<String> = key_names
            .iter()
            .map(|name| format!("🔑 {}", name))
            .collect();

        let key_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要编辑的 API Key")
            .items(&key_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let key_name = &key_names[key_idx];

        // 输入新的 API Key 值
        let new_key: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("新的 API Key 值（输入不可见）")
            .interact()
            .map_err(|_| "用户取消操作")?;

        if new_key.is_empty() {
            return Err("API Key 值不能为空".to_string());
        }

        // 更新 API Key
        self.config_manager
            .codex_mut()
            .update_api_key(site_name, key_name, new_key)?;

        show_success(&format!("成功更新 API Key: {}", key_name));

        Ok(())
    }

    // ========================================================================
    // 删除配置
    // ========================================================================

    fn handle_delete(&mut self) -> Result<(), String> {
        println!("\n{}", style("🗑️  删除 Codex API 配置").cyan().bold());
        println!();

        // 获取所有站点
        let sites = self.config_manager.codex().get_all_sites()?;

        if sites.is_empty() {
            show_error("没有可用的站点配置");
            return Ok(());
        }

        // 选择删除类型
        let delete_choices = vec!["删除整个站点", "删除站点中的 API Key", "返回"];

        let delete_choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择删除类型")
            .items(&delete_choices)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        match delete_choice {
            0 => self.delete_site()?,
            1 => self.delete_api_key()?,
            2 => return Ok(()),
            _ => return Ok(()),
        }

        self.wait_for_back();
        Ok(())
    }

    /// 删除站点
    fn delete_site(&mut self) -> Result<(), String> {
        // 获取所有站点
        let sites = self.config_manager.codex().get_all_sites()?;

        // 选择站点
        let site_names: Vec<String> = sites.keys().cloned().collect();
        let site_items: Vec<String> = site_names.iter().map(|name| format!("🌐 {}", name)).collect();

        let site_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要删除的站点")
            .items(&site_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_site = &site_names[site_idx];

        // 确认删除
        show_warning(&format!("⚠️  警告：即将删除站点 '{}'", selected_site));
        let confirmed = confirm("确认删除", false).map_err(|e| e.to_string())?;

        if !confirmed {
            show_info("用户取消删除");
            return Ok(());
        }

        // 执行删除
        self.config_manager.codex_mut().delete_site(selected_site)?;

        show_success(&format!("成功删除站点: {}", selected_site));

        Ok(())
    }

    /// 删除 API Key
    fn delete_api_key(&mut self) -> Result<(), String> {
        // 获取所有站点
        let sites = self.config_manager.codex().get_all_sites()?;

        // 选择站点
        let site_names: Vec<String> = sites.keys().cloned().collect();
        let site_items: Vec<String> = site_names.iter().map(|name| format!("🌐 {}", name)).collect();

        let site_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择站点")
            .items(&site_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_site_name = &site_names[site_idx];
        let selected_site = sites.get(selected_site_name).unwrap();

        if selected_site.api_keys.is_empty() {
            show_error("该站点没有 API Key");
            return Ok(());
        }

        // 选择 API Key
        let key_names: Vec<String> = selected_site.api_keys.keys().cloned().collect();
        let key_items: Vec<String> = key_names
            .iter()
            .map(|name| format!("🔑 {}", name))
            .collect();

        let key_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要删除的 API Key")
            .items(&key_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_key = &key_names[key_idx];

        // 确认删除
        show_warning(&format!(
            "⚠️  警告：即将删除站点 '{}' 的 API Key '{}'",
            selected_site_name, selected_key
        ));
        let confirmed = confirm("确认删除", false).map_err(|e| e.to_string())?;

        if !confirmed {
            show_info("用户取消删除");
            return Ok(());
        }

        // 执行删除
        self.config_manager
            .codex_mut()
            .delete_api_key(selected_site_name, selected_key)?;

        show_success(&format!("成功删除 API Key: {}", selected_key));

        Ok(())
    }

    // ========================================================================
    // 辅助方法
    // ========================================================================

    fn wait_for_back(&self) {
        println!();
        println!("{}", style("按回车键返回...").dim());
        let _ = std::io::stdin().read_line(&mut String::new());
    }
}
