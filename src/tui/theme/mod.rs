// 主题系统模块

use ratatui::style::{Color, Modifier, Style};

/// 应用主题
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
    pub bg: Color,
    pub fg: Color,
    pub border: Color,
    pub highlight: Color,
    pub muted: Color,
}

impl Theme {
    /// 暗色主题 (默认)
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            bg: Color::Reset,
            fg: Color::White,
            border: Color::DarkGray,
            highlight: Color::Yellow,
            muted: Color::DarkGray,
        }
    }

    // === 样式辅助方法 ===

    /// 标题样式
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }

    /// 高亮样式
    pub fn highlight_style(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    /// 边框样式
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    /// 激活边框样式
    pub fn active_border_style(&self) -> Style {
        Style::default().fg(self.primary)
    }

    /// 成功消息样式
    pub fn success_style(&self) -> Style {
        Style::default().fg(self.success)
    }

    /// 错误消息样式
    pub fn error_style(&self) -> Style {
        Style::default().fg(self.error)
    }

    /// 警告消息样式
    pub fn warning_style(&self) -> Style {
        Style::default().fg(self.warning)
    }

    /// 信息消息样式
    pub fn info_style(&self) -> Style {
        Style::default().fg(self.info)
    }

    /// 静音文本样式
    pub fn muted_style(&self) -> Style {
        Style::default().fg(self.muted)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
