// 多选对话框组件

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::theme::Theme;

/// 多选对话框
#[derive(Debug, Clone)]
pub struct MultiSelectDialog {
    /// 标题
    pub title: String,
    /// 所有选项
    pub items: Vec<String>,
    /// 过滤后的选项索引
    filtered_indices: Vec<usize>,
    /// 已选中的项目索引（相对于原始 items）
    pub selected: Vec<usize>,
    /// 列表状态（当前高亮项）
    pub list_state: ListState,
    /// 是否可见
    pub visible: bool,
    /// 是否处于搜索模式
    pub search_mode: bool,
    /// 搜索关键词
    pub search_query: String,
    /// 是否正在加载
    pub loading: bool,
    /// 加载/错误消息
    pub message: Option<String>,
}

impl Default for MultiSelectDialog {
    fn default() -> Self {
        Self::new("选择项目")
    }
}

impl MultiSelectDialog {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            items: Vec::new(),
            filtered_indices: Vec::new(),
            selected: Vec::new(),
            list_state: ListState::default(),
            visible: false,
            search_mode: false,
            search_query: String::new(),
            loading: false,
            message: None,
        }
    }

    /// 设置选项列表
    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.filtered_indices = (0..self.items.len()).collect();
        self.selected.clear();
        self.search_query.clear();
        self.search_mode = false;
        if !self.filtered_indices.is_empty() {
            self.list_state.select(Some(0));
        } else {
            self.list_state.select(None);
        }
    }

    /// 设置已选中的项（用于预选）
    pub fn set_selected(&mut self, selected_items: &[String]) {
        self.selected.clear();
        for (i, item) in self.items.iter().enumerate() {
            if selected_items.contains(item) {
                self.selected.push(i);
            }
        }
    }

    /// 显示对话框
    pub fn show(&mut self) {
        self.visible = true;
        self.loading = false;
        self.message = None;
    }

    /// 显示加载状态
    pub fn show_loading(&mut self, message: &str) {
        self.visible = true;
        self.loading = true;
        self.message = Some(message.to_string());
    }

    /// 显示错误
    pub fn show_error(&mut self, message: &str) {
        self.loading = false;
        self.message = Some(message.to_string());
    }

    /// 隐藏对话框
    pub fn hide(&mut self) {
        self.visible = false;
        self.loading = false;
        self.search_mode = false;
        self.search_query.clear();
    }

    /// 进入搜索模式
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
    }

    /// 退出搜索模式
    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
        // 保留搜索结果，但退出编辑
    }

    /// 清除搜索
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.apply_filter();
    }

    /// 处理搜索输入
    pub fn handle_search_input(&mut self, c: char) {
        self.search_query.push(c);
        self.apply_filter();
    }

    /// 处理退格
    pub fn handle_search_backspace(&mut self) {
        self.search_query.pop();
        self.apply_filter();
    }

    /// 应用过滤
    fn apply_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.items.len()).collect();
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.filtered_indices = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.to_lowercase().contains(&query_lower))
                .map(|(i, _)| i)
                .collect();
        }

        // 调整选中状态
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else if let Some(i) = self.list_state.selected() {
            if i >= self.filtered_indices.len() {
                self.list_state
                    .select(Some(self.filtered_indices.len() - 1));
            }
        } else {
            self.list_state.select(Some(0));
        }
    }

    /// 选择下一项
    pub fn select_next(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_indices.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// 选择上一项
    pub fn select_prev(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_indices.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    /// 切换当前项的选中状态
    pub fn toggle_current(&mut self) {
        if let Some(filter_idx) = self.list_state.selected() {
            if let Some(&original_idx) = self.filtered_indices.get(filter_idx) {
                if self.selected.contains(&original_idx) {
                    self.selected.retain(|&x| x != original_idx);
                } else {
                    self.selected.push(original_idx);
                }
            }
        }
    }

    /// 全选当前过滤结果
    pub fn select_all(&mut self) {
        for &idx in &self.filtered_indices {
            if !self.selected.contains(&idx) {
                self.selected.push(idx);
            }
        }
    }

    /// 获取已选中的项目名称
    pub fn get_selected_items(&self) -> Vec<String> {
        self.selected
            .iter()
            .filter_map(|&i| self.items.get(i).cloned())
            .collect()
    }

    /// 渲染对话框
    pub fn render(&mut self, frame: &mut Frame, theme: &Theme, area: Rect) {
        if !self.visible {
            return;
        }

        // 居中弹窗
        let popup_area = centered_rect(70, 80, area);
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.active_border_style())
            .title(Span::styled(
                format!(
                    " {} ({}/{}) ",
                    self.title,
                    self.selected.len(),
                    self.items.len()
                ),
                theme.title_style(),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        // 如果正在加载
        if self.loading {
            let loading_text = self.message.as_deref().unwrap_or("加载中...");
            let text = Paragraph::new(format!("⏳ {}", loading_text)).style(theme.muted_style());
            frame.render_widget(text, inner);
            return;
        }

        // 如果有错误消息
        if let Some(ref msg) = self.message {
            if self.items.is_empty() {
                let text = Paragraph::new(format!("❌ {}", msg)).style(theme.error_style());
                frame.render_widget(text, inner);
                return;
            }
        }

        // 分割布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 搜索框
                Constraint::Min(5),    // 列表
                Constraint::Length(2), // 帮助
            ])
            .split(inner);

        // 搜索框
        let search_style = if self.search_mode {
            theme.active_border_style()
        } else {
            theme.border_style()
        };

        let search_block = Block::default()
            .borders(Borders::ALL)
            .border_style(search_style)
            .title(Span::styled(" / 搜索 ", theme.title_style()));

        let search_inner = search_block.inner(chunks[0]);
        frame.render_widget(search_block, chunks[0]);

        let search_text = if self.search_query.is_empty() {
            if self.search_mode {
                "输入关键词过滤..."
            } else {
                "按 / 开始搜索"
            }
        } else {
            &self.search_query
        };

        let search_style = if self.search_query.is_empty() && !self.search_mode {
            theme.muted_style()
        } else {
            Style::default().fg(theme.fg)
        };

        let cursor = if self.search_mode { "▌" } else { "" };
        let search_para = Paragraph::new(format!("{}{}", search_text, cursor)).style(search_style);
        frame.render_widget(search_para, search_inner);

        // 列表
        let list_items: Vec<ListItem> = self
            .filtered_indices
            .iter()
            .map(|&i| {
                let item = &self.items[i];
                let is_selected = self.selected.contains(&i);
                let checkbox = if is_selected { "[✓]" } else { "[ ]" };
                let style = if is_selected {
                    Style::default().fg(theme.success)
                } else {
                    Style::default().fg(theme.fg)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", checkbox), style),
                    Span::styled(item, style),
                ]))
            })
            .collect();

        let list = List::new(list_items)
            .highlight_style(theme.highlight_style())
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);

        // 帮助提示
        let help_text = if self.search_mode {
            "[Enter]确认搜索 [Esc]取消搜索"
        } else {
            "[Space]选择 [a]全选 [Enter]确认 [/]搜索 [Esc]取消"
        };
        let help = Paragraph::new(help_text).style(theme.muted_style());
        frame.render_widget(help, chunks[2]);
    }
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
