//! Functions for printing pages to the terminal

use std::{
    borrow::Cow,
    io::{self, BufRead, BufReader, Read, Write},
};

use anyhow::{Context, Result};
use yansi::Paint;

use crate::{
    config::{Config, PlaceholderFormat, StyleConfig},
    formatter::{PageSnippet, highlight_lines},
    line_iterator::LineIterator,
};

/// Page content comes from tldr-pages, i.e. from outside this program. A page
/// must not be able to send control sequences (clear the screen, change the
/// terminal title, move the cursor, ...) to the user's terminal, so everything
/// that is printed is stripped of control characters first. Line breaks and
/// tabs are layout, not control: keep them.
fn sanitize_control_chars(s: &str) -> Cow<'_, str> {
    if !s.chars().any(is_unsafe_control_char) {
        return Cow::Borrowed(s);
    }

    Cow::Owned(
        s.chars()
            .map(|c| {
                if is_unsafe_control_char(c) {
                    '\u{FFFD}'
                } else {
                    c
                }
            })
            .collect(),
    )
}

fn is_unsafe_control_char(c: char) -> bool {
    c.is_control() && !matches!(c, '\n' | '\r' | '\t')
}

/// Set up display pager
///
/// SAFETY: this function may be called multiple times
#[cfg(not(target_os = "windows"))]
fn configure_pager(_: bool) {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| pager::Pager::with_default_pager("less -R").setup());
}

#[cfg(target_os = "windows")]
fn configure_pager(enable_styles: bool) {
    use crate::utils::print_warning;
    print_warning(enable_styles, "--pager flag not available on Windows!");
}

/// Print page by path
pub fn print_page(
    reader: impl Read,
    enable_markdown: bool,
    enable_styles: bool,
    use_pager: bool,
    config: &Config,
) -> Result<()> {
    let reader = BufReader::new(reader);

    // Configure pager if applicable
    if use_pager || config.display.use_pager {
        configure_pager(enable_styles);
    }

    // Lock stdout only once, this improves performance considerably
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if enable_markdown {
        // Print the raw markdown of the file.
        for line in reader.lines() {
            let line = line.context("Error while reading from a page")?;
            writeln!(handle, "{}", sanitize_control_chars(&line))
                .context("Could not write to stdout")?;
        }
    } else {
        // Closure that processes a page snippet and writes it to stdout
        let mut process_snippet = |snip: PageSnippet<&str>| {
            if snip.is_empty() {
                Ok(())
            } else {
                print_snippet(
                    &mut handle,
                    snip,
                    &config.style,
                    config.display.placeholder_format,
                )
                .context("Failed to print snippet")
            }
        };

        // Print highlighted lines
        highlight_lines(
            LineIterator::new(reader),
            &mut process_snippet,
            !config.display.compact,
            config.display.show_title,
            config.display.indent,
        )
        .context("Could not write to stdout")?;
    }

    // We're done outputting data, flush stdout now!
    handle.flush().context("Could not flush stdout")?;

    Ok(())
}

fn print_snippet(
    writer: &mut impl Write,
    snip: PageSnippet<&str>,
    style: &StyleConfig,
    placeholder_format: PlaceholderFormat,
) -> io::Result<()> {
    use PageSnippet::*;

    match snip {
        CommandName(s) | Title(s) => write!(
            writer,
            "{}",
            sanitize_control_chars(s).paint(style.command_name)
        ),
        Placeholder(s) => write!(
            writer,
            "{}",
            sanitize_control_chars(s).paint(style.example_variable)
        ),
        PlaceholderVariants { short, long } => match placeholder_format {
            PlaceholderFormat::Short => write!(
                writer,
                "{}",
                sanitize_control_chars(short).paint(style.example_code)
            ),
            PlaceholderFormat::Long => write!(
                writer,
                "{}",
                sanitize_control_chars(long).paint(style.example_code)
            ),
            PlaceholderFormat::Both => {
                write!(
                    writer,
                    "{}",
                    format!(
                        "[{}|{}]",
                        sanitize_control_chars(short),
                        sanitize_control_chars(long)
                    )
                    .paint(style.example_code)
                )
            }
        },
        NormalCode(s) => write!(
            writer,
            "{}",
            sanitize_control_chars(s).paint(style.example_code)
        ),
        Description(s) => write!(
            writer,
            "{}",
            sanitize_control_chars(s).paint(style.description)
        ),
        Text(s) => write!(
            writer,
            "{}",
            sanitize_control_chars(s).paint(style.example_text)
        ),
        Indent(n) => write!(writer, "{:n$}", ' '),
        Linebreak => writeln!(writer),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_terminal_control_sequences() {
        assert_eq!(
            sanitize_control_chars("\x1b[2J\x1b]0;owned\x07"),
            "\u{FFFD}[2J\u{FFFD}]0;owned\u{FFFD}"
        );
        assert_eq!(
            sanitize_control_chars("bell\x07 and nul\x00"),
            "bell\u{FFFD} and nul\u{FFFD}"
        );
    }

    #[test]
    fn keeps_layout_whitespace() {
        assert_eq!(sanitize_control_chars("a\nb\tc\r\nd"), "a\nb\tc\r\nd");
    }

    #[test]
    fn leaves_plain_text_borrowed() {
        assert!(matches!(
            sanitize_control_chars("plain tldr content"),
            Cow::Borrowed("plain tldr content")
        ));
    }
}
