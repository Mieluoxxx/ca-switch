// 主题系统模块

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

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
            border: Color::LightGreen,
            highlight: Color::Yellow,
            muted: Color::LightGreen,
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

    /// 激活状态的边框类型（双线边框，用于在颜色失效时标识焦点）
    pub fn active_border_type(&self) -> BorderType {
        BorderType::Double
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

    /// Provider 列表高亮样式（焦点在 Provider 时显示）
    pub fn provider_highlight_style(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }

    /// Model 列表高亮样式（焦点在 Model 时显示）
    pub fn model_highlight_style(&self) -> Style {
        Style::default()
            .fg(self.bg)
            .bg(self.highlight)
            .add_modifier(Modifier::BOLD)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
