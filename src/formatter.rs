//! Functions related to formatting and printing lines from a `Tokenizer`.

use std::fmt::Write;
use std::io::BufRead;

use config::Config;
use tokenizer::Tokenizer;
use types::LineType;

/// Provide formatting for {{ curly braces }} in ExampleCode lines
fn format_code(command: &str, text: &str, config: &Config) -> String {
    let mut parts = String::new();
    parts.reserve(text.len() * 3);

    for part in text.split("}}") {
        let var_begin_op = part.find("{{");
        let command_begin_op = part[..var_begin_op.unwrap_or(part.len())].find(&command);

        let begin = command_begin_op.map(|v| v + command.len()).unwrap_or(0);
        let end = var_begin_op.unwrap_or(part.len());

        if let Some(command_begin) =  command_begin_op {
            write!(parts, "{}", config.example_code_style.paint(&part[..command_begin])).unwrap();

            write!(parts, "{}", config.highlight_style.paint(command)).unwrap();
        }

        write!(parts, "{}", config.example_code_style.paint(&part[begin..end])).unwrap();

        if let Some(var_begin) = var_begin_op {
            let var_slice = &part[var_begin + 2..];
            write!(parts, "{}", config.example_variable_stlye.paint(var_slice)).unwrap();
        }
    }

    parts
}

fn format_description(description: &str, config: &Config) -> String {
    if let Some(first_space) = description.find(' ') {
        let mut highlighted_description = String::new();
        write!(
            highlighted_description,
            "{}",
            config.highlight_style.paint(&description[..first_space])
        ).unwrap();

        write!(
            highlighted_description,
            "{}",
            config.description_style.paint(&description[first_space..])
        ).unwrap();

        return highlighted_description;
    }

    String::from(description)
}

/// Print a token stream to an ANSI terminal.
pub fn print_lines<R>(tokenizer: &mut Tokenizer<R>, config: &Config) where R: BufRead {
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
            LineType::Description(text) => println!("  {}", format_description(&text, &config)),
            LineType::ExampleText(text) => println!("  {}", config.example_text_style.paint(text)),
            LineType::ExampleCode(text) => println!("      {}", &format_code(&command, &text, &config)),
            LineType::Other(text) => debug!("Unknown line type: {:?}", text),
        }
    }
    println!("");
}
