// 主布局渲染

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::tui::{
    app::App,
    theme::Theme,
    types::{AppTab, MessageType},
};

// ============================================================================
// 布局常量
// ============================================================================

/// Provider Tab 三栏布局比例
mod provider_tab_layout {
    pub const PROVIDER_LIST_PERCENT: u16 = 25;
    pub const MODEL_LIST_PERCENT: u16 = 30;
    pub const DETAIL_PANEL_PERCENT: u16 = 45;
}

/// MCP Tab 两栏布局比例
mod mcp_tab_layout {
    pub const SERVER_LIST_PERCENT: u16 = 35;
    pub const DETAIL_PANEL_PERCENT: u16 = 65;
}

/// 多选模式布局比例
mod multi_select_layout {
    pub const LIST_PERCENT: u16 = 40;
    pub const DETAIL_PERCENT: u16 = 60;
}

/// 帮助弹窗尺寸
mod help_popup_layout {
    pub const WIDTH_PERCENT: u16 = 60;
    pub const HEIGHT_PERCENT: u16 = 70;
}

/// 渲染主界面
pub fn render(frame: &mut Frame, app: &mut App, theme: &Theme) {
    // 清理过期消息
    app.cleanup_expired_messages();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(10),   // Content
            Constraint::Length(4), // Footer (2行内容 + 2行边框)
        ])
        .split(frame.area());

    render_header(frame, theme, chunks[0]);
    render_tabs(frame, app, theme, chunks[1]);
    render_content(frame, app, theme, chunks[2]);
    render_footer(frame, app, theme, chunks[3]);

    // 覆盖层 - 对话框和表单
    let full_area = frame.area();

    // Provider 表单
    app.provider_form.render(frame, theme, full_area);

    // Model 表单
    app.model_form.render(frame, theme, full_area);

    // 删除确认对话框
    app.delete_dialog.render(frame, theme, full_area);

    // 应用配置对话框
    app.apply_dialog.render(frame, theme, full_area);

    // 应用范围选择对话框
    app.apply_scope_dialog.render(frame, theme, full_area);

    // Model 删除对话框
    app.model_delete_dialog.render(frame, theme, full_area);

    // 模型多选对话框
    app.model_select_dialog.render(frame, theme, full_area);

    // MCP 表单
    app.mcp_form.render(frame, theme, full_area);

    // MCP 删除对话框
    app.mcp_delete_dialog.render(frame, theme, full_area);

    // MCP 同步范围对话框
    app.mcp_apply_scope_dialog.render(frame, theme, full_area);

    // 帮助弹窗
    if app.help_visible {
        render_help_popup(frame, theme, full_area);
    }

    // Toast 消息
    if let Some(ref msg) = app.status_message {
        render_toast(frame, msg, theme, full_area);
    }
}

/// 渲染顶部标题栏
fn render_header(frame: &mut Frame, theme: &Theme, area: Rect) {
    let version = env!("CARGO_PKG_VERSION");
    let title = format!(" 🚀 opcd v{} ", version);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(title, theme.title_style()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Paragraph::new("Coding Agent 配置管理工具").style(Style::default().fg(theme.muted));
    frame.render_widget(text, inner);
}

/// 渲染 Tab 栏
fn render_tabs(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let titles: Vec<Line> = AppTab::all()
        .iter()
        .map(|tab| {
            let icon = match tab {
                AppTab::Providers => "🔌",
                AppTab::Mcp => "🧩",
                AppTab::Backup => "💾",
                AppTab::Status => "📊",
            };
            Line::from(format!(" {} {} ", icon, tab.title()))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title(Span::styled(" 功能模块 ", theme.title_style())),
        )
        .select(app.current_tab.index())
        .highlight_style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider("|");

    frame.render_widget(tabs, area);
}

/// 渲染内容区域
fn render_content(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    match app.current_tab {
        AppTab::Providers => render_providers_tab(frame, app, theme, area),
        AppTab::Mcp => render_mcp_tab(frame, app, theme, area),
        AppTab::Backup => render_backup_tab(frame, app, theme, area),
        AppTab::Status => render_status_tab(frame, app, theme, area),
    }
}

/// 渲染 Provider Tab（三栏布局：Provider列表 + Model列表 + 详情面板）
fn render_providers_tab(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    // 多选模式使用两栏布局
    if app.is_multi_apply_mode {
        render_providers_multi_select_mode(frame, app, theme, area);
        return;
    }

    // 三栏布局：Provider列表 + Model列表 + 详情面板
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(provider_tab_layout::PROVIDER_LIST_PERCENT),
            Constraint::Percentage(provider_tab_layout::MODEL_LIST_PERCENT),
            Constraint::Percentage(provider_tab_layout::DETAIL_PANEL_PERCENT),
        ])
        .split(area);

    // 左侧: Provider 列表
    let provider_border_type = if app.provider_tab_focus == 0 {
        theme.active_border_type()
    } else {
        BorderType::Plain
    };

    let provider_items: Vec<ListItem> = app
        .providers
        .iter()
        .map(|name| {
            ListItem::new(Line::from(vec![
                Span::raw(" 🔌 "),
                Span::styled(name.clone(), Style::default().fg(theme.fg)),
            ]))
        })
        .collect();

    let provider_list = List::new(provider_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if app.provider_tab_focus == 0 {
                    theme.active_border_style()
                } else {
                    theme.border_style()
                })
                .border_type(provider_border_type)
                .title(format!(" Providers ({}) ", app.get_provider_count())),
        )
        .highlight_style(if app.provider_tab_focus == 0 {
            theme.provider_highlight_style()
        } else {
            Style::default()
        })
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(provider_list, chunks[0], &mut app.provider_list_state);

    // 中间: Model 列表
    let model_border_type = if app.provider_tab_focus == 1 {
        theme.active_border_type()
    } else {
        BorderType::Plain
    };

    let model_title = if app.get_selected_provider().is_some() {
        format!(" Models ({}) ", app.models.len())
    } else {
        " Models ".to_string()
    };

    // 检查是否需要显示搜索框
    let show_search = app.search_active || !app.search_query.is_empty();

    let model_area = if show_search {
        let model_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(chunks[1]);

        // 渲染搜索框
        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if app.search_active {
                theme.active_border_style()
            } else {
                theme.border_style()
            })
            .border_type(if app.search_active {
                theme.active_border_type()
            } else {
                BorderType::Plain
            })
            .title(Span::styled(" / 搜索 ", theme.title_style()));

        let search_inner = search_block.inner(model_chunks[0]);
        frame.render_widget(search_block, model_chunks[0]);

        let search_text = if app.search_query.is_empty() {
            "输入关键词过滤模型..."
        } else {
            &app.search_query
        };

        let cursor = if app.search_active { "▌" } else { "" };
        let search_style = if app.search_query.is_empty() {
            theme.muted_style()
        } else {
            Style::default().fg(theme.fg)
        };

        let search_para = Paragraph::new(format!("{}{}", search_text, cursor)).style(search_style);
        frame.render_widget(search_para, search_inner);

        model_chunks[1]
    } else {
        chunks[1]
    };

    let model_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if app.provider_tab_focus == 1 {
            theme.active_border_style()
        } else {
            theme.border_style()
        })
        .border_type(model_border_type)
        .title(model_title);

    let model_inner = model_block.inner(model_area);
    frame.render_widget(model_block, model_area);

    if app.get_selected_provider().is_some() {
        let filtered_models = app.get_filtered_models();

        if !filtered_models.is_empty() {
            let model_items: Vec<ListItem> = filtered_models
                .iter()
                .map(|name| {
                    ListItem::new(Line::from(vec![
                        Span::raw(" 🤖 "),
                        Span::styled((*name).clone(), Style::default().fg(theme.info)),
                    ]))
                })
                .collect();

            let model_list = List::new(model_items)
                .highlight_style(if app.provider_tab_focus == 1 {
                    theme.model_highlight_style()
                } else {
                    Style::default()
                })
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(model_list, model_inner, &mut app.model_list_state);
        } else if !app.search_query.is_empty() {
            let text = Paragraph::new(format!("没有匹配 \"{}\" 的模型", app.search_query))
                .style(theme.muted_style())
                .wrap(Wrap { trim: true });
            frame.render_widget(text, model_inner);
        } else {
            let text = Paragraph::new("暂无 Model\n\n按 [a] 添加")
                .style(theme.muted_style())
                .wrap(Wrap { trim: true });
            frame.render_widget(text, model_inner);
        }
    } else {
        let text = Paragraph::new("← 选择 Provider")
            .style(theme.muted_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(text, model_inner);
    }

    // 右侧: 详情面板
    render_detail_panel(frame, app, theme, chunks[2]);
}

/// 渲染多选应用模式（两栏布局）
fn render_providers_multi_select_mode(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(multi_select_layout::LIST_PERCENT),
            Constraint::Percentage(multi_select_layout::DETAIL_PERCENT),
        ])
        .split(area);

    // 左侧: Provider 列表（带选择状态）
    let items: Vec<ListItem> = app
        .providers
        .iter()
        .map(|name| {
            let is_selected = app.is_provider_selected(name);
            let (prefix, prefix_style) = if is_selected {
                (
                    "☑",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("☐", Style::default().fg(theme.muted))
            };
            let name_style = if is_selected {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.fg)
            };
            ListItem::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(prefix, prefix_style),
                Span::raw(" "),
                Span::styled(name.clone(), name_style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.active_border_style())
                .border_type(theme.active_border_type())
                .title(format!(
                    " 多选 ({}/{}) ",
                    app.get_selected_count(),
                    app.get_provider_count()
                )),
        )
        .highlight_style(theme.highlight_style())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[0], &mut app.multi_apply_list_state);

    // 右侧: 操作说明
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 多选应用模式 ", theme.title_style()));

    let inner = detail_block.inner(chunks[1]);
    frame.render_widget(detail_block, chunks[1]);

    if let Some(provider_name) = app.get_multi_apply_current() {
        if let Ok(Some(provider)) = app.config_manager.opencode().get_provider(provider_name) {
            let is_selected = app.is_provider_selected(provider_name);
            let status = if is_selected {
                "✓ 已选择"
            } else {
                "○ 未选择"
            };

            let details = vec![
                Line::from(vec![
                    Span::styled("当前项: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(provider_name, Style::default().fg(theme.primary)),
                    Span::styled(format!("  [{}]", status), theme.success_style()),
                ]),
                Line::from(vec![
                    Span::styled("URL: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(&provider.options.base_url, theme.muted_style()),
                ]),
                Line::from(vec![
                    Span::styled("模型数: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(
                        provider.models.len().to_string(),
                        Style::default().fg(theme.success),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "快捷键:",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from("  [Space] 切换选择    [↑/k] 上移    [↓/j] 下移"),
                Line::from("  [Enter] 确认应用    [Esc] 取消    [A] 全选    [C] 清空"),
            ];

            let paragraph = Paragraph::new(details).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    } else {
        let empty = Paragraph::new("没有 Provider")
            .style(theme.muted_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, inner);
    }
}

/// 渲染详情面板（Provider + Model 详情）
fn render_detail_panel(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 详情 ", theme.title_style()));

    let inner = detail_block.inner(area);
    frame.render_widget(detail_block, area);

    if let Some(provider_name) = app.get_selected_provider() {
        if let Ok(Some(provider)) = app.config_manager.opencode().get_provider(provider_name) {
            let mut lines = vec![
                Line::from(Span::styled(
                    "Provider 信息",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(vec![
                    Span::styled("  名称: ", theme.muted_style()),
                    Span::styled(provider_name, Style::default().fg(theme.primary)),
                ]),
                Line::from(vec![
                    Span::styled("  URL:  ", theme.muted_style()),
                    Span::styled(&provider.options.base_url, Style::default().fg(theme.info)),
                ]),
                Line::from(vec![
                    Span::styled("  模型: ", theme.muted_style()),
                    Span::styled(
                        format!("{} 个", provider.models.len()),
                        Style::default().fg(theme.success),
                    ),
                ]),
            ];

            // 显示选中的 Model 详情
            if let Some(model_name) = app.get_selected_model() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "选中模型",
                    Style::default().add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(vec![
                    Span::styled("  名称: ", theme.muted_style()),
                    Span::styled(model_name, Style::default().fg(theme.info)),
                ]));

                // 显示模型限制信息（如果有）
                if let Some(model_info) = provider.models.get(model_name) {
                    if let Some(ref limit) = model_info.limit {
                        if let Some(ctx) = limit.context {
                            lines.push(Line::from(vec![
                                Span::styled("  Context: ", theme.muted_style()),
                                Span::styled(
                                    format_token_count(ctx),
                                    Style::default().fg(theme.fg),
                                ),
                            ]));
                        }
                        if let Some(out) = limit.output {
                            lines.push(Line::from(vec![
                                Span::styled("  Output:  ", theme.muted_style()),
                                Span::styled(
                                    format_token_count(out),
                                    Style::default().fg(theme.fg),
                                ),
                            ]));
                        }
                    }
                }
            }

            let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    } else {
        let text = Paragraph::new("选择一个 Provider 查看详情")
            .style(theme.muted_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(text, inner);
    }
}

/// 格式化 token 数量（如 128000 -> 128k）
fn format_token_count(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{}M", count / 1_000_000)
    } else if count >= 1000 {
        format!("{}k", count / 1000)
    } else {
        count.to_string()
    }
}

/// 渲染 MCP Tab（两栏布局：服务器列表 + 详情面板）
fn render_mcp_tab(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    // 多选同步模式使用特殊布局
    if app.is_mcp_multi_sync_mode {
        render_mcp_multi_sync_mode(frame, app, theme, area);
        return;
    }

    // 两栏布局：服务器列表 + 详情面板
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(mcp_tab_layout::SERVER_LIST_PERCENT),
            Constraint::Percentage(mcp_tab_layout::DETAIL_PANEL_PERCENT),
        ])
        .split(area);

    // 左侧: MCP 服务器列表
    let server_items: Vec<ListItem> = app
        .mcp_servers
        .iter()
        .map(|name| {
            // 获取服务器信息
            let (icon, enabled) =
                if let Ok(Some(server)) = app.config_manager.mcp().get_server(name) {
                    let icon = match server.server_type {
                        crate::config::models::McpServerType::Local => "📦",
                        crate::config::models::McpServerType::Remote => "🌐",
                    };
                    let enabled = server.enabled;
                    (icon, enabled)
                } else {
                    ("📦", true)
                };

            let status = if enabled { "✓" } else { "✗" };
            let status_style = if enabled {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.error)
            };

            ListItem::new(Line::from(vec![
                Span::raw(format!(" {} ", icon)),
                Span::styled(name.clone(), Style::default().fg(theme.fg)),
                Span::raw(" "),
                Span::styled(status, status_style),
            ]))
        })
        .collect();

    let server_list = List::new(server_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.active_border_style())
                .border_type(theme.active_border_type())
                .title(format!(" MCP 服务器 ({}) ", app.mcp_servers.len())),
        )
        .highlight_style(theme.highlight_style())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(server_list, chunks[0], &mut app.mcp_list_state);

    // 右侧: 详情面板
    render_mcp_detail_panel(frame, app, theme, chunks[1]);
}

/// 渲染 MCP 多选同步模式
fn render_mcp_multi_sync_mode(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(multi_select_layout::LIST_PERCENT),
            Constraint::Percentage(multi_select_layout::DETAIL_PERCENT),
        ])
        .split(area);

    // 左侧: 服务器列表（带选择状态）
    let items: Vec<ListItem> = app
        .mcp_servers
        .iter()
        .map(|name| {
            let is_selected = app.is_mcp_server_selected(name);
            let (prefix, prefix_style) = if is_selected {
                (
                    "☑",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                ("☐", Style::default().fg(theme.muted))
            };

            // 获取服务器类型图标
            let icon = if let Ok(Some(server)) = app.config_manager.mcp().get_server(name) {
                match server.server_type {
                    crate::config::models::McpServerType::Local => "📦",
                    crate::config::models::McpServerType::Remote => "🌐",
                }
            } else {
                "📦"
            };

            let name_style = if is_selected {
                Style::default().fg(theme.success)
            } else {
                Style::default().fg(theme.fg)
            };

            ListItem::new(Line::from(vec![
                Span::raw(" "),
                Span::styled(prefix, prefix_style),
                Span::raw(format!(" {} ", icon)),
                Span::styled(name.clone(), name_style),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.active_border_style())
                .border_type(theme.active_border_type())
                .title(format!(
                    " 多选同步 ({}/{}) ",
                    app.get_selected_mcp_count(),
                    app.get_mcp_server_count()
                )),
        )
        .highlight_style(theme.highlight_style())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[0], &mut app.mcp_multi_list_state);

    // 右侧: 操作说明
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 多选同步模式 ", theme.title_style()));

    let inner = detail_block.inner(chunks[1]);
    frame.render_widget(detail_block, chunks[1]);

    if let Some(server_name) = app.get_mcp_multi_current() {
        if let Ok(Some(server)) = app.config_manager.mcp().get_server(server_name) {
            let is_selected = app.is_mcp_server_selected(server_name);
            let status = if is_selected {
                "✓ 已选择"
            } else {
                "○ 未选择"
            };

            let type_str = server.type_display();

            let details = vec![
                Line::from(vec![
                    Span::styled("当前项: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(server_name, Style::default().fg(theme.primary)),
                    Span::styled(format!("  [{}]", status), theme.success_style()),
                ]),
                Line::from(vec![
                    Span::styled("类型: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(type_str, theme.muted_style()),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "快捷键:",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from("  [Space] 切换选择    [↑/k] 上移    [↓/j] 下移"),
                Line::from("  [Enter] 确认同步    [Esc] 取消    [A] 全选    [C] 清空"),
            ];

            let paragraph = Paragraph::new(details).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    } else {
        let empty = Paragraph::new("没有 MCP 服务器")
            .style(theme.muted_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, inner);
    }
}

/// 渲染 MCP 详情面板
fn render_mcp_detail_panel(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 详情 ", theme.title_style()));

    let inner = detail_block.inner(area);
    frame.render_widget(detail_block, area);

    if let Some(server_name) = app.get_selected_mcp_server() {
        if let Ok(Some(server)) = app.config_manager.mcp().get_server(server_name) {
            let type_str = format!(
                "{} {}",
                server.type_display(),
                match server.server_type {
                    crate::config::models::McpServerType::Local => "📦",
                    crate::config::models::McpServerType::Remote => "🌐",
                }
            );

            let status_str = if server.enabled {
                "✓ 已启用"
            } else {
                "✗ 已禁用"
            };
            let status_style = if server.enabled {
                theme.success_style()
            } else {
                theme.error_style()
            };

            let mut lines = vec![
                Line::from(Span::styled(
                    "服务器信息",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
                Line::from(vec![
                    Span::styled("  名称: ", theme.muted_style()),
                    Span::styled(server_name, Style::default().fg(theme.primary)),
                ]),
                Line::from(vec![
                    Span::styled("  类型: ", theme.muted_style()),
                    Span::styled(type_str, Style::default().fg(theme.info)),
                ]),
                Line::from(vec![
                    Span::styled("  状态: ", theme.muted_style()),
                    Span::styled(status_str, status_style),
                ]),
            ];

            // 根据类型显示不同信息
            match server.server_type {
                crate::config::models::McpServerType::Local => {
                    if let Some(ref cmd) = server.command {
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "命令:",
                            Style::default().add_modifier(Modifier::BOLD),
                        )));
                        lines.push(Line::from(vec![
                            Span::styled("  ", theme.muted_style()),
                            Span::styled(cmd.join(" "), Style::default().fg(theme.fg)),
                        ]));
                    }

                    if !server.environment.is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "环境变量:",
                            Style::default().add_modifier(Modifier::BOLD),
                        )));
                        for (key, value) in &server.environment {
                            lines.push(Line::from(vec![
                                Span::styled(format!("  {}: ", key), theme.muted_style()),
                                Span::styled(value, Style::default().fg(theme.fg)),
                            ]));
                        }
                    }
                }
                crate::config::models::McpServerType::Remote => {
                    if let Some(ref url) = server.url {
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("URL: ", Style::default().add_modifier(Modifier::BOLD)),
                            Span::styled(url, Style::default().fg(theme.info)),
                        ]));
                    }

                    if !server.headers.is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "Headers:",
                            Style::default().add_modifier(Modifier::BOLD),
                        )));
                        for (key, _) in &server.headers {
                            lines.push(Line::from(vec![
                                Span::styled(format!("  {}: ", key), theme.muted_style()),
                                Span::styled("********", Style::default().fg(theme.fg)),
                            ]));
                        }
                    }

                    if let Some(ref oauth) = server.oauth {
                        lines.push(Line::from(""));
                        lines.push(Line::from(Span::styled(
                            "OAuth 配置:",
                            Style::default().add_modifier(Modifier::BOLD),
                        )));
                        if oauth.client_id.is_some() {
                            lines.push(Line::from(vec![
                                Span::styled("  Client ID: ", theme.muted_style()),
                                Span::styled("已配置", Style::default().fg(theme.success)),
                            ]));
                        }
                        if oauth.client_secret.is_some() {
                            lines.push(Line::from(vec![
                                Span::styled("  Client Secret: ", theme.muted_style()),
                                Span::styled("已配置", Style::default().fg(theme.success)),
                            ]));
                        }
                        if let Some(ref scope) = oauth.scope {
                            lines.push(Line::from(vec![
                                Span::styled("  Scope: ", theme.muted_style()),
                                Span::styled(scope, Style::default().fg(theme.fg)),
                            ]));
                        }
                    }
                }
            }

            // 超时配置
            if let Some(timeout) = server.timeout {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("超时: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{}ms", timeout), Style::default().fg(theme.fg)),
                ]));
            }

            let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    } else {
        let text = Paragraph::new("选择一个 MCP 服务器查看详情")
            .style(theme.muted_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(text, inner);
    }
}

/// 渲染 Backup Tab
fn render_backup_tab(frame: &mut Frame, _app: &mut App, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(8)])
        .split(area);

    // 主内容区域
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 💾 备份与恢复 ", theme.title_style()));

    let inner = block.inner(chunks[0]);
    frame.render_widget(block, chunks[0]);

    let backup_info = vec![
        Line::from(Span::styled(
            "配置备份功能",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("备份功能支持将您的 Coding Agent 配置备份到 WebDAV 服务器。"),
        Line::from(""),
        Line::from(Span::styled(
            "支持的备份类型:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  • OpenCode 配置"),
        Line::from(""),
        Line::from(Span::styled(
            "使用说明:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  由于备份涉及网络操作，请使用命令行模式执行备份/恢复:"),
        Line::from(""),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("opcd backup", Style::default().fg(theme.info)),
            Span::raw("        # 创建备份"),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("opcd restore", Style::default().fg(theme.info)),
            Span::raw("       # 恢复备份"),
        ]),
    ];

    let paragraph = Paragraph::new(backup_info).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);

    // 底部提示区域
    let hint_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 💡 提示 ", theme.title_style()));

    let hint_inner = hint_block.inner(chunks[1]);
    frame.render_widget(hint_block, chunks[1]);

    let hints = vec![
        Line::from("备份功能需要配置 WebDAV 服务器。"),
        Line::from("您可以使用坚果云、NextCloud 等支持 WebDAV 的服务。"),
        Line::from(""),
        Line::from(vec![
            Span::raw("配置 WebDAV: "),
            Span::styled("opcd webdav config", Style::default().fg(theme.primary)),
        ]),
    ];

    let hint_paragraph = Paragraph::new(hints)
        .style(theme.muted_style())
        .wrap(Wrap { trim: true });
    frame.render_widget(hint_paragraph, hint_inner);
}

/// 渲染 Status Tab
fn render_status_tab(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // 左侧: 配置概览
    let status_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 📊 配置概览 ", theme.title_style()));

    let status_inner = status_block.inner(chunks[0]);
    frame.render_widget(status_block, chunks[0]);

    // 计算 Model 总数
    let total_models: usize = app
        .providers
        .iter()
        .filter_map(|p| app.config_manager.opencode().get_models(p).ok())
        .map(|m| m.len())
        .sum();

    let status_lines = vec![
        Line::from(Span::styled(
            "当前状态",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Provider 数量: ", theme.muted_style()),
            Span::styled(
                app.providers.len().to_string(),
                Style::default().fg(theme.success),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Model 总数:    ", theme.muted_style()),
            Span::styled(total_models.to_string(), Style::default().fg(theme.info)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "已配置的 Provider:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ];

    let mut all_status_lines = status_lines;
    for provider in app.providers.iter().take(8) {
        let model_count = app
            .config_manager
            .opencode()
            .get_models(provider)
            .map(|m| m.len())
            .unwrap_or(0);
        all_status_lines.push(Line::from(vec![
            Span::raw("  • "),
            Span::styled(provider, Style::default().fg(theme.fg)),
            Span::styled(format!(" ({} models)", model_count), theme.muted_style()),
        ]));
    }
    if app.providers.len() > 8 {
        all_status_lines.push(Line::from(Span::styled(
            format!("  ... 还有 {} 个", app.providers.len() - 8),
            theme.muted_style(),
        )));
    }

    let status_paragraph = Paragraph::new(all_status_lines).wrap(Wrap { trim: true });
    frame.render_widget(status_paragraph, status_inner);

    // 右侧: 操作日志
    let log_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(
            format!(" 📝 操作日志 ({}) ", app.operation_logs.len()),
            theme.title_style(),
        ));

    let log_inner = log_block.inner(chunks[1]);
    frame.render_widget(log_block, chunks[1]);

    if !app.operation_logs.is_empty() {
        let mut log_lines: Vec<Line> = Vec::new();
        for log in app.operation_logs.iter().rev().take(15) {
            let style = match log.level {
                MessageType::Success => theme.success_style(),
                MessageType::Error => theme.error_style(),
                MessageType::Warning => theme.warning_style(),
                MessageType::Info => theme.info_style(),
            };
            let icon = match log.level {
                MessageType::Success => "✓",
                MessageType::Error => "✗",
                MessageType::Warning => "⚠",
                MessageType::Info => "ℹ",
            };
            log_lines.push(Line::from(vec![
                Span::styled(format!("[{}] ", log.formatted_time()), theme.muted_style()),
                Span::styled(format!("{} ", icon), style),
                Span::styled(&log.message, style),
            ]));
        }

        let log_paragraph = Paragraph::new(log_lines).wrap(Wrap { trim: true });
        frame.render_widget(log_paragraph, log_inner);
    } else {
        let empty = Paragraph::new("暂无操作日志\n\n执行操作后日志将在此显示")
            .style(theme.muted_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, log_inner);
    }
}

/// 渲染底部状态栏
fn render_footer(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let shortcuts = match app.current_tab {
        AppTab::Providers => {
            if app.is_multi_apply_mode {
                "[j/↓]下移 [k/↑]上移 [Space]选择 [Enter]确认 [A]全选 [C]清空 [Esc]取消"
            } else if app.provider_tab_focus == 0 {
                // Provider 列表焦点
                "[h/l]切换面板 [j/k]导航 [Enter]应用 [a]添加 [e]编辑 [d]删除"
            } else {
                // Model 列表焦点
                "[h/l]切换面板 [j/k]导航 [a]添加 [d]删除 [/]搜索 [t]获取模型"
            }
        }
        AppTab::Mcp => {
            if app.is_mcp_multi_sync_mode {
                "[j/↓]下移 [k/↑]上移 [Space]选择 [Enter]确认 [A]全选 [C]清空 [Esc]取消"
            } else {
                "[j/k]导航 [a]添加 [e]编辑 [d]删除 [Space]启用/禁用 [Enter]同步"
            }
        }
        AppTab::Backup => "[b]备份 [r]恢复 [d]删除",
        AppTab::Status => "[r]刷新",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(" 快捷键 ", theme.title_style()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // 解析并高亮快捷键
    let shortcut_spans = parse_shortcuts_with_highlight(shortcuts, theme);
    let global_spans = vec![
        Span::styled(
            "[Tab]切换",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", theme.muted_style()),
        Span::styled(
            "[?]帮助",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", theme.muted_style()),
        Span::styled(
            "[q]退出",
            Style::default()
                .fg(theme.error)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    let text = Paragraph::new(vec![Line::from(shortcut_spans), Line::from(global_spans)]);
    frame.render_widget(text, inner);
}

/// 解析快捷键字符串，将 [xxx]说明 格式整体高亮显示
fn parse_shortcuts_with_highlight<'a>(text: &str, theme: &Theme) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = text.chars().collect();

    while current_pos < chars.len() {
        if chars[current_pos] == '[' {
            // 找到结束括号
            let mut end_pos = current_pos + 1;
            while end_pos < chars.len() && chars[end_pos] != ']' {
                end_pos += 1;
            }

            if end_pos < chars.len() {
                // 找到完整的 [xxx] 格式，继续收集后面的说明文字
                let mut desc_end = end_pos + 1;
                while desc_end < chars.len() && chars[desc_end] != ' ' && chars[desc_end] != '[' {
                    desc_end += 1;
                }

                // [xxx]说明 整体高亮
                let shortcut_with_desc: String = chars[current_pos..desc_end].iter().collect();
                spans.push(Span::styled(
                    shortcut_with_desc,
                    Style::default()
                        .fg(theme.primary)
                        .add_modifier(Modifier::BOLD),
                ));
                current_pos = desc_end;
            } else {
                // 没找到结束括号，作为普通文本处理
                spans.push(Span::styled(
                    chars[current_pos].to_string(),
                    theme.muted_style(),
                ));
                current_pos += 1;
            }
        } else if chars[current_pos] == ' ' {
            // 空格作为分隔符，使用柔和样式
            spans.push(Span::styled(" ", theme.muted_style()));
            current_pos += 1;
        } else {
            // 收集其他普通文本直到遇到 [ 或空格
            let mut end_pos = current_pos;
            while end_pos < chars.len() && chars[end_pos] != '[' && chars[end_pos] != ' ' {
                end_pos += 1;
            }
            let text_part: String = chars[current_pos..end_pos].iter().collect();
            spans.push(Span::styled(text_part, theme.muted_style()));
            current_pos = end_pos;
        }
    }

    spans
}

/// 渲染帮助弹窗
fn render_help_popup(frame: &mut Frame, theme: &Theme, area: Rect) {
    let popup_area = centered_rect(
        help_popup_layout::WIDTH_PERCENT,
        help_popup_layout::HEIGHT_PERCENT,
        area,
    );

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.active_border_style())
        .title(Span::styled(
            " ❓ 帮助 - 按任意键关闭 ",
            theme.title_style(),
        ));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let help_text = vec![
        Line::from(Span::styled(
            "全局快捷键:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  q / Ctrl+C    退出应用"),
        Line::from("  Tab           切换下一个 Tab"),
        Line::from("  Shift+Tab     切换上一个 Tab"),
        Line::from("  ?             显示/隐藏帮助"),
        Line::from(""),
        Line::from(Span::styled(
            "Provider Tab (三栏布局):",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  h / l         切换面板 (Provider ↔ Model)"),
        Line::from("  j / ↓         选择下一个"),
        Line::from("  k / ↑         选择上一个"),
        Line::from("  Enter         应用配置（进入多选模式）"),
        Line::from("  a             添加 Provider / Model"),
        Line::from("  e             编辑选中的 Provider"),
        Line::from("  d             删除选中的 Provider / Model"),
        Line::from("  /             搜索模型"),
        Line::from("  t             获取站点模型"),
        Line::from(""),
        Line::from(Span::styled(
            "多选应用模式:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  Space         切换选择状态"),
        Line::from("  Enter         确认并选择应用范围"),
        Line::from("  A             全选所有 Provider"),
        Line::from("  C             清空选择"),
        Line::from("  Esc           取消多选模式"),
        Line::from(""),
        Line::from(Span::styled(
            "Backup Tab:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  b             创建备份"),
        Line::from("  r             恢复备份"),
    ];

    let paragraph = Paragraph::new(help_text).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

/// 渲染 Toast 消息
fn render_toast(
    frame: &mut Frame,
    msg: &crate::tui::types::StatusMessage,
    theme: &Theme,
    area: Rect,
) {
    let popup_area = Rect {
        x: area.width.saturating_sub(42),
        y: 1,
        width: 40.min(area.width),
        height: 3,
    };

    frame.render_widget(Clear, popup_area);

    let (icon, style) = match msg.msg_type {
        MessageType::Success => ("✓", theme.success_style()),
        MessageType::Error => ("✗", theme.error_style()),
        MessageType::Warning => ("⚠", theme.warning_style()),
        MessageType::Info => ("ℹ", theme.info_style()),
    };

    let block = Block::default().borders(Borders::ALL).border_style(style);

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let text = Paragraph::new(format!("{} {}", icon, msg.content)).style(style);
    frame.render_widget(text, inner);
}

/// 创建居中矩形
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
