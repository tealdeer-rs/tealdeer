//! Functions related to formatting and printing lines from a `Tokenizer`.

use std::fmt::Write;
use std::io::BufRead;

use ansi_term::{Colour, Style};

use tokenizer::Tokenizer;
use types::LineType;

/// Provide formatting for {{ curly braces }} in ExampleCode lines
fn format_code(command: &str, text: &str) -> String {
    let mut parts = String::new();
    parts.reserve(text.len() * 3);

    for part in text.split("}}") {
        let var_begin_op = part.find("{{");
        let command_begin_op = part[..var_begin_op.unwrap_or(part.len())].find(&command);

        let begin = command_begin_op.map(|v| v + command.len()).unwrap_or(0);
        let end = var_begin_op.unwrap_or(part.len());

        if let Some(command_begin) =  command_begin_op {
            parts.push_str(&part[..command_begin]);

            write!(parts, "{}", Colour::Red.paint(command)).unwrap();
        }

        parts.push_str(&part[begin..end]);

        if let Some(var_begin) = var_begin_op {
            let var_slice = &part[var_begin + 2..];
            write!(parts, "{}", Style::default().underline().paint(var_slice)).unwrap();
        }
    }

    parts
}

/// Print a token stream to an ANSI terminal.
pub fn print_lines<R>(tokenizer: &mut Tokenizer<R>) where R: BufRead {
    let mut command = String::new();
    while let Some(token) = tokenizer.next_token() {
        match token {
            LineType::Empty => println!(""),
            LineType::Title(title) => {
                debug!("Ignoring title");

                // This is safe as long as the parsed title is only the command,
                // and tokenizer yields values in order of appearance.
                command = title;
            },
            LineType::Description(text) => println!("  {}", Colour::White.paint(text)),
            LineType::ExampleText(text) => println!("  {}", text),
            LineType::ExampleCode(text) => println!("      {}", &format_code(&command, &text)),
            LineType::Other(text) => debug!("Unknown line type: {:?}", text),
        }
    }
    println!("");
}
