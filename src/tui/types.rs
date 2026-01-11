// TUI 通用类型定义

/// 应用 Tab 页
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppTab {
    #[default]
    Providers, // Provider + Model 管理（合并后）
    Mcp,    // MCP 服务器管理
    Backup, // 备份恢复
    Status, // 状态监控
}

impl AppTab {
    /// 获取所有 Tab
    pub fn all() -> &'static [AppTab] {
        &[
            AppTab::Providers,
            AppTab::Mcp,
            AppTab::Backup,
            AppTab::Status,
        ]
    }

    /// 获取 Tab 标题
    pub fn title(&self) -> &'static str {
        match self {
            AppTab::Providers => "Providers",
            AppTab::Mcp => "MCP",
            AppTab::Backup => "Backup",
            AppTab::Status => "Status",
        }
    }

    /// 切换到下一个 Tab
    pub fn next(&self) -> Self {
        match self {
            AppTab::Providers => AppTab::Mcp,
            AppTab::Mcp => AppTab::Backup,
            AppTab::Backup => AppTab::Status,
            AppTab::Status => AppTab::Providers,
        }
    }

    /// 切换到上一个 Tab
    pub fn prev(&self) -> Self {
        match self {
            AppTab::Providers => AppTab::Status,
            AppTab::Mcp => AppTab::Providers,
            AppTab::Backup => AppTab::Mcp,
            AppTab::Status => AppTab::Backup,
        }
    }

    /// 获取 Tab 索引
    pub fn index(&self) -> usize {
        match self {
            AppTab::Providers => 0,
            AppTab::Mcp => 1,
            AppTab::Backup => 2,
            AppTab::Status => 3,
        }
    }
}

/// 输入模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal, // 正常导航模式
    Editing, // 编辑/输入模式
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Info,
    Success,
    #[allow(dead_code)] // 保留用于将来警告消息
    Warning,
    Error,
}

/// 状态消息
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub content: String,
    pub msg_type: MessageType,
    pub timestamp: std::time::Instant,
}

impl StatusMessage {
    pub fn new(content: String, msg_type: MessageType) -> Self {
        Self {
            content,
            msg_type,
            timestamp: std::time::Instant::now(),
        }
    }

    /// 检查消息是否过期 (默认3秒后过期)
    pub fn is_expired(&self) -> bool {
        self.timestamp.elapsed() > std::time::Duration::from_secs(3)
    }
}

/// 操作日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub message: String,
    pub level: MessageType,
    pub time: chrono::DateTime<chrono::Local>,
}

impl LogEntry {
    pub fn new(message: String, level: MessageType) -> Self {
        Self {
            message,
            level,
            time: chrono::Local::now(),
        }
    }

    /// 格式化时间
    pub fn formatted_time(&self) -> String {
        self.time.format("%H:%M:%S").to_string()
    }
}
