// 事件处理器
// 负责处理键盘事件和定时事件

use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

/// TUI 事件类型
#[derive(Debug, Clone)]
pub enum Event {
    /// 键盘事件
    Key(KeyEvent),
    /// 定时刷新事件
    Tick,
    /// 终端尺寸变化
    Resize,
}

/// 事件处理器
pub struct EventHandler {
    /// 事件轮询间隔
    tick_rate: Duration,
}

impl EventHandler {
    /// 创建事件处理器
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    /// 获取下一个事件
    /// 阻塞等待直到有事件发生或超时
    pub fn next(&self) -> Result<Event, std::io::Error> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(_, _) => Ok(Event::Resize),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new(Duration::from_millis(100))
    }
}
