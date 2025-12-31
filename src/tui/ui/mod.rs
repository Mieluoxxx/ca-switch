// UI 渲染模块

pub mod components;
mod layout;

use ratatui::Frame;

use super::{app::App, theme::Theme};

pub use components::DialogResult;
pub use layout::render;

/// 渲染主界面
pub fn draw(frame: &mut Frame, app: &mut App, theme: &Theme) {
    render(frame, app, theme);
}
