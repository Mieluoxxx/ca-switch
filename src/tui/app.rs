// TUI App 状态机核心

use ratatui::widgets::ListState;

use crate::config::ConfigManager;

use super::{
    types::{AppTab, InputMode, LogEntry, MessageType, StatusMessage},
    ui::components::{ApplyScopeDialog, ConfirmDialog, FormField, InputForm, MultiSelectDialog},
};

/// Provider 表单类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFormMode {
    Add,
    Edit,
}


/// TUI 应用状态
pub struct App {
    // === 状态管理 ===
    /// 当前选中的 Tab
    pub current_tab: AppTab,
    /// 输入模式
    pub input_mode: InputMode,
    /// 是否应该退出
    pub should_quit: bool,
    /// 是否显示帮助
    pub help_visible: bool,

    // === 数据层 ===
    /// 配置管理器
    pub config_manager: ConfigManager,
    /// Provider 名称列表
    pub providers: Vec<String>,
    /// 当前 Provider 下的 Model 列表
    pub models: Vec<String>,

    // === UI 状态 ===
    /// Provider 列表状态
    pub provider_list_state: ListState,
    /// Model 列表状态
    pub model_list_state: ListState,
    /// Provider Tab 焦点区域 (0=Provider列表, 1=Model列表)
    pub provider_tab_focus: usize,

    // === 表单和对话框 ===
    /// Provider 表单
    pub provider_form: InputForm,
    /// Provider 表单模式
    pub provider_form_mode: ProviderFormMode,
    /// Model 表单
    pub model_form: InputForm,
    /// 确认删除对话框
    pub delete_dialog: ConfirmDialog,
    /// 应用配置确认对话框
    pub apply_dialog: ConfirmDialog,
    /// 应用范围选择对话框
    pub apply_scope_dialog: ApplyScopeDialog,
    /// Model 删除对话框
    pub model_delete_dialog: ConfirmDialog,
    /// 模型多选对话框（获取站点模型时使用）
    pub model_select_dialog: MultiSelectDialog,
    /// 当前正在搜索的列表类型 (用于 / 搜索)
    pub search_active: bool,
    /// 搜索关键词
    pub search_query: String,

    // === 应用配置相关状态 ===
    /// 是否处于多选应用模式
    pub is_multi_apply_mode: bool,
    /// 已选择的 Provider 列表
    pub selected_providers: Vec<String>,
    /// 多选模式下当前高亮项
    pub multi_apply_list_state: ListState,

    // === 消息和日志 ===
    /// 状态栏消息
    pub status_message: Option<StatusMessage>,
    /// 操作日志
    pub operation_logs: Vec<LogEntry>,
}

impl App {
    /// 创建新的 App 实例
    pub fn new() -> Result<Self, String> {
        let config_manager = ConfigManager::new()?;

        // 获取 Provider 列表并排序（保持稳定顺序）
        let mut providers: Vec<String> = config_manager
            .opencode()
            .get_all_providers()
            .unwrap_or_default()
            .keys()
            .cloned()
            .collect();
        providers.sort();

        let mut provider_list_state = ListState::default();
        if !providers.is_empty() {
            provider_list_state.select(Some(0));
        }

        // 创建 Provider 表单
        let provider_form = InputForm::new("添加 Provider")
            .add_field(FormField::new("名称").placeholder("provider-name").required())
            .add_field(FormField::new("API Key").placeholder("sk-xxx...").password().required())
            .add_field(FormField::new("Base URL").placeholder("https://api.example.com/v1"));

        // 创建 Model 表单
        let model_form = InputForm::new("添加 Model")
            .add_field(FormField::new("Model ID").placeholder("gpt-4o").required());

        // 创建删除确认对话框
        let delete_dialog = ConfirmDialog::new("确认删除", "确定要删除这个 Provider 吗？")
            .with_buttons("删除", "取消");

        // 创建应用配置对话框
        let apply_dialog = ConfirmDialog::new("应用配置", "确定要应用这个 Provider 的配置吗？")
            .with_buttons("应用", "取消");

        // 创建应用范围选择对话框
        let apply_scope_dialog = ApplyScopeDialog::new();

        // 创建 Model 删除对话框
        let model_delete_dialog = ConfirmDialog::new("确认删除", "确定要删除这个 Model 吗？")
            .with_buttons("删除", "取消");

        // 创建模型多选对话框
        let model_select_dialog = MultiSelectDialog::new("选择要添加的模型");

        // 初始化多选应用模式的列表状态
        let mut multi_apply_list_state = ListState::default();
        multi_apply_list_state.select(Some(0));

        let mut app = Self {
            current_tab: AppTab::default(),
            input_mode: InputMode::default(),
            should_quit: false,
            help_visible: false,
            config_manager,
            providers,
            models: Vec::new(),
            provider_list_state,
            model_list_state: ListState::default(),
            provider_tab_focus: 0,
            provider_form,
            provider_form_mode: ProviderFormMode::Add,
            model_form,
            delete_dialog,
            apply_dialog,
            apply_scope_dialog,
            model_delete_dialog,
            model_select_dialog,
            search_active: false,
            search_query: String::new(),
            is_multi_apply_mode: false,
            selected_providers: Vec::new(),
            multi_apply_list_state,
            status_message: None,
            operation_logs: Vec::new(),
        };

        // 初始化时加载第一个 Provider 的 Model 列表
        app.refresh_models();

        Ok(app)
    }

    /// 刷新 Provider 列表，同时同步所有相关状态
    pub fn refresh_providers(&mut self) -> Result<(), String> {
        // 获取最新的 Provider 列表并排序（保持稳定顺序）
        let mut new_providers: Vec<String> = self
            .config_manager
            .opencode()
            .get_all_providers()
            .unwrap_or_default()
            .keys()
            .cloned()
            .collect();
        new_providers.sort();
        self.providers = new_providers;

        // 调整 Provider 列表选中状态
        if self.providers.is_empty() {
            self.provider_list_state.select(None);
            self.models.clear();
            self.model_list_state.select(None);
        } else if let Some(i) = self.provider_list_state.selected() {
            if i >= self.providers.len() {
                self.provider_list_state.select(Some(self.providers.len() - 1));
            }
            // 刷新当前选中 Provider 的 Model 列表
            self.refresh_models();
        }

        Ok(())
    }

    /// 切换到下一个 Tab
    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    /// 切换到上一个 Tab
    pub fn prev_tab(&mut self) {
        self.current_tab = self.current_tab.prev();
    }

    /// 切换帮助显示
    pub fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    /// 退出应用
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// 显示 Toast 消息
    pub fn show_toast(&mut self, content: String, msg_type: MessageType) {
        self.status_message = Some(StatusMessage::new(content, msg_type));
    }

    /// 显示成功消息
    pub fn show_success(&mut self, content: &str) {
        self.show_toast(content.to_string(), MessageType::Success);
    }

    /// 显示错误消息
    pub fn show_error(&mut self, content: &str) {
        self.show_toast(content.to_string(), MessageType::Error);
    }

    /// 显示信息消息
    pub fn show_info(&mut self, content: &str) {
        self.show_toast(content.to_string(), MessageType::Info);
    }

    pub fn show_warning(&mut self, content: &str) {
        self.show_toast(content.to_string(), MessageType::Warning);
    }

    /// 记录操作日志
    pub fn log_operation(&mut self, message: String, level: MessageType) {
        self.operation_logs.push(LogEntry::new(message, level));
        // 保留最近100条日志
        if self.operation_logs.len() > 100 {
            self.operation_logs.remove(0);
        }
    }

    /// 清理过期消息
    pub fn cleanup_expired_messages(&mut self) {
        if let Some(ref msg) = self.status_message {
            if msg.is_expired() {
                self.status_message = None;
            }
        }
    }

    // === Provider 列表导航 ===

    /// 选择下一个 Provider
    pub fn select_next_provider(&mut self) {
        if self.providers.is_empty() {
            return;
        }
        let i = match self.provider_list_state.selected() {
            Some(i) => {
                if i >= self.providers.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.provider_list_state.select(Some(i));
    }

    /// 选择上一个 Provider
    pub fn select_prev_provider(&mut self) {
        if self.providers.is_empty() {
            return;
        }
        let i = match self.provider_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.providers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.provider_list_state.select(Some(i));
    }

    /// 获取当前选中的 Provider 名称
    pub fn get_selected_provider(&self) -> Option<&String> {
        self.provider_list_state
            .selected()
            .and_then(|i| self.providers.get(i))
    }

    // === Provider 表单操作 ===

    /// 打开添加 Provider 表单
    pub fn open_add_provider_form(&mut self) {
        self.provider_form.clear();
        self.provider_form.title = "添加 Provider".to_string();
        self.provider_form_mode = ProviderFormMode::Add;
        self.provider_form.show();
        self.input_mode = InputMode::Editing;
    }

    /// 打开编辑 Provider 表单
    pub fn open_edit_provider_form(&mut self) {
        if let Some(provider_name) = self.get_selected_provider().cloned() {
            if let Ok(Some(provider)) = self.config_manager.opencode().get_provider(&provider_name) {
                self.provider_form.clear();
                self.provider_form.title = format!("编辑 Provider: {}", provider_name);
                self.provider_form_mode = ProviderFormMode::Edit;

                // 填充现有数据
                if let Some(field) = self.provider_form.fields.get_mut(0) {
                    field.set_value(&provider_name);
                }
                if let Some(field) = self.provider_form.fields.get_mut(1) {
                    field.set_value(&provider.options.api_key);
                }
                if let Some(field) = self.provider_form.fields.get_mut(2) {
                    field.set_value(&provider.options.base_url);
                }

                self.provider_form.show();
                self.input_mode = InputMode::Editing;
            }
        }
    }

    /// 关闭 Provider 表单
    pub fn close_provider_form(&mut self) {
        self.provider_form.hide();
        self.input_mode = InputMode::Normal;
    }

    /// 提交 Provider 表单
    pub fn submit_provider_form(&mut self) {
        if !self.provider_form.is_valid() {
            self.show_error("请填写所有必填字段");
            return;
        }

        let name = self.provider_form.get_value(0).unwrap_or("").to_string();
        let api_key = self.provider_form.get_value(1).unwrap_or("").to_string();
        let base_url = self.provider_form.get_value(2).unwrap_or("").to_string();

        let result = match self.provider_form_mode {
            ProviderFormMode::Add => {
                self.config_manager.opencode_mut().add_provider(
                    name.clone(),
                    base_url,
                    api_key,
                    None, // npm
                    None, // description
                )
            }
            ProviderFormMode::Edit => {
                // 编辑模式：更新现有 Provider 的元数据
                if let Some(old_name) = self.get_selected_provider().cloned() {
                    if old_name != name {
                        // 名称变化：删除旧的，创建新的
                        let _ = self.config_manager.opencode_mut().delete_provider(&old_name);
                        self.config_manager.opencode_mut().add_provider(
                            name.clone(),
                            base_url,
                            api_key,
                            None,
                            None,
                        )
                    } else {
                        // 名称不变：更新元数据
                        self.config_manager.opencode_mut().update_provider_metadata(
                            &name,
                            Some(base_url),
                            Some(api_key),
                            None,
                            None,
                        )
                    }
                } else {
                    Err("没有选中的 Provider".to_string())
                }
            }
        };

        match result {
            Ok(_) => {
                let action = if self.provider_form_mode == ProviderFormMode::Add { "添加" } else { "更新" };
                self.show_success(&format!("Provider {} 成功: {}", action, name));
                self.log_operation(format!("Provider {} 成功: {}", action, name), MessageType::Success);
                let _ = self.refresh_providers();
                self.close_provider_form();
            }
            Err(e) => {
                self.show_error(&format!("操作失败: {}", e));
            }
        }
    }

    // === 删除对话框操作 ===

    /// 打开删除确认对话框
    pub fn open_delete_dialog(&mut self) {
        if let Some(name) = self.get_selected_provider() {
            self.delete_dialog.message = format!("确定要删除 Provider \"{}\" 吗？\n此操作无法撤销。", name);
            self.delete_dialog.show();
        }
    }

    /// 确认删除 Provider
    pub fn confirm_delete_provider(&mut self) {
        if let Some(name) = self.get_selected_provider().cloned() {
            match self.config_manager.opencode_mut().delete_provider(&name) {
                Ok(_) => {
                    self.show_success(&format!("Provider 已删除: {}", name));
                    self.log_operation(format!("删除 Provider: {}", name), MessageType::Success);
                    let _ = self.refresh_providers();
                    // 调整选中项
                    if self.providers.is_empty() {
                        self.provider_list_state.select(None);
                    } else if let Some(i) = self.provider_list_state.selected() {
                        if i >= self.providers.len() {
                            self.provider_list_state.select(Some(self.providers.len() - 1));
                        }
                    }
                }
                Err(e) => {
                    self.show_error(&format!("删除失败: {}", e));
                }
            }
        }
        self.delete_dialog.hide();
    }

    // === 应用配置对话框操作 ===

    /// 打开应用配置对话框
    #[allow(dead_code)]
    pub fn open_apply_dialog(&mut self) {
        if let Some(name) = self.get_selected_provider() {
            self.apply_dialog.message = format!(
                "确定要应用 Provider \"{}\" 的配置吗？\n这将更新 OpenCode 的 provider 设置。",
                name
            );
            self.apply_dialog.show();
        }
    }

    /// 确认应用配置
    pub fn confirm_apply_provider(&mut self) {
        if let Some(name) = self.get_selected_provider().cloned() {
            match self.config_manager.apply_multiple_opencode_to_project(&vec![name.clone()]) {
                Ok(_) => {
                    self.show_success(&format!("配置已应用: {}", name));
                    self.log_operation(format!("应用 Provider 配置: {}", name), MessageType::Success);
                }
                Err(e) => {
                    self.show_error(&format!("应用配置失败: {}", e));
                }
            }
        }
        self.apply_dialog.hide();
    }

    // === 多选应用模式操作 ===

    /// 进入多选应用模式
    pub fn enter_multi_apply_mode(&mut self) {
        if self.providers.is_empty() {
            return;
        }
        self.is_multi_apply_mode = true;
        self.selected_providers.clear();
        // 默认选中当前高亮的 Provider
        if let Some(idx) = self.provider_list_state.selected() {
            if let Some(name) = self.providers.get(idx) {
                self.selected_providers.push(name.clone());
            }
        }
        self.multi_apply_list_state.select(Some(0));
    }

    /// 退出多选应用模式
    pub fn exit_multi_apply_mode(&mut self) {
        self.is_multi_apply_mode = false;
        self.selected_providers.clear();
    }

    /// 在多选模式下切换当前 Provider 的选择状态
    pub fn toggle_provider_selection(&mut self) {
        if let Some(idx) = self.multi_apply_list_state.selected() {
            if let Some(name) = self.providers.get(idx) {
                if self.selected_providers.contains(name) {
                    self.selected_providers.retain(|p| p != name);
                } else {
                    self.selected_providers.push(name.clone());
                }
            }
        }
    }

    /// 多选模式下选择下一个 Provider
    pub fn select_next_multi_apply(&mut self) {
        if self.providers.is_empty() {
            return;
        }
        let i = match self.multi_apply_list_state.selected() {
            Some(i) => {
                if i >= self.providers.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.multi_apply_list_state.select(Some(i));
    }

    /// 多选模式下选择上一个 Provider
    pub fn select_prev_multi_apply(&mut self) {
        if self.providers.is_empty() {
            return;
        }
        let i = match self.multi_apply_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.providers.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.multi_apply_list_state.select(Some(i));
    }

    /// 获取多选模式下当前高亮的 Provider 名称
    pub fn get_multi_apply_current(&self) -> Option<&String> {
        self.multi_apply_list_state
            .selected()
            .and_then(|i| self.providers.get(i))
    }

    /// 确认选择的 Provider，打开应用范围对话框
    pub fn confirm_selected_providers(&mut self) {
        if self.selected_providers.is_empty() {
            self.show_warning("请先选择要应用的 Provider");
            return;
        }
        // 显示所有选中的 Provider
        self.apply_scope_dialog.show_multiple(&self.selected_providers);
        self.is_multi_apply_mode = false;
    }

    /// 执行应用配置（从范围对话框确认后调用）
    pub fn execute_apply_config(&mut self) {
        if self.selected_providers.is_empty() {
            return;
        }

        // 提取需要的值，避免借用冲突
        let apply_to_global = self.apply_scope_dialog.apply_to_global;
        let apply_to_project = self.apply_scope_dialog.apply_to_project;
        let target_description = self.apply_scope_dialog.get_target_description();

        let provider_names: Vec<String> = self.selected_providers.clone();

        // 应用到全局
        if apply_to_global {
            match self
                .config_manager
                .apply_multiple_opencode_to_global(&provider_names)
            {
                Ok(_) => {
                    self.show_success(&format!(
                        "已应用到全局配置 ({} 个 Provider)",
                        provider_names.len()
                    ));
                }
                Err(e) => {
                    self.show_error(&format!("应用到全局配置失败: {}", e));
                }
            }
        }

        // 应用到项目
        if apply_to_project {
            match self
                .config_manager
                .apply_multiple_opencode_to_project(&provider_names)
            {
                Ok(_) => {
                    self.show_success(&format!(
                        "已应用到当前项目 ({} 个 Provider)",
                        provider_names.len()
                    ));
                }
                Err(e) => {
                    self.show_error(&format!("应用到项目失败: {}", e));
                }
            }
        }

        // 记录操作日志
        self.log_operation(
            format!(
                "应用 {} 个 Provider 到: {}",
                provider_names.len(),
                target_description
            ),
            MessageType::Success,
        );

        // 清理状态
        self.selected_providers.clear();
        self.apply_scope_dialog.hide();
    }

    /// 获取已选择的 Provider 数量
    pub fn get_selected_count(&self) -> usize {
        self.selected_providers.len()
    }

    /// 获取 Provider 总数
    pub fn get_provider_count(&self) -> usize {
        self.providers.len()
    }

    /// 检查 Provider 是否被选中
    pub fn is_provider_selected(&self, name: &str) -> bool {
        self.selected_providers.contains(&name.to_string())
    }

    // === Model 操作 ===

    /// 刷新 Model 列表（基于当前选中的 Provider）
    pub fn refresh_models(&mut self) {
        if let Some(provider_name) = self.get_selected_provider().cloned() {
            if let Ok(models) = self.config_manager.opencode().get_models(&provider_name) {
                let mut model_list: Vec<String> = models.keys().cloned().collect();
                model_list.sort();
                self.models = model_list;

                // 调整选中状态
                if !self.models.is_empty() {
                    if self.model_list_state.selected().is_none() {
                        self.model_list_state.select(Some(0));
                    } else if let Some(i) = self.model_list_state.selected() {
                        if i >= self.models.len() {
                            self.model_list_state.select(Some(self.models.len() - 1));
                        }
                    }
                } else {
                    self.model_list_state.select(None);
                }
            } else {
                self.models = Vec::new();
                self.model_list_state.select(None);
            }
        } else {
            self.models = Vec::new();
            self.model_list_state.select(None);
        }
    }

    /// Model 列表导航
    pub fn select_next_model(&mut self) {
        if self.models.is_empty() {
            return;
        }
        let i = match self.model_list_state.selected() {
            Some(i) => {
                if i >= self.models.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.model_list_state.select(Some(i));
    }

    pub fn select_prev_model(&mut self) {
        if self.models.is_empty() {
            return;
        }
        let i = match self.model_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.models.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.model_list_state.select(Some(i));
    }

    /// 获取当前选中的 Model 名称
    pub fn get_selected_model(&self) -> Option<&String> {
        self.model_list_state
            .selected()
            .and_then(|i| self.models.get(i))
    }

    /// 设置 Provider Tab 焦点区域
    pub fn set_provider_tab_focus(&mut self, focus: usize) {
        self.provider_tab_focus = focus.min(1);
    }

    // === Model 表单操作 ===

    /// 打开添加 Model 表单
    pub fn open_add_model_form(&mut self) {
        if self.get_selected_provider().is_none() {
            self.show_error("请先选择一个 Provider");
            return;
        }
        self.model_form.clear();
        self.model_form.title = "添加 Model".to_string();
        self.model_form.show();
        self.input_mode = InputMode::Editing;
    }

    /// 关闭 Model 表单
    pub fn close_model_form(&mut self) {
        self.model_form.hide();
        self.input_mode = InputMode::Normal;
    }

    /// 提交 Model 表单
    pub fn submit_model_form(&mut self) {
        if !self.model_form.is_valid() {
            self.show_error("请填写 Model ID");
            return;
        }

        let model_id = self.model_form.get_value(0).unwrap_or("").to_string();

        if let Some(provider_name) = self.get_selected_provider().cloned() {
            // 创建 ModelInfo
            let model_info = crate::config::models::OpenCodeModelInfo {
                name: model_id.clone(),
                limit: None,
                model_detection: None,
            };

            let result = self.config_manager.opencode_mut().add_model(
                &provider_name,
                model_id.clone(),
                model_info,
            );

            match result {
                Ok(_) => {
                    self.show_success(&format!("Model 添加成功: {}", model_id));
                    self.log_operation(format!("添加 Model: {}", model_id), MessageType::Success);
                    self.refresh_models();
                    self.close_model_form();
                }
                Err(e) => {
                    self.show_error(&format!("添加失败: {}", e));
                }
            }
        }
    }

    // === Model 删除对话框操作 ===

    /// 打开 Model 删除对话框
    pub fn open_model_delete_dialog(&mut self) {
        if let Some(model_name) = self.get_selected_model() {
            self.model_delete_dialog.message =
                format!("确定要删除 Model \"{}\" 吗？\n此操作无法撤销。", model_name);
            self.model_delete_dialog.show();
        }
    }

    /// 确认删除 Model
    pub fn confirm_delete_model(&mut self) {
        if let (Some(provider_name), Some(model_name)) =
            (self.get_selected_provider().cloned(), self.get_selected_model().cloned())
        {
            match self.config_manager.opencode_mut().delete_model(&provider_name, &model_name) {
                Ok(_) => {
                    self.show_success(&format!("Model 已删除: {}", model_name));
                    self.log_operation(format!("删除 Model: {}", model_name), MessageType::Success);
                    self.refresh_models();
                }
                Err(e) => {
                    self.show_error(&format!("删除失败: {}", e));
                }
            }
        }
        self.model_delete_dialog.hide();
    }

    // === 站点模型获取 ===

    /// 准备获取站点模型（显示加载状态）
    pub fn prepare_fetch_site_models(&mut self) -> Option<(String, String)> {
        let provider_name = match self.get_selected_provider().cloned() {
            Some(name) => name,
            None => {
                self.show_error("请先选择一个 Provider");
                return None;
            }
        };

        // 获取 Provider 信息
        if let Ok(Some(provider)) = self.config_manager.opencode().get_provider(&provider_name) {
            let base_url = provider.options.base_url.clone();
            let api_key = provider.options.api_key.clone();

            // 显示加载状态
            self.model_select_dialog.show_loading("正在获取站点可用模型列表...");

            // 预选已有的模型
            self.model_select_dialog.set_selected(&self.models);

            Some((base_url, api_key))
        } else {
            self.show_error("无法获取 Provider 信息");
            None
        }
    }

    /// 设置获取到的站点模型列表
    pub fn set_fetched_models(&mut self, models: Vec<String>) {
        // 过滤掉已添加的模型
        let existing: std::collections::HashSet<_> = self.models.iter().collect();
        let new_models: Vec<String> = models
            .into_iter()
            .filter(|m| !existing.contains(m))
            .collect();

        if new_models.is_empty() {
            self.model_select_dialog.hide();
            self.show_info("所有可用模型都已添加");
            return;
        }

        self.model_select_dialog.set_items(new_models);
        self.model_select_dialog.show();
    }

    /// 设置获取模型失败
    pub fn set_fetch_models_error(&mut self, error: String) {
        self.model_select_dialog.show_error(&error);
    }

    /// 关闭模型选择对话框
    pub fn close_model_select_dialog(&mut self) {
        self.model_select_dialog.hide();
    }

    /// 确认添加选中的模型
    pub fn confirm_add_selected_models(&mut self) {
        let selected_models = self.model_select_dialog.get_selected_items();

        if selected_models.is_empty() {
            self.show_error("请至少选择一个模型");
            return;
        }

        if let Some(provider_name) = self.get_selected_provider().cloned() {
            let mut success_count = 0;
            let mut fail_count = 0;

            for model_id in &selected_models {
                let model_info = crate::config::models::OpenCodeModelInfo {
                    name: model_id.clone(),
                    limit: None,
                    model_detection: None,
                };

                match self.config_manager.opencode_mut().add_model(
                    &provider_name,
                    model_id.clone(),
                    model_info,
                ) {
                    Ok(_) => success_count += 1,
                    Err(_) => fail_count += 1,
                }
            }

            if success_count > 0 {
                self.show_success(&format!("成功添加 {} 个模型", success_count));
                self.log_operation(
                    format!("批量添加 {} 个模型到 {}", success_count, provider_name),
                    MessageType::Success,
                );
            }

            if fail_count > 0 {
                self.show_error(&format!("{} 个模型添加失败（可能已存在）", fail_count));
            }

            self.refresh_models();
        }

        self.model_select_dialog.hide();
    }

    // === 搜索功能 ===

    /// 进入搜索模式
    pub fn enter_search_mode(&mut self) {
        self.search_active = true;
        self.search_query.clear();
        self.input_mode = InputMode::Editing;
    }

    /// 退出搜索模式
    pub fn exit_search_mode(&mut self) {
        self.search_active = false;
        self.input_mode = InputMode::Normal;
    }

    /// 处理搜索输入
    pub fn handle_search_input(&mut self, c: char) {
        self.search_query.push(c);
    }

    /// 处理搜索退格
    pub fn handle_search_backspace(&mut self) {
        self.search_query.pop();
    }

    /// 清除搜索
    pub fn clear_search(&mut self) {
        self.search_query.clear();
    }

    /// 获取过滤后的模型列表
    pub fn get_filtered_models(&self) -> Vec<&String> {
        if self.search_query.is_empty() {
            self.models.iter().collect()
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.models
                .iter()
                .filter(|m| m.to_lowercase().contains(&query_lower))
                .collect()
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new().expect("Failed to create App")
    }
}
