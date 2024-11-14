use yansi::{Color, Paint};

/// Print a warning to stderr. If `enable_styles` is true, then a yellow
/// message will be printed.
pub fn print_warning(enable_styles: bool, message: &str) {
    print_msg(enable_styles, message, "Warning: ", Color::Yellow);
}

/// Print an anyhow error to stderr. If `enable_styles` is true, then a red
/// message will be printed.
pub fn print_error(enable_styles: bool, error: &anyhow::Error) {
    print_msg(enable_styles, &format!("{error:?}"), "Error: ", Color::Red);
}

fn print_msg(enable_styles: bool, message: &str, prefix: &'static str, color: Color) {
    if enable_styles {
        eprintln!("{}{}", prefix.paint(color), message.paint(color));
    } else {
        eprintln!("{message}");
    }
}
