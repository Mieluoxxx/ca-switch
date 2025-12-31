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
        AppTab::Models => handle_model_tab_key(app, key),
        AppTab::Backup => handle_backup_tab_key(app, key),
        AppTab::Status => handle_status_tab_key(app, key),
    }
}

/// Provider Tab 按键处理
fn handle_provider_tab_key(app: &mut App, key: KeyEvent) -> bool {
    // 如果处于多选应用模式，优先处理多选相关的按键
    if app.is_multi_apply_mode {
        return handle_multi_apply_mode(app, key);
    }

    match key.code {
        // 导航
        KeyCode::Down | KeyCode::Char('j') => {
            app.select_next_provider();
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.select_prev_provider();
            true
        }
        // 添加 Provider
        KeyCode::Char('a') => {
            app.open_add_provider_form();
            true
        }
        // 编辑 Provider
        KeyCode::Char('e') => {
            app.open_edit_provider_form();
            true
        }
        // 删除 Provider
        KeyCode::Char('d') => {
            app.open_delete_dialog();
            true
        }
        // 应用配置 - 进入多选模式
        KeyCode::Enter => {
            app.enter_multi_apply_mode();
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

/// Model Tab 按键处理
fn handle_model_tab_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        // 切换焦点区域 (Tab 键在 Provider 和 Model 列表之间切换)
        KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            app.toggle_model_tab_focus();
            true
        }
        // 导航
        KeyCode::Down | KeyCode::Char('j') => {
            if app.model_tab_focus == 0 {
                app.select_next_model_provider();
            } else {
                app.select_next_model();
            }
            true
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.model_tab_focus == 0 {
                app.select_prev_model_provider();
            } else {
                app.select_prev_model();
            }
            true
        }
        // 选择 Provider (加载其 Model 列表)
        KeyCode::Enter => {
            if app.model_tab_focus == 0 {
                app.select_model_tab_provider();
            }
            true
        }
        // 添加 Model
        KeyCode::Char('a') => {
            app.open_add_model_form();
            true
        }
        // 删除 Model
        KeyCode::Char('d') => {
            if app.model_tab_focus == 1 {
                app.open_model_delete_dialog();
            }
            true
        }
        // 获取站点模型 (t = test/fetch)
        KeyCode::Char('t') => {
            // 返回需要异步获取的信息，由主循环处理
            if let Some((base_url, api_key)) = app.prepare_fetch_site_models() {
                // 同步获取模型（因为 TUI 主循环不是 async）
                // 使用 tokio 的 block_on 或者显示提示让用户等待
                fetch_site_models_sync(app, &base_url, &api_key);
            }
            true
        }
        // 搜索模型
        KeyCode::Char('/') => {
            if app.model_tab_focus == 1 && app.model_tab_provider.is_some() {
                app.enter_search_mode();
            }
            true
        }
        _ => false,
    }
}

/// 同步获取站点模型
fn fetch_site_models_sync(app: &mut App, base_url: &str, api_key: &str) {
    use crate::config::Detector;
    use crate::config::SiteDetectionResult;
    use std::panic;
    use tokio::runtime::Handle;

    let base_url = base_url.to_string();
    let api_key = api_key.to_string();

    // 使用 catch_unwind 捕获潜在的 panic
    let fetch_result: Result<SiteDetectionResult, String> = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        // 使用当前 tokio runtime 的 handle 来执行异步任务
        match Handle::try_current() {
            Ok(handle) => {
                // 如果已经在 tokio runtime 中，使用 block_in_place
                Ok(tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        let detector = Detector::new();
                        detector.detect_site(&base_url, &api_key).await
                    })
                }))
            }
            Err(_) => {
                // 如果没有 runtime，创建一个新的
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| format!("运行时错误: {}", e))?;

                Ok(rt.block_on(async {
                    let detector = Detector::new();
                    detector.detect_site(&base_url, &api_key).await
                }))
            }
        }
    }))
    .map_err(|_| "获取模型时发生错误".to_string())
    .and_then(|r| r);

    match fetch_result {
        Ok(result) => {
            if result.is_available && !result.available_models.is_empty() {
                app.set_fetched_models(result.available_models);
            } else if let Some(error) = result.error_message {
                app.set_fetch_models_error(error);
            } else {
                app.set_fetch_models_error("站点不可用或无法获取模型列表".to_string());
            }
        }
        Err(e) => {
            app.set_fetch_models_error(e);
        }
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
