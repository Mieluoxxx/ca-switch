// 全局键盘事件处理器

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::{
    app::App,
    types::{AppTab, InputMode},
    ui::DialogResult,
};

/// 处理键盘事件
/// 返回 true 表示事件已处理
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    // 如果正在显示帮助，任意键关闭
    if app.help_visible {
        app.help_visible = false;
        return true;
    }

    // 优先处理对话框
    if app.delete_dialog.visible {
        return handle_delete_dialog(app, key);
    }

    if app.apply_dialog.visible {
        return handle_apply_dialog(app, key);
    }

    if app.apply_scope_dialog.visible {
        return handle_apply_scope_dialog(app, key);
    }

    // 处理表单输入
    if app.provider_form.visible {
        return handle_provider_form(app, key);
    }

    if app.model_form.visible {
        return handle_model_form(app, key);
    }

    // Model 删除对话框
    if app.model_delete_dialog.visible {
        return handle_model_delete_dialog(app, key);
    }

    // 模型多选对话框
    if app.model_select_dialog.visible {
        return handle_model_select_dialog(app, key);
    }

    // MCP 删除对话框
    if app.mcp_delete_dialog.visible {
        return handle_mcp_delete_dialog(app, key);
    }

    // MCP 应用范围对话框
    if app.mcp_apply_scope_dialog.visible {
        return handle_mcp_apply_scope_dialog(app, key);
    }

    // MCP 表单
    if app.mcp_form.visible {
        return handle_mcp_form(app, key);
    }

    // 搜索模式
    if app.search_active {
        return handle_search_mode(app, key);
    }

    // 根据输入模式分发事件
    match app.input_mode {
        InputMode::Normal => handle_normal_mode(app, key),
        InputMode::Editing => handle_editing_mode(app, key),
    }
}

/// 处理删除对话框
fn handle_delete_dialog(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.delete_dialog.toggle_selection();
            true
        }
        KeyCode::Enter => {
            let result = app.delete_dialog.confirm();
            if result == DialogResult::Confirm {
                app.confirm_delete_provider();
            } else {
                app.delete_dialog.hide();
            }
            true
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.delete_dialog.hide();
            true
        }
        KeyCode::Char('y') => {
            // 快捷键确认
            app.delete_dialog.selected = 0;
            app.confirm_delete_provider();
            true
        }
        KeyCode::Char('n') => {
            // 快捷键取消
            app.delete_dialog.hide();
            true
        }
        _ => true,
    }
}

/// 处理应用配置对话框
fn handle_apply_dialog(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.apply_dialog.toggle_selection();
            true
        }
        KeyCode::Enter => {
            let result = app.apply_dialog.confirm();
            if result == DialogResult::Confirm {
                app.confirm_apply_provider();
            } else {
                app.apply_dialog.hide();
            }
            true
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.apply_dialog.hide();
            true
        }
        KeyCode::Char('y') => {
            app.apply_dialog.selected = 0;
            app.confirm_apply_provider();
            true
        }
        KeyCode::Char('n') => {
            app.apply_dialog.hide();
            true
        }
        _ => true,
    }
}

/// 处理应用范围选择对话框
fn handle_apply_scope_dialog(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.apply_scope_dialog.toggle_option();
            true
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.apply_scope_dialog.selected_option =
                (app.apply_scope_dialog.selected_option + 1) % 3;
            app.apply_scope_dialog.update_state_from_option();
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.apply_scope_dialog.selected_option = if app.apply_scope_dialog.selected_option == 0
            {
                2
            } else {
                app.apply_scope_dialog.selected_option - 1
            };
            app.apply_scope_dialog.update_state_from_option();
            true
        }
        KeyCode::Enter => {
            app.execute_apply_config();
            true
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.apply_scope_dialog.hide();
            true
        }
        _ => true,
    }
}

/// 处理 Provider 表单输入
fn handle_provider_form(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.close_provider_form();
            true
        }
        KeyCode::Enter => {
            app.submit_provider_form();
            true
        }
        KeyCode::Tab => {
            app.provider_form.focus_next();
            true
        }
        KeyCode::BackTab => {
            app.provider_form.focus_prev();
            true
        }
        KeyCode::Backspace => {
            app.provider_form.handle_backspace();
            true
        }
        KeyCode::Delete => {
            app.provider_form.handle_delete();
            true
        }
        KeyCode::Char(c) => {
            app.provider_form.handle_input(c);
            true
        }
        _ => true,
    }
}

/// 处理正常模式的按键
fn handle_normal_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // 退出
        KeyCode::Char('q') => {
            app.quit();
            true
        }
        // Ctrl+C 退出
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.quit();
            true
        }
        // Tab 切换
        KeyCode::Tab => {
            app.next_tab();
            true
        }
        // Shift+Tab 反向切换
        KeyCode::BackTab => {
            app.prev_tab();
            true
        }
        // 显示帮助
        KeyCode::Char('?') => {
            app.toggle_help();
            true
        }
        // 根据当前 Tab 分发
        _ => handle_tab_specific_key(app, key),
    }
}

/// 处理编辑模式的按键
fn handle_editing_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Esc 退出编辑模式
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            true
        }
        // 其他按键由具体表单处理
        _ => false,
    }
}

/// 处理 Tab 特定的按键
fn handle_tab_specific_key(app: &mut App, key: KeyEvent) -> bool {
    match app.current_tab {
        AppTab::Providers => handle_provider_tab_key(app, key),
        AppTab::Mcp => handle_mcp_tab_key(app, key),
        AppTab::Backup => handle_backup_tab_key(app, key),
        AppTab::Status => handle_status_tab_key(app, key),
    }
}

/// Provider Tab 按键处理（三栏布局：Provider列表 + Model列表 + 详情）
fn handle_provider_tab_key(app: &mut App, key: KeyEvent) -> bool {
    // 如果处于多选应用模式，优先处理多选相关的按键
    if app.is_multi_apply_mode {
        return handle_multi_apply_mode(app, key);
    }

    match key.code {
        // 焦点切换：h/l 或 左右方向键
        KeyCode::Char('h') | KeyCode::Left => {
            if app.provider_tab_focus > 0 {
                app.set_provider_tab_focus(0);
            }
            true
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.provider_tab_focus == 0 && app.get_selected_provider().is_some() {
                app.set_provider_tab_focus(1);
                // 确保刷新 Model 列表
                app.refresh_models();
            }
            true
        }
        // 导航：根据当前焦点选择不同的列表
        KeyCode::Down | KeyCode::Char('j') => {
            if app.provider_tab_focus == 0 {
                app.select_next_provider();
                app.refresh_models(); // 立即刷新 Model 列表
            } else {
                app.select_next_model();
            }
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.provider_tab_focus == 0 {
                app.select_prev_provider();
                app.refresh_models(); // 立即刷新 Model 列表
            } else {
                app.select_prev_model();
            }
            true
        }
        // 添加：根据焦点决定添加 Provider 还是 Model
        KeyCode::Char('a') => {
            if app.provider_tab_focus == 0 {
                app.open_add_provider_form();
            } else {
                app.open_add_model_form();
            }
            true
        }
        // 编辑 Provider（仅在 Provider 焦点时）
        KeyCode::Char('e') => {
            if app.provider_tab_focus == 0 {
                app.open_edit_provider_form();
            }
            true
        }
        // 删除：根据焦点决定删除 Provider 还是 Model
        KeyCode::Char('d') => {
            if app.provider_tab_focus == 0 {
                app.open_delete_dialog();
            } else {
                app.open_model_delete_dialog();
            }
            true
        }
        // 应用配置 - 进入多选模式（仅在 Provider 焦点时）
        KeyCode::Enter => {
            if app.provider_tab_focus == 0 {
                app.enter_multi_apply_mode();
            }
            true
        }
        // 获取站点模型（仅在 Model 焦点时）
        KeyCode::Char('t') => {
            if app.provider_tab_focus == 1 || app.get_selected_provider().is_some() {
                if let Some((base_url, api_key)) = app.prepare_fetch_site_models() {
                    fetch_site_models_sync(app, &base_url, &api_key);
                }
            }
            true
        }
        // 搜索模型
        KeyCode::Char('/') => {
            if app.get_selected_provider().is_some() {
                app.set_provider_tab_focus(1); // 切换到 Model 列表
                app.enter_search_mode();
            }
            true
        }
        _ => false,
    }
}

/// 多选应用模式按键处理
fn handle_multi_apply_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // 导航
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_multi_apply();
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev_multi_apply();
            true
        }
        // 切换选择状态
        KeyCode::Char(' ') => {
            app.toggle_provider_selection();
            true
        }
        // 确认选择，打开应用范围对话框
        KeyCode::Enter => {
            app.confirm_selected_providers();
            true
        }
        // 取消多选模式
        KeyCode::Esc => {
            app.exit_multi_apply_mode();
            true
        }
        // 快捷键：全选
        KeyCode::Char('A') => {
            // 选择所有 Provider
            app.selected_providers = app.providers.clone();
            true
        }
        // 快捷键：清空选择
        KeyCode::Char('c') => {
            app.selected_providers.clear();
            true
        }
        _ => false,
    }
}

/// 处理 Model 表单输入
fn handle_model_form(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.close_model_form();
            true
        }
        KeyCode::Enter => {
            app.submit_model_form();
            true
        }
        KeyCode::Tab => {
            app.model_form.focus_next();
            true
        }
        KeyCode::BackTab => {
            app.model_form.focus_prev();
            true
        }
        KeyCode::Backspace => {
            app.model_form.handle_backspace();
            true
        }
        KeyCode::Delete => {
            app.model_form.handle_delete();
            true
        }
        KeyCode::Char(c) => {
            app.model_form.handle_input(c);
            true
        }
        _ => true,
    }
}

/// 处理 Model 删除对话框
fn handle_model_delete_dialog(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.model_delete_dialog.toggle_selection();
            true
        }
        KeyCode::Enter => {
            let result = app.model_delete_dialog.confirm();
            if result == DialogResult::Confirm {
                app.confirm_delete_model();
            } else {
                app.model_delete_dialog.hide();
            }
            true
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.model_delete_dialog.hide();
            true
        }
        KeyCode::Char('y') => {
            app.model_delete_dialog.selected = 0;
            app.confirm_delete_model();
            true
        }
        KeyCode::Char('n') => {
            app.model_delete_dialog.hide();
            true
        }
        _ => true,
    }
}

/// 同步获取站点模型
fn fetch_site_models_sync(app: &mut App, base_url: &str, api_key: &str) {
    use crate::config::Detector;
    use crate::config::SiteDetectionResult;
    use tokio::runtime::{Builder, Runtime};

    let base_url = base_url.to_string();
    let api_key = api_key.to_string();

    let runtime_result: Result<Runtime, String> = std::panic::catch_unwind(|| {
        Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("运行时创建失败: {}", e))
    })
    .map_err(|_| "获取模型时发生错误".to_string())
    .and_then(|r| r);

    let fetch_result = match runtime_result {
        Ok(rt) => rt.block_on(async {
            let detector = Detector::new();
            detector.detect_site(&base_url, &api_key).await
        }),
        Err(_) => SiteDetectionResult {
            detected_at: chrono::Utc::now().to_rfc3339(),
            is_available: false,
            api_key_valid: false,
            available_models: Vec::new(),
            response_time_ms: None,
            error_message: Some("获取模型时发生错误".to_string()),
        },
    };

    let result = fetch_result;
    if result.is_available && !result.available_models.is_empty() {
        app.set_fetched_models(result.available_models);
    } else if let Some(error) = result.error_message {
        app.set_fetch_models_error(error);
    } else {
        app.set_fetch_models_error("站点不可用或无法获取模型列表".to_string());
    }
}

/// Backup Tab 按键处理
fn handle_backup_tab_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('b') => {
            app.show_info("备份功能开发中...");
            true
        }
        KeyCode::Char('r') => {
            app.show_info("恢复功能开发中...");
            true
        }
        _ => false,
    }
}

/// Status Tab 按键处理
fn handle_status_tab_key(_app: &mut App, _key: KeyEvent) -> bool {
    // Status Tab 目前没有特殊操作
    false
}

/// 处理模型多选对话框
fn handle_model_select_dialog(app: &mut App, key: KeyEvent) -> bool {
    // 如果在搜索模式
    if app.model_select_dialog.search_mode {
        match key.code {
            KeyCode::Esc => {
                app.model_select_dialog.exit_search_mode();
                true
            }
            KeyCode::Enter => {
                app.model_select_dialog.exit_search_mode();
                true
            }
            KeyCode::Backspace => {
                app.model_select_dialog.handle_search_backspace();
                true
            }
            KeyCode::Char(c) => {
                app.model_select_dialog.handle_search_input(c);
                true
            }
            _ => true,
        }
    } else {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                app.close_model_select_dialog();
                true
            }
            KeyCode::Enter => {
                app.confirm_add_selected_models();
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.model_select_dialog.select_next();
                true
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.model_select_dialog.select_prev();
                true
            }
            KeyCode::Char(' ') => {
                app.model_select_dialog.toggle_current();
                true
            }
            KeyCode::Char('a') => {
                app.model_select_dialog.select_all();
                true
            }
            KeyCode::Char('/') => {
                app.model_select_dialog.enter_search_mode();
                true
            }
            KeyCode::Char('c') => {
                app.model_select_dialog.clear_search();
                true
            }
            _ => true,
        }
    }
}

/// 处理搜索模式
fn handle_search_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Esc => {
            app.exit_search_mode();
            app.clear_search();
            true
        }
        KeyCode::Enter => {
            app.exit_search_mode();
            true
        }
        KeyCode::Backspace => {
            app.handle_search_backspace();
            true
        }
        KeyCode::Char(c) => {
            app.handle_search_input(c);
            true
        }
        _ => true,
    }
}

// ============================================================================
// MCP Tab 事件处理
// ============================================================================

/// MCP Tab 按键处理
fn handle_mcp_tab_key(app: &mut App, key: KeyEvent) -> bool {
    // 如果处于多选同步模式
    if app.is_mcp_multi_sync_mode {
        return handle_mcp_multi_sync_mode(app, key);
    }

    match key.code {
        // 导航
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_mcp_server();
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev_mcp_server();
            true
        }
        // 添加 MCP 服务器
        KeyCode::Char('a') => {
            app.open_add_mcp_form();
            true
        }
        // 编辑 MCP 服务器
        KeyCode::Char('e') => {
            app.open_edit_mcp_form();
            true
        }
        // 删除 MCP 服务器
        KeyCode::Char('d') => {
            app.open_mcp_delete_dialog();
            true
        }
        // 切换启用状态
        KeyCode::Char(' ') => {
            app.toggle_mcp_server_enabled();
            true
        }
        // 同步配置 - 进入多选模式
        KeyCode::Enter => {
            app.enter_mcp_multi_sync_mode();
            true
        }
        _ => false,
    }
}

/// MCP 多选同步模式按键处理
fn handle_mcp_multi_sync_mode(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // 导航
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_mcp_multi();
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev_mcp_multi();
            true
        }
        // 切换选择状态
        KeyCode::Char(' ') => {
            app.toggle_mcp_server_selection();
            true
        }
        // 确认选择，打开同步范围对话框
        KeyCode::Enter => {
            app.confirm_selected_mcp_servers();
            true
        }
        // 取消多选模式
        KeyCode::Esc => {
            app.exit_mcp_multi_sync_mode();
            true
        }
        // 全选
        KeyCode::Char('A') => {
            app.select_all_mcp_servers();
            true
        }
        // 清空选择
        KeyCode::Char('C') => {
            app.clear_mcp_selection();
            true
        }
        _ => false,
    }
}

/// 处理 MCP 删除对话框
fn handle_mcp_delete_dialog(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.mcp_delete_dialog.toggle_selection();
            true
        }
        KeyCode::Enter => {
            let result = app.mcp_delete_dialog.confirm();
            if result == DialogResult::Confirm {
                app.confirm_delete_mcp_server();
            } else {
                app.mcp_delete_dialog.hide();
            }
            true
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mcp_delete_dialog.hide();
            true
        }
        KeyCode::Char('y') => {
            app.mcp_delete_dialog.selected = 0;
            app.confirm_delete_mcp_server();
            true
        }
        KeyCode::Char('n') => {
            app.mcp_delete_dialog.hide();
            true
        }
        _ => true,
    }
}

/// 处理 MCP 应用范围对话框
fn handle_mcp_apply_scope_dialog(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.mcp_apply_scope_dialog.toggle_option();
            true
        }
        KeyCode::Enter => {
            app.execute_mcp_sync();
            true
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mcp_apply_scope_dialog.hide();
            true
        }
        _ => true,
    }
}

/// 处理 MCP 表单输入（纯 JSON 编辑器模式）
fn handle_mcp_form(app: &mut App, key: KeyEvent) -> bool {
    // 如果正在显示关闭确认对话框
    if app.mcp_form.confirm_close {
        return handle_mcp_form_close_confirm(app, key);
    }

    match key.code {
        KeyCode::Esc => {
            // 检查是否有未保存的修改
            if app.mcp_form.request_close() {
                app.close_mcp_form();
            }
            // 如果有修改，request_close 会设置 confirm_close = true
            true
        }
        // Ctrl+S 保存
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.submit_mcp_form();
            true
        }
        // Ctrl+T 切换服务器类型模板
        KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.mcp_form.toggle_mode();
            true
        }
        // Ctrl+D 清空内容（切换到空模板）
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.mcp_form.clear_to_empty();
            true
        }
        // Enter 换行
        KeyCode::Enter => {
            app.mcp_form.handle_enter();
            true
        }
        // 方向键导航
        KeyCode::Up => {
            app.mcp_form.cursor_up();
            true
        }
        KeyCode::Down => {
            app.mcp_form.cursor_down();
            true
        }
        KeyCode::Left => {
            app.mcp_form.cursor_left();
            true
        }
        KeyCode::Right => {
            app.mcp_form.cursor_right();
            true
        }
        KeyCode::Backspace => {
            app.mcp_form.handle_backspace();
            true
        }
        KeyCode::Delete => {
            app.mcp_form.handle_delete();
            true
        }
        KeyCode::Char(c) => {
            app.mcp_form.handle_input(c);
            true
        }
        _ => true,
    }
}

/// 处理 MCP 表单关闭确认对话框
fn handle_mcp_form_close_confirm(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // Y - 确认放弃修改
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.mcp_form.confirm_discard();
            app.close_mcp_form();
            true
        }
        // N 或 Esc - 取消，继续编辑
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.mcp_form.cancel_close();
            true
        }
        // Ctrl+S - 保存
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.mcp_form.cancel_close();
            app.submit_mcp_form();
            true
        }
        _ => true,
    }
}
