// TUI 模块入口
// 基于 Ratatui 的终端用户界面

pub mod app;
pub mod event;
pub mod handlers;
pub mod terminal;
pub mod theme;
pub mod types;
pub mod ui;

use std::io;

use app::App;
use event::{Event, EventHandler};
use handlers::handle_key_event;
use terminal::{init, restore, TerminalGuard};
use theme::Theme;

/// 运行 TUI 应用
pub fn run() -> io::Result<()> {
    // 初始化终端
    let mut terminal = init()?;
    let _guard = TerminalGuard::new();

    // 创建应用状态
    let mut app = App::new().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // 创建事件处理器
    let event_handler = EventHandler::default();

    // 创建主题
    let theme = Theme::default();

    // 主循环
    loop {
        // 渲染 UI
        terminal.draw(|frame| {
            ui::draw(frame, &mut app, &theme);
        })?;

        // 处理事件
        match event_handler.next()? {
            Event::Key(key) => {
                handle_key_event(&mut app, key);
            }
            Event::Tick => {
                // 定时任务 (如刷新数据)
            }
            Event::Resize => {
                // 终端尺寸变化时自动重绘
            }
        }

        // 检查是否退出
        if app.should_quit {
            break;
        }
    }

    // 恢复终端
    restore()?;

    Ok(())
}
