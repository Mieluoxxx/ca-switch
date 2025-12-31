// 主布局渲染

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

use crate::tui::{
    app::App,
    theme::Theme,
    types::{AppTab, MessageType},
};

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

    // Model 删除对话框
    app.model_delete_dialog.render(frame, theme, full_area);

    // 模型多选对话框
    app.model_select_dialog.render(frame, theme, full_area);

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
    let title = format!(" 🚀 ca-switch v{} ", version);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(Span::styled(title, theme.title_style()));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = Paragraph::new("Coding Agent 配置管理工具")
        .style(Style::default().fg(theme.muted));
    frame.render_widget(text, inner);
}

/// 渲染 Tab 栏
fn render_tabs(frame: &mut Frame, app: &App, theme: &Theme, area: Rect) {
    let titles: Vec<Line> = AppTab::all()
        .iter()
        .map(|tab| {
            let icon = match tab {
                AppTab::Providers => "🔌",
                AppTab::Models => "🤖",
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
                .title(" 功能模块 "),
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
        AppTab::Models => render_models_tab(frame, app, theme, area),
        AppTab::Backup => render_backup_tab(frame, app, theme, area),
        AppTab::Status => render_status_tab(frame, app, theme, area),
    }
}

/// 渲染 Provider Tab
fn render_providers_tab(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // 左侧: Provider 列表
    let items: Vec<ListItem> = app
        .providers
        .iter()
        .map(|name| {
            ListItem::new(Line::from(vec![
                Span::raw(" 🔌 "),
                Span::styled(name.clone(), Style::default().fg(theme.fg)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.active_border_style())
                .title(format!(" Providers ({}) ", app.providers.len())),
        )
        .highlight_style(theme.highlight_style())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(list, chunks[0], &mut app.provider_list_state);

    // 右侧: Provider 详情
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(" Provider 详情 ");

    let inner = detail_block.inner(chunks[1]);
    frame.render_widget(detail_block, chunks[1]);

    if let Some(provider_name) = app.get_selected_provider() {
        if let Ok(Some(provider)) = app.config_manager.opencode().get_provider(provider_name) {
            let details = vec![
                Line::from(vec![
                    Span::styled("名称: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(provider_name),
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
                    "可用模型:",
                    Style::default().add_modifier(Modifier::BOLD),
                )),
            ];

            let mut all_lines = details;
            // 排序模型名称以保持稳定的显示顺序
            let mut model_names: Vec<&String> = provider.models.keys().collect();
            model_names.sort();
            for model_name in model_names {
                all_lines.push(Line::from(vec![
                    Span::raw("  • "),
                    Span::styled(model_name, Style::default().fg(theme.info)),
                ]));
            }

            let paragraph = Paragraph::new(all_lines).wrap(Wrap { trim: true });
            frame.render_widget(paragraph, inner);
        }
    } else {
        let empty = Paragraph::new("选择一个 Provider 查看详情")
            .style(theme.muted_style())
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, inner);
    }
}

/// 渲染 Model Tab
fn render_models_tab(frame: &mut Frame, app: &mut App, theme: &Theme, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(area);

    // 左侧: Provider 选择列表
    let provider_items: Vec<ListItem> = app
        .providers
        .iter()
        .map(|name| {
            let is_selected = app.model_tab_provider.as_ref() == Some(name);
            let prefix = if is_selected { "✓ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(name.clone(), Style::default().fg(theme.fg)),
            ]))
        })
        .collect();

    let provider_border = if app.model_tab_focus == 0 {
        theme.active_border_style()
    } else {
        theme.border_style()
    };

    let provider_list = List::new(provider_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(provider_border)
                .title(format!(" Provider ({}) ", app.providers.len())),
        )
        .highlight_style(theme.highlight_style())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(provider_list, chunks[0], &mut app.model_provider_list_state);

    // 右侧: Model 列表
    let model_border = if app.model_tab_focus == 1 {
        theme.active_border_style()
    } else {
        theme.border_style()
    };

    let model_title = if let Some(provider) = &app.model_tab_provider {
        format!(" {} 的 Models ({}) ", provider, app.models.len())
    } else {
        " Models (请先选择 Provider) ".to_string()
    };

    // 显示搜索框时，需要分割右侧区域
    let show_search = app.search_active || !app.search_query.is_empty();

    let model_area = if show_search {
        // 右侧分割：搜索框 + 模型列表
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(5)])
            .split(chunks[1]);

        // 渲染搜索框
        let search_border = if app.search_active {
            theme.active_border_style()
        } else {
            theme.border_style()
        };

        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(search_border)
            .title(" / 搜索 ");

        let search_inner = search_block.inner(right_chunks[0]);
        frame.render_widget(search_block, right_chunks[0]);

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

        right_chunks[1]
    } else {
        // 不显示搜索框时，整个区域给模型列表
        chunks[1]
    };

    let model_block = Block::default()
        .borders(Borders::ALL)
        .border_style(model_border)
        .title(model_title);

    let inner = model_block.inner(model_area);
    frame.render_widget(model_block, model_area);

    if app.model_tab_provider.is_some() {
        // 获取过滤后的模型列表
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
                .highlight_style(theme.highlight_style())
                .highlight_symbol("▶ ");

            frame.render_stateful_widget(model_list, inner, &mut app.model_list_state);
        } else if !app.search_query.is_empty() {
            let text = Paragraph::new(format!("没有匹配 \"{}\" 的模型\n\n按 [Esc] 清除搜索", app.search_query))
                .style(theme.muted_style())
                .wrap(Wrap { trim: true });
            frame.render_widget(text, inner);
        } else {
            let text = Paragraph::new("暂无 Model\n\n按 [a] 添加 [t] 获取站点模型")
                .style(theme.muted_style())
                .wrap(Wrap { trim: true });
            frame.render_widget(text, inner);
        }
    } else {
        let text = Paragraph::new("选择左侧的 Provider 来查看其 Model 列表\n\n按 [Enter] 选择 Provider")
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
        .title(" 💾 备份与恢复 ");

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
            Span::styled("ca-switch backup", Style::default().fg(theme.info)),
            Span::raw("        # 创建备份"),
        ]),
        Line::from(vec![
            Span::raw("    "),
            Span::styled("ca-switch restore", Style::default().fg(theme.info)),
            Span::raw("       # 恢复备份"),
        ]),
    ];

    let paragraph = Paragraph::new(backup_info).wrap(Wrap { trim: true });
    frame.render_widget(paragraph, inner);

    // 底部提示区域
    let hint_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(" 💡 提示 ");

    let hint_inner = hint_block.inner(chunks[1]);
    frame.render_widget(hint_block, chunks[1]);

    let hints = vec![
        Line::from("备份功能需要配置 WebDAV 服务器。"),
        Line::from("您可以使用坚果云、NextCloud 等支持 WebDAV 的服务。"),
        Line::from(""),
        Line::from(vec![
            Span::raw("配置 WebDAV: "),
            Span::styled("ca-switch webdav config", Style::default().fg(theme.primary)),
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
        .title(" 📊 配置概览 ");

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
            Span::styled(
                total_models.to_string(),
                Style::default().fg(theme.info),
            ),
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
        .title(format!(" 📝 操作日志 ({}) ", app.operation_logs.len()));

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
            "[j/↓]下移 [k/↑]上移 [Enter]应用 [a]添加 [e]编辑 [d]删除 [t]检测"
        }
        AppTab::Models => "[h/l]切换面板 [j/k]导航 [Enter]选择 [a]添加 [d]删除 [/]搜索 [t]获取模型",
        AppTab::Backup => "[b]备份 [r]恢复 [d]删除",
        AppTab::Status => "[r]刷新",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(" 快捷键 ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // 解析并高亮快捷键
    let shortcut_spans = parse_shortcuts_with_highlight(shortcuts, theme);
    let global_spans = vec![
        Span::styled("[Tab]切换", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" ", theme.muted_style()),
        Span::styled("[?]帮助", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" ", theme.muted_style()),
        Span::styled("[q]退出", Style::default().fg(theme.error).add_modifier(Modifier::BOLD)),
    ];

    let text = Paragraph::new(vec![
        Line::from(shortcut_spans),
        Line::from(global_spans),
    ]);
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
                    Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
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
    let popup_area = centered_rect(60, 70, area);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.active_border_style())
        .title(" ❓ 帮助 - 按任意键关闭 ");

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
            "Provider Tab:",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from("  j / ↓         选择下一个"),
        Line::from("  k / ↑         选择上一个"),
        Line::from("  Enter         应用选中的 Provider"),
        Line::from("  a             添加新 Provider"),
        Line::from("  e             编辑选中的 Provider"),
        Line::from("  d             删除选中的 Provider"),
        Line::from("  t             检测站点连接"),
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

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style);

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
