// Output utility functions for terminal display

use console::style;

/// 显示成功消息
pub fn show_success(message: &str) {
    println!("{} {}", style("✨").green(), style(message).green());
}

/// 显示错误消息
pub fn show_error(message: &str) {
    println!("{} {}", style("❌").red(), style(message).red());
}

/// 显示信息消息
pub fn show_info(message: &str) {
    println!("{} {}", style("ℹ️ ").blue(), style(message).blue());
}
