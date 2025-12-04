// Claude 命令模块 - 基于新架构重构
// 支持新的配置文件结构：claude.json + config.json

use crate::config::{ClaudeSite, ConfigManager};
use crate::ui::{confirm, show_error, show_info, show_success, show_warning, ApiMenuChoice};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};

/// Claude API 管理命令
pub struct ClaudeCommand {
    config_manager: ConfigManager,
}

impl ClaudeCommand {
    /// 创建新的 Claude 命令实例
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            config_manager: ConfigManager::new()?,
        })
    }

    /// 执行 Claude API 管理命令
    pub fn execute(&mut self) -> Result<(), String> {
        loop {
            let choice = crate::ui::show_api_menu("📡 Claude配置管理").map_err(|e| e.to_string())?;

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
        println!("\n{}", style("🔄 切换 Claude API 配置").cyan().bold());
        println!();

        // 获取所有站点
        let sites = self.config_manager.claude().get_all_sites()?;

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

        // 检查是否有 tokens
        if selected_site.tokens.is_empty() {
            show_error("该站点没有配置 Token，请先添加 Token");
            return Ok(());
        }

        // 选择 Token
        let token_names: Vec<String> = selected_site.tokens.keys().cloned().collect();
        let token_items: Vec<String> = token_names
            .iter()
            .map(|name| {
                let token = selected_site.tokens.get(name).unwrap();
                let preview = if token.len() > 20 {
                    format!("{}...", &token[..20])
                } else {
                    token.clone()
                };
                format!("🔑 {} ({})", name, preview)
            })
            .collect();

        let token_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择 Token")
            .items(&token_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_token_name = &token_names[token_idx];
        let selected_token = selected_site.tokens.get(selected_token_name).unwrap();

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
        println!(
            "  {} {}",
            style("Token:").white(),
            style(format!("{}...", &selected_token[..20.min(selected_token.len())])).cyan()
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
            .switch_claude_config(selected_site_name, selected_token_name)?;

        show_success(&format!(
            "✨ 成功切换到配置: {} - {}",
            selected_site_name, selected_token_name
        ));
        self.wait_for_back();

        Ok(())
    }

    // ========================================================================
    // 查看配置
    // ========================================================================

    fn handle_list(&self) -> Result<(), String> {
        println!("\n{}", style("📋 Claude API 配置列表").cyan().bold());
        println!();

        // 显示当前激活的配置
        if let Some(active_config) = self.config_manager.get_active_claude_config()? {
            println!("{}", style("🎯 当前使用的配置:").green().bold());
            println!("  {} {}", style("站点:").white(), style(&active_config.site).cyan());
            println!(
                "  {} {}",
                style("URL:").white(),
                style(&active_config.site_url).dim()
            );
            println!(
                "  {} {}",
                style("Token:").white(),
                style(&active_config.token_name).cyan()
            );
            if let Some(ref base_url) = active_config.base_url {
                println!("  {} {}", style("Base URL:").white(), style(base_url).dim());
            }
            if let Some(ref model) = active_config.model {
                println!("  {} {}", style("Model:").white(), style(model).yellow());
            }
            if active_config.vertex.enabled {
                println!("  {} {}", style("Vertex AI:").white(), style("启用").green());
                if let Some(ref project_id) = active_config.vertex.project_id {
                    println!(
                        "    {} {}",
                        style("Project ID:").white(),
                        style(project_id).dim()
                    );
                }
            }
            println!();
        } else {
            println!("{}", style("⚠️  当前没有激活的配置").yellow());
            println!();
        }

        // 显示所有站点
        let sites = self.config_manager.claude().get_all_sites()?;

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

            println!("  {} {}", style("Tokens:").white(), style(site.tokens.len()).yellow());
            for (token_name, token) in &site.tokens {
                let preview = if token.len() > 20 {
                    format!("{}...", &token[..20])
                } else {
                    token.clone()
                };
                println!("    - {} ({})", style(token_name).cyan(), style(preview).dim());
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
        println!("\n{}", style("➕ 添加 Claude API 配置").cyan().bold());
        println!();

        // 选择操作类型
        let choices = vec!["添加新站点", "在已有站点中添加 Token", "返回"];

        let choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择操作")
            .items(&choices)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        match choice {
            0 => self.add_new_site(),
            1 => self.add_token_to_existing_site(),
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
            .claude()
            .get_site(&site_name)?
            .is_some()
        {
            return Err(format!("站点 '{}' 已存在", site_name));
        }

        // 输入 Base URL
        let base_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("API Base URL")
            .default("https://api.anthropic.com".to_string())
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

        // 创建站点（使用 base_url 作为 url）
        self.config_manager
            .claude_mut()
            .add_site(site_name.clone(), base_url.clone(), description)?;

        // 配置 Vertex AI
        println!();
        let use_vertex = confirm("是否使用 Vertex AI", false).map_err(|e| e.to_string())?;

        if use_vertex {
            // Vertex 模式
            let project_id: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Vertex Project ID")
                .interact_text()
                .map_err(|_| "用户取消操作")?;

            let skip_auth = confirm("跳过 Vertex 认证", false).map_err(|e| e.to_string())?;

            let vertex_config = crate::config::VertexConfig {
                enabled: true,
                project_id: Some(project_id),
                base_url: Some(base_url.clone()),
                skip_auth,
            };

            // 更新站点配置：设置 model 和 vertex
            self.config_manager
                .claude_mut()
                .update_site_config(&site_name, None, model, Some(vertex_config))?;
        } else {
            // 普通模式：设置 base_url 和 model
            self.config_manager
                .claude_mut()
                .update_site_config(&site_name, Some(base_url), model, None)?;
        }

        show_success(&format!("成功创建站点: {}", site_name));

        // 询问是否立即添加 Token
        println!();
        let add_token = confirm("是否立即添加 Token", true).map_err(|e| e.to_string())?;

        if add_token {
            self.add_token_to_site(&site_name)?;
        }

        self.wait_for_back();
        Ok(())
    }

    /// 在已有站点中添加 Token
    fn add_token_to_existing_site(&mut self) -> Result<(), String> {
        // 获取所有站点
        let sites = self.config_manager.claude().get_all_sites()?;

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

        self.add_token_to_site(selected_site)?;
        self.wait_for_back();
        Ok(())
    }

    /// 添加 Token 到指定站点
    fn add_token_to_site(&mut self, site_name: &str) -> Result<(), String> {
        println!("\n{}", style(format!("为站点 '{}' 添加 Token", site_name)).cyan());
        println!();

        // 输入 Token 名称
        let token_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Token 名称")
            .default("主账号".to_string())
            .interact_text()
            .map_err(|_| "用户取消操作")?;

        // 输入 Token 值
        let token: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Token 值（输入不可见）")
            .interact()
            .map_err(|_| "用户取消操作")?;

        if token.is_empty() {
            return Err("Token 值不能为空".to_string());
        }

        // 添加 Token
        self.config_manager
            .claude_mut()
            .add_token(site_name, token_name.clone(), token)?;

        show_success(&format!("成功添加 Token: {}", token_name));

        Ok(())
    }

    // ========================================================================
    // 编辑配置
    // ========================================================================

    fn handle_edit(&mut self) -> Result<(), String> {
        println!("\n{}", style("✏️  编辑 Claude API 配置").cyan().bold());
        println!();

        // 获取所有站点
        let sites = self.config_manager.claude().get_all_sites()?;

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
            "编辑 Token",
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
            2 => self.edit_token(selected_site_name, selected_site)?,
            3 => return Ok(()),
            _ => return Ok(()),
        }

        self.wait_for_back();
        Ok(())
    }

    /// 编辑站点元数据
    fn edit_site_metadata(&mut self, site_name: &str, site: &ClaudeSite) -> Result<(), String> {
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
            .claude_mut()
            .update_site_metadata(site_name, Some(new_url), new_description)?;

        show_success("成功更新站点元数据");

        Ok(())
    }

    /// 编辑站点配置
    fn edit_site_config(&mut self, site_name: &str, site: &ClaudeSite) -> Result<(), String> {
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

        // 编辑 Vertex AI 配置
        println!();
        println!("{}", style("🔷 Vertex AI 配置").cyan());

        let vertex_enabled = confirm(
            &format!("是否启用 Vertex AI (当前: {})", if site.config.vertex.enabled { "已启用" } else { "未启用" }),
            site.config.vertex.enabled
        ).map_err(|e| e.to_string())?;

        let vertex_config = if vertex_enabled {
            // Vertex Project ID
            let current_project_id = site.config.vertex.project_id.clone().unwrap_or_default();
            let project_id: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Vertex Project ID")
                .default(current_project_id)
                .allow_empty(true)
                .interact_text()
                .map_err(|_| "用户取消操作")?;

            let project_id = if project_id.is_empty() {
                None
            } else {
                Some(project_id)
            };

            // Vertex Base URL
            let current_vertex_url = site.config.vertex.base_url.clone().unwrap_or_default();
            let vertex_url: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Vertex Base URL（可选）")
                .default(current_vertex_url)
                .allow_empty(true)
                .interact_text()
                .map_err(|_| "用户取消操作")?;

            let vertex_url = if vertex_url.is_empty() {
                None
            } else {
                Some(vertex_url)
            };

            // Skip Auth
            let skip_auth = confirm(
                &format!("是否跳过 Vertex 认证 (当前: {})", if site.config.vertex.skip_auth { "是" } else { "否" }),
                site.config.vertex.skip_auth
            ).map_err(|e| e.to_string())?;

            Some(crate::config::VertexConfig {
                enabled: true,
                project_id,
                base_url: vertex_url,
                skip_auth,
            })
        } else {
            Some(crate::config::VertexConfig {
                enabled: false,
                project_id: None,
                base_url: None,
                skip_auth: false,
            })
        };

        // 更新站点配置
        self.config_manager
            .claude_mut()
            .update_site_config(site_name, new_base_url, new_model, vertex_config)?;

        show_success("成功更新站点配置");

        Ok(())
    }

    /// 编辑 Token
    fn edit_token(&mut self, site_name: &str, site: &ClaudeSite) -> Result<(), String> {
        if site.tokens.is_empty() {
            show_error("该站点没有 Token");
            return Ok(());
        }

        println!("\n{}", style("编辑 Token").cyan());
        println!();

        // 选择 Token
        let token_names: Vec<String> = site.tokens.keys().cloned().collect();
        let token_items: Vec<String> = token_names
            .iter()
            .map(|name| format!("🔑 {}", name))
            .collect();

        let token_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要编辑的 Token")
            .items(&token_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let token_name = &token_names[token_idx];

        // 输入新的 Token 值
        let new_token: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("新的 Token 值（输入不可见）")
            .interact()
            .map_err(|_| "用户取消操作")?;

        if new_token.is_empty() {
            return Err("Token 值不能为空".to_string());
        }

        // 更新 Token
        self.config_manager
            .claude_mut()
            .update_token(site_name, token_name, new_token)?;

        show_success(&format!("成功更新 Token: {}", token_name));

        Ok(())
    }

    // ========================================================================
    // 删除配置
    // ========================================================================

    fn handle_delete(&mut self) -> Result<(), String> {
        println!("\n{}", style("🗑️  删除 Claude API 配置").cyan().bold());
        println!();

        // 获取所有站点
        let sites = self.config_manager.claude().get_all_sites()?;

        if sites.is_empty() {
            show_error("没有可用的站点配置");
            return Ok(());
        }

        // 选择删除类型
        let delete_choices = vec!["删除整个站点", "删除站点中的 Token", "返回"];

        let delete_choice = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择删除类型")
            .items(&delete_choices)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        match delete_choice {
            0 => self.delete_site()?,
            1 => self.delete_token()?,
            2 => return Ok(()),
            _ => return Ok(()),
        }

        self.wait_for_back();
        Ok(())
    }

    /// 删除站点
    fn delete_site(&mut self) -> Result<(), String> {
        // 获取所有站点
        let sites = self.config_manager.claude().get_all_sites()?;

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
        self.config_manager.claude_mut().remove_site(selected_site)?;

        show_success(&format!("成功删除站点: {}", selected_site));

        Ok(())
    }

    /// 删除 Token
    fn delete_token(&mut self) -> Result<(), String> {
        // 获取所有站点
        let sites = self.config_manager.claude().get_all_sites()?;

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

        if selected_site.tokens.is_empty() {
            show_error("该站点没有 Token");
            return Ok(());
        }

        // 选择 Token
        let token_names: Vec<String> = selected_site.tokens.keys().cloned().collect();
        let token_items: Vec<String> = token_names
            .iter()
            .map(|name| format!("🔑 {}", name))
            .collect();

        let token_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要删除的 Token")
            .items(&token_items)
            .default(0)
            .interact()
            .map_err(|_| "用户取消操作")?;

        let selected_token = &token_names[token_idx];

        // 确认删除
        show_warning(&format!(
            "⚠️  警告：即将删除站点 '{}' 的 Token '{}'",
            selected_site_name, selected_token
        ));
        let confirmed = confirm("确认删除", false).map_err(|e| e.to_string())?;

        if !confirmed {
            show_info("用户取消删除");
            return Ok(());
        }

        // 执行删除
        self.config_manager
            .claude_mut()
            .remove_token(selected_site_name, selected_token)?;

        show_success(&format!("成功删除 Token: {}", selected_token));

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
