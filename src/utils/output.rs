// Output utility functions for terminal display

use console::style;

/// Message type for controlling output style and icon
pub enum MessageType {
    Success,
    Error,
    Info,
    Warning,
}

impl MessageType {
    fn emoji(&self) -> &'static str {
        match self {
            MessageType::Success => "✨",
            MessageType::Error => "❌",
            MessageType::Info => "ℹ️",
            MessageType::Warning => "⚠️",
        }
    }

    fn styled<T>(&self, text: T) -> console::StyledObject<T> {
        match self {
            MessageType::Success => style(text).green(),
            MessageType::Error => style(text).red(),
            MessageType::Info => style(text).blue(),
            MessageType::Warning => style(text).yellow(),
        }
    }
}

/// Display a formatted message with the specified type
pub fn show_message(message: &str, msg_type: MessageType) {
    println!(
        "{} {}",
        msg_type.styled(msg_type.emoji()),
        msg_type.styled(message)
    );
}

/// Display a success message
pub fn show_success(message: &str) {
    show_message(message, MessageType::Success);
}

/// Display an error message
pub fn show_error(message: &str) {
    show_message(message, MessageType::Error);
}

/// Display an info message
pub fn show_info(message: &str) {
    show_message(message, MessageType::Info);
}

/// Display a warning message
pub fn show_warning(message: &str) {
    show_message(message, MessageType::Warning);
}

/// Print a titled section with a separator line
pub fn print_section(title: &str) {
    println!("\n{}", style(title).cyan().bold());
    println!("{}", style("═".repeat(40)).dim());
}

/// Display path information with label
pub fn display_paths(label: &str, path: &std::path::Path) {
    println!("{} {}", style(label).white(), style(path.display()).cyan());
}
