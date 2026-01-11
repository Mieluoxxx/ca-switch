// MCP 服务器 JSON 编辑器组件

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::config::models::{McpServer, McpServerType};
use crate::tui::theme::Theme;
use std::collections::HashMap;

// ============================================================================
// 布局常量
// ============================================================================

/// MCP 表单弹窗宽度百分比
const MCP_FORM_WIDTH_PERCENT: u16 = 80;
/// MCP 表单弹窗高度（行数）
const MCP_FORM_HEIGHT: u16 = 22;

/// MCP 服务器 JSON 编辑器
#[derive(Debug, Clone)]
pub struct McpServerForm {
    pub visible: bool,
    pub is_edit_mode: bool,
    pub edit_server_name: Option<String>,

    // JSON 内容编辑（包含名称作为 key）
    pub json_content: Vec<String>, // 每行一个字符串
    pub json_cursor_row: usize,
    pub json_cursor_col: usize,

    // 错误信息
    pub error_message: Option<String>,

    // 内容修改追踪
    pub has_changes: bool,         // 是否有未保存的修改
    pub confirm_close: bool,       // 是否显示关闭确认
    original_content: Vec<String>, // 原始内容（用于比较）
}

impl Default for McpServerForm {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl McpServerForm {
    pub fn new() -> Self {
        let default_content = Self::default_json_template();
        Self {
            visible: false,
            is_edit_mode: false,
            edit_server_name: None,
            json_content: default_content.clone(),
            json_cursor_row: 1,
            json_cursor_col: 2,
            error_message: None,
            has_changes: false,
            confirm_close: false,
            original_content: default_content,
        }
    }

    /// 默认 JSON 模板（本地服务器）
    fn default_json_template() -> Vec<String> {
        vec![
            "{".to_string(),
            "  \"my-server\": {".to_string(),
            "    \"command\": \"npx\",".to_string(),
            "    \"args\": [\"-y\", \"@anthropic/mcp-server-xxx\"],".to_string(),
            "    \"env\": {}".to_string(),
            "  }".to_string(),
            "}".to_string(),
        ]
    }

    /// 远程服务器 JSON 模板
    fn remote_json_template() -> Vec<String> {
        vec![
            "{".to_string(),
            "  \"my-server\": {".to_string(),
            "    \"url\": \"https://mcp-server.example.com\",".to_string(),
            "    \"headers\": {},".to_string(),
            "    \"oauth\": {".to_string(),
            "      \"clientId\": \"\",".to_string(),
            "      \"clientSecret\": \"\",".to_string(),
            "      \"scope\": \"\"".to_string(),
            "    }".to_string(),
            "  }".to_string(),
            "}".to_string(),
        ]
    }

    /// 空 JSON 模板
    fn empty_json_template() -> Vec<String> {
        vec![
            "{".to_string(),
            "  \"my-server\": {".to_string(),
            "  }".to_string(),
            "}".to_string(),
        ]
    }

    /// 显示新建表单
    pub fn show_create(&mut self) {
        self.clear();
        self.visible = true;
        self.is_edit_mode = false;
        self.edit_server_name = None;
        self.original_content = self.json_content.clone();
        self.has_changes = false;
        self.confirm_close = false;
    }

    /// 显示编辑表单（从原始 JSON 内容）
    pub fn show_edit_raw(&mut self, name: &str, raw_json: &str) {
        self.clear();
        self.visible = true;
        self.is_edit_mode = true;
        self.edit_server_name = Some(name.to_string());

        // 构建带名称 key 的 JSON
        let wrapped_json = format!("{{\n  \"{}\": {}\n}}", name, raw_json.trim());
        self.json_content = wrapped_json.lines().map(|s| s.to_string()).collect();
        self.json_cursor_row = 1;
        self.json_cursor_col = 2;
        self.original_content = self.json_content.clone();
        self.has_changes = false;
        self.confirm_close = false;
    }

    /// 显示编辑表单（从 McpServer 结构体，兼容旧代码）
    pub fn show_edit(&mut self, name: &str, server: &McpServer) {
        self.clear();
        self.visible = true;
        self.is_edit_mode = true;
        self.edit_server_name = Some(name.to_string());

        // 将服务器配置转换为带名称 key 的 JSON
        self.json_content = self.server_to_json_lines(name, server);
        self.json_cursor_row = 1;
        self.json_cursor_col = 2;
        self.original_content = self.json_content.clone();
        self.has_changes = false;
        self.confirm_close = false;
    }

    /// 将 McpServer 转换为 JSON 行（包含名称作为 key）
    fn server_to_json_lines(&self, name: &str, server: &McpServer) -> Vec<String> {
        let mut lines = vec!["{".to_string(), format!("  \"{}\": {{", name)];

        match server.server_type {
            McpServerType::Local => {
                // command 和 args
                if let Some(ref cmd) = server.command {
                    if !cmd.is_empty() {
                        lines.push(format!("    \"command\": \"{}\",", cmd[0]));
                        if cmd.len() > 1 {
                            let args: Vec<String> =
                                cmd[1..].iter().map(|a| format!("\"{}\"", a)).collect();
                            lines.push(format!("    \"args\": [{}],", args.join(", ")));
                        } else {
                            lines.push("    \"args\": [],".to_string());
                        }
                    }
                }

                // env
                if server.environment.is_empty() {
                    lines.push("    \"env\": {}".to_string());
                } else {
                    lines.push("    \"env\": {".to_string());
                    let env_items: Vec<(&String, &String)> = server.environment.iter().collect();
                    for (i, (k, v)) in env_items.iter().enumerate() {
                        let comma = if i < env_items.len() - 1 { "," } else { "" };
                        lines.push(format!("      \"{}\": \"{}\"{}", k, v, comma));
                    }
                    lines.push("    }".to_string());
                }
            }
            McpServerType::Remote => {
                // url
                if let Some(ref url) = server.url {
                    lines.push(format!("    \"url\": \"{}\",", url));
                }

                // headers
                if server.headers.is_empty() {
                    lines.push("    \"headers\": {},".to_string());
                } else {
                    lines.push("    \"headers\": {".to_string());
                    let header_items: Vec<(&String, &String)> = server.headers.iter().collect();
                    for (i, (k, v)) in header_items.iter().enumerate() {
                        let comma = if i < header_items.len() - 1 { "," } else { "" };
                        lines.push(format!("      \"{}\": \"{}\"{}", k, v, comma));
                    }
                    lines.push("    },".to_string());
                }

                // oauth
                if let Some(ref oauth) = server.oauth {
                    lines.push("    \"oauth\": {".to_string());
                    let mut oauth_items = Vec::new();
                    if let Some(ref cid) = oauth.client_id {
                        oauth_items.push(format!("      \"clientId\": \"{}\"", cid));
                    }
                    if let Some(ref cs) = oauth.client_secret {
                        oauth_items.push(format!("      \"clientSecret\": \"{}\"", cs));
                    }
                    if let Some(ref scope) = oauth.scope {
                        oauth_items.push(format!("      \"scope\": \"{}\"", scope));
                    }
                    for (i, item) in oauth_items.iter().enumerate() {
                        let comma = if i < oauth_items.len() - 1 { "," } else { "" };
                        lines.push(format!("{}{}", item, comma));
                    }
                    lines.push("    }".to_string());
                }
            }
        }

        lines.push("  }".to_string());
        lines.push("}".to_string());
        lines
    }

    /// 隐藏表单
    pub fn hide(&mut self) {
        self.visible = false;
        self.error_message = None;
    }

    /// 清空表单（恢复默认模板）
    pub fn clear(&mut self) {
        self.json_content = Self::default_json_template();
        self.json_cursor_row = 1;
        self.json_cursor_col = 2;
        self.error_message = None;
    }

    /// 完全清空内容
    pub fn clear_to_empty(&mut self) {
        self.json_content = vec![String::new()];
        self.json_cursor_row = 0;
        self.json_cursor_col = 0;
        self.error_message = None;
    }

    /// 标记内容已修改
    fn mark_changed(&mut self) {
        self.has_changes = self.json_content != self.original_content;
        self.error_message = None;
    }

    /// 检查是否有未保存的修改
    pub fn has_unsaved_changes(&self) -> bool {
        self.has_changes || self.json_content != self.original_content
    }

    /// 请求关闭（如果有未保存修改则显示确认）
    pub fn request_close(&mut self) -> bool {
        if self.has_unsaved_changes() {
            self.confirm_close = true;
            false // 不立即关闭，显示确认
        } else {
            true // 可以直接关闭
        }
    }

    /// 确认关闭（放弃修改）
    pub fn confirm_discard(&mut self) {
        self.confirm_close = false;
        self.has_changes = false;
    }

    /// 取消关闭确认
    pub fn cancel_close(&mut self) {
        self.confirm_close = false;
    }

    /// 处理字符输入
    pub fn handle_input(&mut self, c: char) {
        if self.json_cursor_row < self.json_content.len() {
            let line = &mut self.json_content[self.json_cursor_row];
            if self.json_cursor_col <= line.len() {
                line.insert(self.json_cursor_col, c);
                self.json_cursor_col += 1;
            }
        }
        self.mark_changed();
    }

    /// 处理退格
    pub fn handle_backspace(&mut self) {
        if self.json_cursor_col > 0 {
            let line = &mut self.json_content[self.json_cursor_row];
            self.json_cursor_col -= 1;
            if self.json_cursor_col < line.len() {
                line.remove(self.json_cursor_col);
            }
        } else if self.json_cursor_row > 0 {
            // 合并到上一行
            let current_line = self.json_content.remove(self.json_cursor_row);
            self.json_cursor_row -= 1;
            self.json_cursor_col = self.json_content[self.json_cursor_row].len();
            self.json_content[self.json_cursor_row].push_str(&current_line);
        }
        self.mark_changed();
    }

    /// 处理删除键
    pub fn handle_delete(&mut self) {
        let line = &mut self.json_content[self.json_cursor_row];
        if self.json_cursor_col < line.len() {
            line.remove(self.json_cursor_col);
        } else if self.json_cursor_row + 1 < self.json_content.len() {
            // 合并下一行
            let next_line = self.json_content.remove(self.json_cursor_row + 1);
            self.json_content[self.json_cursor_row].push_str(&next_line);
        }
        self.mark_changed();
    }

    /// 处理回车键换行
    pub fn handle_enter(&mut self) {
        let current_line = &self.json_content[self.json_cursor_row];
        let remaining = current_line[self.json_cursor_col..].to_string();
        self.json_content[self.json_cursor_row] = current_line[..self.json_cursor_col].to_string();
        self.json_cursor_row += 1;
        self.json_content.insert(self.json_cursor_row, remaining);
        self.json_cursor_col = 0;
        self.mark_changed();
    }

    /// 光标上移
    pub fn cursor_up(&mut self) {
        if self.json_cursor_row > 0 {
            self.json_cursor_row -= 1;
            let line_len = self.json_content[self.json_cursor_row].len();
            if self.json_cursor_col > line_len {
                self.json_cursor_col = line_len;
            }
        }
    }

    /// 光标下移
    pub fn cursor_down(&mut self) {
        if self.json_cursor_row + 1 < self.json_content.len() {
            self.json_cursor_row += 1;
            let line_len = self.json_content[self.json_cursor_row].len();
            if self.json_cursor_col > line_len {
                self.json_cursor_col = line_len;
            }
        }
    }

    /// 光标左移
    pub fn cursor_left(&mut self) {
        if self.json_cursor_col > 0 {
            self.json_cursor_col -= 1;
        } else if self.json_cursor_row > 0 {
            self.json_cursor_row -= 1;
            self.json_cursor_col = self.json_content[self.json_cursor_row].len();
        }
    }

    /// 光标右移
    pub fn cursor_right(&mut self) {
        let line_len = self.json_content[self.json_cursor_row].len();
        if self.json_cursor_col < line_len {
            self.json_cursor_col += 1;
        } else if self.json_cursor_row + 1 < self.json_content.len() {
            self.json_cursor_row += 1;
            self.json_cursor_col = 0;
        }
    }

    /// 切换模板类型（本地 → 远程 → 空 → 本地）
    pub fn toggle_mode(&mut self) {
        let has_command = self.json_content.iter().any(|l| l.contains("\"command\""));
        let has_url = self.json_content.iter().any(|l| l.contains("\"url\""));

        if has_command {
            // 本地 → 远程
            self.json_content = Self::remote_json_template();
        } else if has_url {
            // 远程 → 空
            self.json_content = Self::empty_json_template();
        } else {
            // 空 → 本地
            self.json_content = Self::default_json_template();
        }
        self.json_cursor_row = 1;
        self.json_cursor_col = 2;
    }

    /// 获取当前模板类型名称
    pub fn get_template_name(&self) -> &'static str {
        let has_command = self.json_content.iter().any(|l| l.contains("\"command\""));
        let has_url = self.json_content.iter().any(|l| l.contains("\"url\""));

        if has_command {
            "本地服务器"
        } else if has_url {
            "远程服务器"
        } else {
            "空模板"
        }
    }

    /// 验证表单
    pub fn is_valid(&self) -> bool {
        self.parse_json().is_ok()
    }

    /// 解析 JSON 内容
    fn parse_json(&self) -> Result<serde_json::Value, String> {
        let json_str = self.json_content.join("\n");
        serde_json::from_str(&json_str).map_err(|e| format!("JSON 解析错误: {}", e))
    }

    /// 解析原始 JSON，返回 (名称, 配置JSON字符串)
    pub fn parse_raw_json(&self) -> Result<(String, String), String> {
        let json_str = self.json_content.join("\n");
        let json: serde_json::Value =
            serde_json::from_str(&json_str).map_err(|e| format!("JSON 解析错误: {}", e))?;

        let obj = json
            .as_object()
            .ok_or_else(|| "JSON 必须是对象格式".to_string())?;

        let (name, config) = obj
            .iter()
            .next()
            .ok_or_else(|| "JSON 对象不能为空".to_string())?;

        // 将配置部分格式化为 JSON 字符串
        let config_str =
            serde_json::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;

        Ok((name.clone(), config_str))
    }

    /// 获取服务器名称（从 JSON 解析）
    pub fn get_name(&self) -> String {
        if let Ok(json) = self.parse_json() {
            if let Some(obj) = json.as_object() {
                if let Some(name) = obj.keys().next() {
                    return name.clone();
                }
            }
        }
        String::new()
    }

    /// 构建 McpServer 对象
    /// 使用统一的 McpServer::from_json 方法解析 JSON 配置
    pub fn build_server(&self) -> McpServer {
        let json_str = self.json_content.join("\n");

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
            // 获取第一个 key 作为服务器名称，其 value 为配置
            if let Some(obj) = json.as_object() {
                if let Some((_name, config)) = obj.iter().next() {
                    // 使用统一的 McpServer::from_json 方法
                    return McpServer::from_json(config)
                        .unwrap_or_else(|_| McpServer::new_local(vec![], HashMap::new()));
                }
            }
        }
        // 解析失败，返回空服务器
        McpServer::new_local(vec![], HashMap::new())
    }

    /// 渲染表单
    pub fn render(&self, frame: &mut Frame, theme: &Theme, area: Rect) {
        if !self.visible {
            return;
        }

        // 如果显示关闭确认对话框
        if self.confirm_close {
            self.render_close_confirm(frame, theme, area);
            return;
        }

        // 计算弹窗大小
        let popup_area = centered_rect(MCP_FORM_WIDTH_PERCENT, MCP_FORM_HEIGHT, area);
        frame.render_widget(Clear, popup_area);

        // 标题显示修改状态
        let modified_mark = if self.has_unsaved_changes() { " *" } else { "" };
        let title = if self.is_edit_mode {
            format!(" 编辑 MCP 服务器{} ", modified_mark)
        } else {
            format!(" 添加 MCP 服务器{} ", modified_mark)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.active_border_style())
            .title(Span::styled(title, theme.title_style()));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // 布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // JSON 编辑器
                Constraint::Length(2), // 提示/错误信息
            ])
            .split(inner);

        // JSON 编辑器
        let template_name = self.get_template_name();
        let json_title = format!(" {} [Ctrl+T 切换] ", template_name);
        let json_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.active_border_style())
            .title(Span::styled(
                json_title,
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ));

        let json_inner = json_block.inner(chunks[0]);
        frame.render_widget(json_block, chunks[0]);

        // 渲染 JSON 内容（带行号）
        let json_lines: Vec<Line> = self
            .json_content
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let line_num = format!("{:2} ", i + 1);
                Line::from(vec![
                    Span::styled(line_num, theme.muted_style()),
                    Span::styled(line.as_str(), Style::default().fg(theme.info)),
                ])
            })
            .collect();

        let json_para = Paragraph::new(json_lines);
        frame.render_widget(json_para, json_inner);

        // 光标
        let cursor_x = json_inner.x + 3 + self.json_cursor_col as u16;
        let cursor_y = json_inner.y + self.json_cursor_row as u16;
        if cursor_y < json_inner.y + json_inner.height {
            frame.set_cursor_position((cursor_x, cursor_y));
        }

        // 底部提示/错误
        let hint_line = if let Some(ref err) = self.error_message {
            Line::from(Span::styled(err.as_str(), theme.error_style()))
        } else {
            Line::from(vec![
                Span::styled("[Ctrl+T]", Style::default().fg(theme.info)),
                Span::raw(" 切换 "),
                Span::styled("[Ctrl+D]", Style::default().fg(theme.warning)),
                Span::raw(" 清空 "),
                Span::styled("[Ctrl+S]", Style::default().fg(theme.success)),
                Span::raw(" 保存 "),
                Span::styled("[Esc]", Style::default().fg(theme.error)),
                Span::raw(" 取消"),
            ])
        };

        let hint_para = Paragraph::new(hint_line);
        frame.render_widget(hint_para, chunks[1]);
    }

    /// 渲染关闭确认对话框
    fn render_close_confirm(&self, frame: &mut Frame, theme: &Theme, area: Rect) {
        // 小型确认对话框，使用 saturating_sub 防止下溢
        const DIALOG_WIDTH: u16 = 50;
        const DIALOG_HEIGHT: u16 = 8;

        let popup_area = Rect {
            x: (area.width / 2).saturating_sub(DIALOG_WIDTH / 2),
            y: (area.height / 2).saturating_sub(DIALOG_HEIGHT / 2),
            width: DIALOG_WIDTH.min(area.width),
            height: DIALOG_HEIGHT.min(area.height),
        };

        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.warning))
            .title(Span::styled(
                " ⚠ 确认关闭 ",
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "有未保存的修改，确定要放弃吗？",
                Style::default().fg(theme.fg),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "[Y]",
                    Style::default()
                        .fg(theme.error)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" 放弃修改  "),
                Span::styled(
                    "[N]",
                    Style::default()
                        .fg(theme.success)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" 继续编辑  "),
                Span::styled(
                    "[Ctrl+S]",
                    Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" 保存"),
            ]),
        ];

        let paragraph = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, inner);
    }
}

/// 创建居中矩形
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let vertical_padding = (r.height.saturating_sub(height)) / 2;

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(vertical_padding),
            Constraint::Length(height),
            Constraint::Min(0),
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
