// 应用范围选择对话框

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::theme::Theme;

/// 获取当前项目路径显示（将 home 目录替换为 ~）
fn get_project_path_display() -> String {
    let current_dir = std::env::current_dir().unwrap_or_default();
    let home_dir = dirs::home_dir().unwrap_or_default();

    if let Some(home) = home_dir.to_str() {
        if let Some(path) = current_dir.to_str() {
            if path.starts_with(home) {
                let relative = &path[home.len()..];
                if relative.is_empty() {
                    return "~".to_string();
                }
                // 去掉开头的斜杠（如果有）
                let relative = relative.strip_prefix('/').unwrap_or(relative);
                return format!("~/{}", relative);
            }
        }
    }
    current_dir.to_string_lossy().to_string()
}

/// 截断过长的路径，在中间使用 ... 省略
fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    // 对于形如 ~/a/b/c/d 的路径，优先保留开头和结尾
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() <= 2 {
        // 路径太短，直接截断
        return format!("{}...", &path[..max_len.saturating_sub(3)]);
    }

    // 保留开头 ~/ 和最后的文件夹名
    let start = parts[0]; // ~
    let end = parts[parts.len() - 1]; // 最后的文件夹
    let middle = if parts.len() > 3 { "..." } else { parts[1] };

    let result = format!("{}/{}/{}", start, middle, end);
    if result.len() <= max_len {
        result
    } else {
        // 还是太长，只保留开头和...
        format!(
            "{}/.../{}...",
            start,
            &end[..((max_len - start.len() - 7).max(3))]
        )
    }
}

/// 应用范围选择对话框
#[derive(Debug, Clone)]
pub struct ApplyScopeDialog {
    pub title: String,
    pub provider_name: String,
    pub provider_names: Vec<String>, // 所有选中的Provider列表
    pub visible: bool,
    pub apply_to_global: bool,
    pub apply_to_project: bool,
    pub selected_option: usize, // 0=全局, 1=项目, 2=两者
}

impl Default for ApplyScopeDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplyScopeDialog {
    pub fn new() -> Self {
        Self {
            title: "应用配置".to_string(),
            provider_name: String::new(),
            provider_names: Vec::new(),
            visible: false,
            apply_to_global: true,
            apply_to_project: false,
            selected_option: 0,
        }
    }

    /// 显示对话框（单个Provider）
    #[allow(dead_code)]
    pub fn show(&mut self, provider_name: &str) {
        self.provider_name = provider_name.to_string();
        self.provider_names = vec![provider_name.to_string()];
        self.visible = true;
        self.selected_option = 0;
        self.update_state_from_option();
    }

    /// 显示对话框（多个Provider）
    pub fn show_multiple(&mut self, provider_names: &[String]) {
        self.provider_names = provider_names.to_vec();
        self.title = "应用配置".to_string();
        // 显示第一个或摘要信息
        if provider_names.len() == 1 {
            self.provider_name = provider_names[0].clone();
        } else {
            self.provider_name = format!("{} 个 Provider", provider_names.len());
        }
        self.visible = true;
        self.selected_option = 0;
        self.update_state_from_option();
    }

    /// 显示清空 MCP 配置对话框（空选择时使用）
    pub fn show_clear_mcp(&mut self) {
        self.provider_names.clear();
        self.title = "清空 MCP 配置".to_string();
        self.provider_name = "将清空目标配置中的所有 MCP 服务器".to_string();
        self.visible = true;
        self.selected_option = 0;
        self.update_state_from_option();
    }

    /// 隐藏对话框
    pub fn hide(&mut self) {
        self.visible = false;
        self.provider_names.clear();
    }

    /// 切换选项
    pub fn toggle_option(&mut self) {
        self.selected_option = (self.selected_option + 1) % 3;
        self.update_state_from_option();
    }

    /// 根据当前 selected_option 的值更新 apply_to_global 和 apply_to_project 状态
    ///
    /// - selected_option = 0: 仅项目配置 (apply_to_global=false, apply_to_project=true)
    /// - selected_option = 1: 仅全局配置 (apply_to_global=true, apply_to_project=false)
    /// - selected_option = 2: 两者都应用 (apply_to_global=true, apply_to_project=true)
    pub fn update_state_from_option(&mut self) {
        match self.selected_option {
            0 => {
                self.apply_to_global = false;
                self.apply_to_project = true;
            }
            1 => {
                self.apply_to_global = true;
                self.apply_to_project = false;
            }
            2 => {
                self.apply_to_global = true;
                self.apply_to_project = true;
            }
            _ => {}
        }
    }

    /// 获取应用目标描述
    pub fn get_target_description(&self) -> String {
        match (self.apply_to_global, self.apply_to_project) {
            (true, true) => "全局配置 + 当前项目".to_string(),
            (true, false) => "全局配置".to_string(),
            (false, true) => "当前项目".to_string(),
            (false, false) => "无".to_string(),
        }
    }

    /// 渲染对话框
    pub fn render(&self, frame: &mut Frame, theme: &Theme, area: Rect) {
        if !self.visible {
            return;
        }

        // 计算对话框大小（居中）
        let popup_width = 54;
        let popup_height = 14;
        let x = (area.width.saturating_sub(popup_width)) / 2;
        let y = (area.height.saturating_sub(popup_height)) / 2;
        let dialog_area = Rect::new(x, y, popup_width, popup_height);

        // 绘制背景清除
        frame.render_widget(Clear, dialog_area);

        // 创建主块
        let block = Block::default()
            .title(format!(" {} ", self.title))
            .title_style(theme.title_style())
            .borders(Borders::ALL)
            .border_style(theme.active_border_style());

        // 内部内容区域
        let inner_area = block.inner(dialog_area);
        frame.render_widget(block, dialog_area);

        // 内容布局
        let content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Provider 名称标签
                Constraint::Length(1), // Provider 名称值
                Constraint::Length(1), // 空行
                Constraint::Length(3), // 选项区域（三个选项）
                Constraint::Length(1), // 空行
                Constraint::Length(3), // 操作提示
                Constraint::Length(2), // 状态行
            ])
            .split(inner_area);

        // 1-2. 名称信息（根据模式显示不同内容）
        let is_clear_mode = self.title.contains("清空");
        if is_clear_mode {
            // 清空模式：显示警告提示
            let warning = Paragraph::new("⚠ 注意:").style(
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(warning, content_layout[0]);

            let name_value = Paragraph::new(self.provider_name.clone())
                .style(Style::default().fg(theme.warning));
            frame.render_widget(name_value, content_layout[1]);
        } else {
            // 正常模式：显示 Provider/MCP 服务器信息
            let label = if self.title.contains("MCP") {
                "MCP 服务器:"
            } else {
                "Provider:"
            };
            let name_label = Paragraph::new(label).style(theme.muted_style());
            frame.render_widget(name_label, content_layout[0]);

            let name_value = Paragraph::new(self.provider_name.clone()).style(
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            );
            frame.render_widget(name_value, content_layout[1]);
        }

        // 3. 空行
        let spacer = Paragraph::new("");
        frame.render_widget(spacer, content_layout[2]);

        // 4-6. 选项
        let option_style = Style::default().fg(Color::White);
        let selected_style = Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD);

        let project_path = get_project_path_display();
        // 截断过长的路径（对话框宽度54，减去前缀和后缀，保留约30个字符）
        let truncated_path = truncate_path(&project_path, 30);

        let opt1 = if self.selected_option == 0 {
            Span::styled(
                format!("▶ 当前项目 {}/.opencode/", truncated_path),
                selected_style,
            )
        } else {
            Span::styled(
                format!("  当前项目 {}/.opencode/", truncated_path),
                option_style,
            )
        };
        let opt2 = if self.selected_option == 1 {
            Span::styled("▶ 全局配置 ~/.opencode/", selected_style)
        } else {
            Span::styled("  全局配置 ~/.opencode/", option_style)
        };
        let opt3 = if self.selected_option == 2 {
            Span::styled("▶ 两者都应用", selected_style)
        } else {
            Span::styled("  两者都应用", option_style)
        };

        let options_text = Text::from(vec![Line::from(opt1), Line::from(opt2), Line::from(opt3)]);
        let options_widget = Paragraph::new(options_text).style(theme.muted_style());
        frame.render_widget(options_widget, content_layout[3]);

        // 4. 空行
        let spacer = Paragraph::new("");
        frame.render_widget(spacer, content_layout[4]);

        // 5. 操作提示
        let hint = Text::from(vec![Line::from(vec![
            Span::styled("↑/↓/j/k: 切换选项  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter: 确认", Style::default().fg(Color::DarkGray)),
            Span::styled("  Esc: 取消", Style::default().fg(Color::DarkGray)),
        ])]);
        let hint_widget = Paragraph::new(hint)
            .alignment(Alignment::Center)
            .style(theme.muted_style());
        frame.render_widget(hint_widget, content_layout[5]);

        // 6. 当前选择状态
        let target = self.get_target_description();
        let (status_text, status_style) = if is_clear_mode {
            (
                format!("将清空: {} 的 MCP 配置", target),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            (
                format!("将应用到: {}", target),
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            )
        };
        let status = Paragraph::new(status_text)
            .style(status_style)
            .alignment(Alignment::Center);
        frame.render_widget(status, content_layout[6]);
    }
}
