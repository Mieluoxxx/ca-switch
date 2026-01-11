// 输入表单组件

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use tui_input::Input;

use crate::tui::theme::Theme;

/// 表单字段
#[derive(Debug, Clone)]
pub struct FormField {
    pub label: String,
    pub input: Input,
    pub placeholder: String,
    pub is_password: bool,
    pub is_required: bool,
}

impl FormField {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            input: Input::default(),
            placeholder: String::new(),
            is_password: false,
            is_required: false,
        }
    }

    pub fn placeholder(mut self, text: &str) -> Self {
        self.placeholder = text.to_string();
        self
    }

    pub fn password(mut self) -> Self {
        self.is_password = true;
        self
    }

    pub fn required(mut self) -> Self {
        self.is_required = true;
        self
    }

    pub fn value(&self) -> &str {
        self.input.value()
    }

    pub fn set_value(&mut self, value: &str) {
        self.input = Input::new(value.to_string());
    }

    pub fn is_valid(&self) -> bool {
        if self.is_required {
            !self.input.value().trim().is_empty()
        } else {
            true
        }
    }
}

/// 输入表单
#[derive(Debug, Clone)]
pub struct InputForm {
    pub title: String,
    pub fields: Vec<FormField>,
    pub focused_field: usize,
    pub visible: bool,
}

impl InputForm {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            fields: Vec::new(),
            focused_field: 0,
            visible: false,
        }
    }

    pub fn add_field(mut self, field: FormField) -> Self {
        self.fields.push(field);
        self
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.focused_field = 0;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn clear(&mut self) {
        for field in &mut self.fields {
            field.input = Input::default();
        }
        self.focused_field = 0;
    }

    pub fn focus_next(&mut self) {
        if !self.fields.is_empty() {
            self.focused_field = (self.focused_field + 1) % self.fields.len();
        }
    }

    pub fn focus_prev(&mut self) {
        if !self.fields.is_empty() {
            self.focused_field = if self.focused_field == 0 {
                self.fields.len() - 1
            } else {
                self.focused_field - 1
            };
        }
    }

    pub fn handle_input(&mut self, c: char) {
        if let Some(field) = self.fields.get_mut(self.focused_field) {
            field.input.handle(tui_input::InputRequest::InsertChar(c));
        }
    }

    pub fn handle_backspace(&mut self) {
        if let Some(field) = self.fields.get_mut(self.focused_field) {
            field.input.handle(tui_input::InputRequest::DeletePrevChar);
        }
    }

    pub fn handle_delete(&mut self) {
        if let Some(field) = self.fields.get_mut(self.focused_field) {
            field.input.handle(tui_input::InputRequest::DeleteNextChar);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.fields.iter().all(|f| f.is_valid())
    }

    pub fn get_value(&self, index: usize) -> Option<&str> {
        self.fields.get(index).map(|f| f.value())
    }

    pub fn render(&self, frame: &mut Frame, theme: &Theme, area: Rect) {
        if !self.visible {
            return;
        }

        let height = (self.fields.len() * 3 + 6).min(80) as u16;
        let popup_area = centered_rect(60, height.min(area.height - 4), area);
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

        let mut constraints: Vec<Constraint> =
            self.fields.iter().map(|_| Constraint::Length(3)).collect();
        constraints.push(Constraint::Length(2)); // 按钮行
        constraints.push(Constraint::Min(0)); // 剩余空间

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // 渲染每个字段
        for (i, field) in self.fields.iter().enumerate() {
            let is_focused = i == self.focused_field;

            let border_style = if is_focused {
                theme.active_border_style()
            } else {
                theme.border_style()
            };

            let label_style = if is_focused {
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            let required_marker = if field.is_required { " *" } else { "" };

            let field_block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(Span::styled(
                    format!(" {}{} ", field.label, required_marker),
                    label_style,
                ));

            let field_inner = field_block.inner(chunks[i]);
            frame.render_widget(field_block, chunks[i]);

            let display_value = if field.is_password && !field.value().is_empty() {
                "*".repeat(field.value().len())
            } else if field.value().is_empty() && !is_focused {
                field.placeholder.clone()
            } else {
                field.value().to_string()
            };

            let text_style = if field.value().is_empty() && !is_focused {
                theme.muted_style()
            } else {
                Style::default().fg(theme.fg)
            };

            let text = Paragraph::new(display_value).style(text_style);
            frame.render_widget(text, field_inner);

            // 光标
            if is_focused {
                let cursor_x = field_inner.x + field.input.visual_cursor() as u16;
                let cursor_y = field_inner.y;
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }

        // 底部提示
        let hint = Line::from(vec![
            Span::styled("[Tab]", Style::default().fg(theme.primary)),
            Span::raw(" 下一项 "),
            Span::styled("[Enter]", Style::default().fg(theme.success)),
            Span::raw(" 提交 "),
            Span::styled("[Esc]", Style::default().fg(theme.error)),
            Span::raw(" 取消"),
        ]);
        let hint_para = Paragraph::new(hint);
        frame.render_widget(hint_para, chunks[self.fields.len()]);
    }
}

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
