use ansi_term::{Color, Style};

/// Print a warning to stderr. If `enable_styles` is true, then a yellow
/// message will be printed.
pub fn print_warning(enable_styles: bool, message: &str) {
    print_msg(enable_styles, message, "Warning: ", Color::Yellow);
}

/// Print an error to stderr. If `enable_styles` is true, then a red message
/// will be printed.
pub fn print_error(enable_styles: bool, message: &str) {
    print_msg(enable_styles, message, "Error: ", Color::Red);
}

fn print_msg(enable_styles: bool, message: &str, prefix: &'static str, color: Color) {
    if enable_styles {
        let style = Style::new().fg(color);
        eprintln!("{}{}", style.paint(prefix), style.paint(message));
    } else {
        eprintln!("{}", message);
    }
}
