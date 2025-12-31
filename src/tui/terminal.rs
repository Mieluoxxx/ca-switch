// 终端管理器
// 负责初始化和恢复终端状态

use std::io::{self, Stdout};

use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// 终端类型别名
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// 初始化终端
/// 进入 raw mode 和备用屏幕
pub fn init() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// 恢复终端
/// 退出 raw mode 和备用屏幕
pub fn restore() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// 终端守卫
/// 确保在 panic 时也能正确恢复终端
pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = restore();
    }
}

impl Default for TerminalGuard {
    fn default() -> Self {
        Self::new()
    }
}
