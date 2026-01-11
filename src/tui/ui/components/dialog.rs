// 确认对话框组件

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::tui::theme::Theme;

/// 对话框结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogResult {
    Confirm,
    Cancel,
}

/// 确认对话框
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub selected: usize, // 0 = confirm, 1 = cancel
    pub visible: bool,
}

impl ConfirmDialog {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            confirm_text: "确认".to_string(),
            cancel_text: "取消".to_string(),
            selected: 1, // 默认选中取消
            visible: false,
        }
    }

    pub fn with_buttons(mut self, confirm: &str, cancel: &str) -> Self {
        self.confirm_text = confirm.to_string();
        self.cancel_text = cancel.to_string();
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.selected = 1;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn toggle_selection(&mut self) {
        self.selected = if self.selected == 0 { 1 } else { 0 };
    }

    pub fn confirm(&self) -> DialogResult {
        if self.selected == 0 {
            DialogResult::Confirm
        } else {
            DialogResult::Cancel
        }
    }

    pub fn render(&self, frame: &mut Frame, theme: &Theme, area: Rect) {
        if !self.visible {
            return;
        }

        let popup_area = centered_rect(50, 30, area);
        frame.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.active_border_style())
            .title(Span::styled(
                format!(" {} ", self.title),
                theme.title_style(),
            ));

        let inner = block.inner(popup_area);
        frame.render_widget(block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(inner);

        // 消息
        let message = Paragraph::new(self.message.clone()).style(Style::default().fg(theme.fg));
        frame.render_widget(message, chunks[0]);

        // 按钮
        let confirm_style = if self.selected == 0 {
            Style::default()
                .fg(theme.bg)
                .bg(theme.success)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.success)
        };

        let cancel_style = if self.selected == 1 {
            Style::default()
                .fg(theme.bg)
                .bg(theme.error)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.error)
        };

        let buttons = Line::from(vec![
            Span::raw("  "),
            Span::styled(format!(" {} ", self.confirm_text), confirm_style),
            Span::raw("    "),
            Span::styled(format!(" {} ", self.cancel_text), cancel_style),
        ]);

        let buttons_para = Paragraph::new(buttons);
        frame.render_widget(buttons_para, chunks[1]);
    }
}

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
