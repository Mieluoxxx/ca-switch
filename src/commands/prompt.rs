use crate::error::{CliError, Result};
use crate::ui::{confirm, show_error, show_info, show_success};
use console::style;
use dialoguer::{theme::ColorfulTheme, Editor, Input, Select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Prompt 元数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub file: String,  // 对应的 txt 文件名
    pub created_at: String,
    pub updated_at: String,
}

/// Prompt 索引文件结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptIndex {
    #[serde(default)]
    pub prompts: HashMap<String, PromptMetadata>,
}

/// Prompt 管理命令
pub struct PromptCommand {
    prompts_dir: PathBuf,
    index_file: PathBuf,
}

impl PromptCommand {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| CliError::Config("无法获取用户主目录".into()))?;
        let prompts_dir = home.join(".ca-switch").join("prompts");
        let index_file = prompts_dir.join("index.json");

        // 确保 prompts 目录存在
        if !prompts_dir.exists() {
            fs::create_dir_all(&prompts_dir)
                .map_err(|e| CliError::Config(format!("创建 prompts 目录失败: {e}")))?;
        }

        // 确保 index.json 存在
        if !index_file.exists() {
            let empty_index = PromptIndex::default();
            let content = serde_json::to_string_pretty(&empty_index)
                .map_err(|e| CliError::Config(format!("序列化索引失败: {e}")))?;
            fs::write(&index_file, content)
                .map_err(|e| CliError::Config(format!("创建索引文件失败: {e}")))?;
        }

        Ok(Self {
            prompts_dir,
            index_file,
        })
    }

    /// 执行 Prompt 管理命令
    pub async fn execute(&mut self) -> Result<()> {
        loop {
            let choice = self.show_prompt_menu()?;

            match choice {
                PromptMenuChoice::List => {
                    if let Err(e) = self.handle_list().await {
                        show_error(&format!("查看 prompts 失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                PromptMenuChoice::View => {
                    if let Err(e) = self.handle_view().await {
                        show_error(&format!("查看 prompt 失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                PromptMenuChoice::Add => {
                    if let Err(e) = self.handle_add().await {
                        show_error(&format!("添加 prompt 失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                PromptMenuChoice::Edit => {
                    if let Err(e) = self.handle_edit().await {
                        show_error(&format!("编辑 prompt 失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                PromptMenuChoice::Delete => {
                    if let Err(e) = self.handle_delete().await {
                        show_error(&format!("删除 prompt 失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                PromptMenuChoice::Copy => {
                    if let Err(e) = self.handle_copy().await {
                        show_error(&format!("复制 prompt 失败: {e}"));
                        self.wait_for_back()?;
                    }
                }
                PromptMenuChoice::Back => break,
            }
        }

        Ok(())
    }

    /// 显示 Prompt 菜单
    fn show_prompt_menu(&self) -> Result<PromptMenuChoice> {
        println!("\n{}", style("📝 Prompt 管理").cyan().bold());
        println!("{}", style("═".repeat(40)).dim());

        let choices = [
            PromptMenuChoice::List,
            PromptMenuChoice::View,
            PromptMenuChoice::Add,
            PromptMenuChoice::Edit,
            PromptMenuChoice::Delete,
            PromptMenuChoice::Copy,
            PromptMenuChoice::Back,
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("请选择操作")
            .items(&choices)
            .default(0)
            .interact()
            .map_err(|_| CliError::UserCancelled)?;

        Ok(choices[selection])
    }

    /// 读取索引文件
    fn read_index(&self) -> Result<PromptIndex> {
        let content = fs::read_to_string(&self.index_file)
            .map_err(|e| CliError::Config(format!("读取索引文件失败: {e}")))?;

        serde_json::from_str(&content)
            .map_err(|e| CliError::Config(format!("解析索引文件失败: {e}")))
    }

    /// 保存索引文件
    fn save_index(&self, index: &PromptIndex) -> Result<()> {
        let content = serde_json::to_string_pretty(index)
            .map_err(|e| CliError::Config(format!("序列化索引失败: {e}")))?;

        fs::write(&self.index_file, content)
            .map_err(|e| CliError::Config(format!("保存索引文件失败: {e}")))?;

        Ok(())
    }

    /// 读取 prompt 内容
    fn read_prompt_content(&self, file_name: &str) -> Result<String> {
        let file_path = self.prompts_dir.join(file_name);
        fs::read_to_string(&file_path)
            .map_err(|e| CliError::Config(format!("读取 prompt 文件失败: {e}")))
    }

    /// 保存 prompt 内容
    fn save_prompt_content(&self, file_name: &str, content: &str) -> Result<()> {
        let file_path = self.prompts_dir.join(file_name);
        fs::write(&file_path, content)
            .map_err(|e| CliError::Config(format!("保存 prompt 文件失败: {e}")))?;

        Ok(())
    }

    /// 删除 prompt 文件
    fn delete_prompt_file(&self, file_name: &str) -> Result<()> {
        let file_path = self.prompts_dir.join(file_name);
        if file_path.exists() {
            fs::remove_file(&file_path)
                .map_err(|e| CliError::Config(format!("删除 prompt 文件失败: {e}")))?;
        }
        Ok(())
    }

    /// 生成唯一的文件名
    fn generate_filename(&self, name: &str) -> String {
        // 使用简单的文件名（name + .txt），如果重复则添加时间戳
        let base_name = name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>();

        let mut filename = format!("{}.txt", base_name);
        let mut counter = 1;

        // 检查文件是否存在，如果存在则添加序号
        while self.prompts_dir.join(&filename).exists() {
            filename = format!("{}_{}.txt", base_name, counter);
            counter += 1;
        }

        filename
    }

    /// 处理列表展示
    async fn handle_list(&self) -> Result<()> {
        println!("\n{}", style("📋 Prompt 列表").cyan().bold());
        println!("{}", style("═".repeat(60)).dim());

        let index = self.read_index()?;

        if index.prompts.is_empty() {
            show_info("暂无保存的 prompts");
            println!("\n提示: 使用 '添加 Prompt' 功能创建新的 prompt");
            return Ok(());
        }

        // 按名称排序
        let mut sorted_prompts: Vec<_> = index.prompts.iter().collect();
        sorted_prompts.sort_by(|a, b| a.0.cmp(b.0));

        println!("\n共找到 {} 个 prompts:\n", style(index.prompts.len()).cyan().bold());

        for (name, metadata) in sorted_prompts {
            let category = metadata.category
                .as_ref()
                .map(|c| format!("[{}]", style(c).yellow()))
                .unwrap_or_else(|| "".to_string());

            let description = metadata.description
                .as_ref()
                .map(|d| style(d).dim().to_string())
                .unwrap_or_else(|| style("无描述").dim().to_string());

            println!("  {} {} {}",
                style("▪").cyan(),
                style(name).white().bold(),
                category
            );
            println!("    {}", description);
            println!("    {} {} | {} {}",
                style("文件:").dim(),
                style(&metadata.file).dim(),
                style("更新于:").dim(),
                style(&metadata.updated_at).dim()
            );
            println!();
        }

        self.wait_for_back()?;
        Ok(())
    }

    /// 处理查看内容
    async fn handle_view(&self) -> Result<()> {
        let index = self.read_index()?;

        if index.prompts.is_empty() {
            show_info("暂无保存的 prompts");
            return Ok(());
        }

        let prompt_names: Vec<String> = {
            let mut names: Vec<_> = index.prompts.keys().cloned().collect();
            names.sort();
            names
        };

        let items: Vec<String> = prompt_names
            .iter()
            .map(|name| {
                let metadata = &index.prompts[name];
                let category = metadata.category
                    .as_ref()
                    .map(|c| format!(" [{}]", c))
                    .unwrap_or_default();
                format!("📄 {}{}", name, category)
            })
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要查看的 prompt")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| CliError::UserCancelled)?;

        let selected_name = &prompt_names[selection];
        let metadata = &index.prompts[selected_name];

        // 读取 prompt 内容
        let content = self.read_prompt_content(&metadata.file)?;

        println!("\n{}", style("═".repeat(60)).dim());
        println!("{} {}", style("名称:").white().bold(), style(selected_name).cyan());

        if let Some(category) = &metadata.category {
            println!("{} {}", style("分类:").white(), style(category).yellow());
        }

        if let Some(description) = &metadata.description {
            println!("{} {}", style("描述:").white(), description);
        }

        println!("{} {}", style("文件:").dim(), metadata.file);
        println!("{} {}", style("创建于:").dim(), metadata.created_at);
        println!("{} {}", style("更新于:").dim(), metadata.updated_at);
        println!("{}", style("═".repeat(60)).dim());
        println!("\n{}", style("内容:").white().bold());
        println!("{}", style("─".repeat(60)).dim());
        println!("{}", content);
        println!("{}", style("─".repeat(60)).dim());

        self.wait_for_back()?;
        Ok(())
    }

    /// 处理添加 prompt
    async fn handle_add(&self) -> Result<()> {
        println!("\n{}", style("➕ 添加新 Prompt").cyan().bold());
        println!("{}", style("═".repeat(40)).dim());

        // 输入名称
        let name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Prompt 名称")
            .validate_with(|input: &String| -> std::result::Result<(), &str> {
                if input.trim().is_empty() {
                    Err("名称不能为空")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|_| CliError::UserCancelled)?;

        let name = name.trim().to_string();

        // 读取索引
        let mut index = self.read_index()?;

        // 检查是否已存在
        if index.prompts.contains_key(&name) {
            if !confirm(&format!("Prompt '{name}' 已存在，是否覆盖?"), false)? {
                show_info("操作已取消");
                return Ok(());
            }
        }

        // 输入描述
        let description: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("描述 (可选)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| CliError::UserCancelled)?;

        let description = if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_string())
        };

        // 输入分类
        let category: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("分类 (可选)")
            .allow_empty(true)
            .interact_text()
            .map_err(|_| CliError::UserCancelled)?;

        let category = if category.trim().is_empty() {
            None
        } else {
            Some(category.trim().to_string())
        };

        // 使用编辑器输入内容
        println!("\n{}", style("请在编辑器中输入 prompt 内容...").dim());
        let content = Editor::new()
            .edit("")
            .map_err(|e| CliError::Config(format!("打开编辑器失败: {e}")))?
            .ok_or_else(|| CliError::UserCancelled)?;

        let content = content.trim().to_string();

        if content.is_empty() {
            show_error("内容不能为空");
            return Ok(());
        }

        // 生成文件名
        let filename = self.generate_filename(&name);

        // 保存内容到 txt 文件
        self.save_prompt_content(&filename, &content)?;

        // 更新索引
        let now = chrono::Local::now().to_rfc3339();
        let metadata = PromptMetadata {
            name: name.clone(),
            description,
            category,
            file: filename,
            created_at: now.clone(),
            updated_at: now,
        };

        index.prompts.insert(name.clone(), metadata);
        self.save_index(&index)?;

        show_success(&format!("Prompt '{name}' 已保存"));

        self.wait_for_back()?;
        Ok(())
    }

    /// 处理编辑 prompt
    async fn handle_edit(&self) -> Result<()> {
        let mut index = self.read_index()?;

        if index.prompts.is_empty() {
            show_info("暂无保存的 prompts");
            return Ok(());
        }

        let prompt_names: Vec<String> = {
            let mut names: Vec<_> = index.prompts.keys().cloned().collect();
            names.sort();
            names
        };

        let items: Vec<String> = prompt_names
            .iter()
            .map(|name| format!("📝 {}", name))
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要编辑的 prompt")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| CliError::UserCancelled)?;

        let selected_name = &prompt_names[selection];
        let metadata = index.prompts.get_mut(selected_name)
            .ok_or_else(|| CliError::Config(format!("Prompt '{selected_name}' 不存在")))?;

        println!("\n{}", style("📝 编辑 Prompt").cyan().bold());
        println!("{}", style("═".repeat(40)).dim());

        // 编辑描述
        let default_desc = metadata.description.clone().unwrap_or_default();
        let new_description: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("描述 (可选)")
            .allow_empty(true)
            .default(default_desc)
            .interact_text()
            .map_err(|_| CliError::UserCancelled)?;

        metadata.description = if new_description.trim().is_empty() {
            None
        } else {
            Some(new_description.trim().to_string())
        };

        // 编辑分类
        let default_cat = metadata.category.clone().unwrap_or_default();
        let new_category: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("分类 (可选)")
            .allow_empty(true)
            .default(default_cat)
            .interact_text()
            .map_err(|_| CliError::UserCancelled)?;

        metadata.category = if new_category.trim().is_empty() {
            None
        } else {
            Some(new_category.trim().to_string())
        };

        // 编辑内容
        if confirm("是否编辑内容?", true)? {
            let current_content = self.read_prompt_content(&metadata.file)?;

            println!("\n{}", style("请在编辑器中修改 prompt 内容...").dim());
            let new_content = Editor::new()
                .edit(&current_content)
                .map_err(|e| CliError::Config(format!("打开编辑器失败: {e}")))?
                .ok_or_else(|| CliError::UserCancelled)?;

            let new_content = new_content.trim().to_string();

            if !new_content.is_empty() {
                self.save_prompt_content(&metadata.file, &new_content)?;
            }
        }

        // 更新时间
        metadata.updated_at = chrono::Local::now().to_rfc3339();

        // 保存索引
        self.save_index(&index)?;
        show_success(&format!("Prompt '{selected_name}' 已更新"));

        self.wait_for_back()?;
        Ok(())
    }

    /// 处理删除 prompt
    async fn handle_delete(&self) -> Result<()> {
        let mut index = self.read_index()?;

        if index.prompts.is_empty() {
            show_info("暂无保存的 prompts");
            return Ok(());
        }

        let prompt_names: Vec<String> = {
            let mut names: Vec<_> = index.prompts.keys().cloned().collect();
            names.sort();
            names
        };

        let items: Vec<String> = prompt_names
            .iter()
            .map(|name| format!("🗑️  {}", name))
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要删除的 prompt")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| CliError::UserCancelled)?;

        let selected_name = &prompt_names[selection];

        if confirm(&format!("确定要删除 prompt '{selected_name}'?"), false)? {
            // 获取文件名并删除文件
            if let Some(metadata) = index.prompts.get(selected_name) {
                self.delete_prompt_file(&metadata.file)?;
            }

            // 从索引中移除
            index.prompts.remove(selected_name);
            self.save_index(&index)?;

            show_success(&format!("Prompt '{selected_name}' 已删除"));
        } else {
            show_info("操作已取消");
        }

        self.wait_for_back()?;
        Ok(())
    }

    /// 处理复制到剪贴板
    async fn handle_copy(&self) -> Result<()> {
        let index = self.read_index()?;

        if index.prompts.is_empty() {
            show_info("暂无保存的 prompts");
            return Ok(());
        }

        let prompt_names: Vec<String> = {
            let mut names: Vec<_> = index.prompts.keys().cloned().collect();
            names.sort();
            names
        };

        let items: Vec<String> = prompt_names
            .iter()
            .map(|name| format!("📋 {}", name))
            .collect();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("选择要复制的 prompt")
            .items(&items)
            .default(0)
            .interact()
            .map_err(|_| CliError::UserCancelled)?;

        let selected_name = &prompt_names[selection];
        let metadata = &index.prompts[selected_name];

        // 读取内容
        let content = self.read_prompt_content(&metadata.file)?;

        // 使用 cli-clipboard 复制到剪贴板
        use cli_clipboard::{ClipboardContext, ClipboardProvider};

        let mut ctx = ClipboardContext::new()
            .map_err(|e| CliError::Config(format!("初始化剪贴板失败: {e}")))?;

        ctx.set_contents(content)
            .map_err(|e| CliError::Config(format!("复制到剪贴板失败: {e}")))?;

        show_success(&format!("Prompt '{selected_name}' 已复制到剪贴板"));

        self.wait_for_back()?;
        Ok(())
    }

    /// 等待返回
    fn wait_for_back(&self) -> Result<()> {
        use crate::ui::wait_for_back_confirm;
        wait_for_back_confirm("")
    }
}

/// Prompt 菜单选项
#[derive(Debug, Clone, Copy)]
enum PromptMenuChoice {
    List,
    View,
    Add,
    Edit,
    Delete,
    Copy,
    Back,
}

impl std::fmt::Display for PromptMenuChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PromptMenuChoice::List => write!(f, "📋 列表展示 - 查看所有 prompts"),
            PromptMenuChoice::View => write!(f, "👁️  查看内容 - 查看 prompt 详细内容"),
            PromptMenuChoice::Add => write!(f, "➕ 添加 Prompt - 创建新的 prompt"),
            PromptMenuChoice::Edit => write!(f, "📝 编辑 Prompt - 修改现有 prompt"),
            PromptMenuChoice::Delete => write!(f, "🗑️  删除 Prompt - 删除指定 prompt"),
            PromptMenuChoice::Copy => write!(f, "📋 复制到剪贴板 - 快速复用 prompt"),
            PromptMenuChoice::Back => write!(f, "⬅️  返回上一级菜单"),
        }
    }
}
