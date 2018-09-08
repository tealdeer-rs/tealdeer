//! Functions related to formatting and printing lines from a `Tokenizer`.

use std::fmt::Write;
use std::io::BufRead;

use ansi_term::{ANSIString, ANSIStrings};

use config::Config;
use tokenizer::Tokenizer;
use types::LineType;

fn highlight_command<'a>(
    command: &'a str,
    example_code: &'a str,
    config: &Config,
    parts: &mut Vec<ANSIString<'a>>
) {
    let mut code_part_end_pos = 0;
    while let Some(command_start) = example_code[code_part_end_pos..].find(&command) {
        let code_part = &example_code[code_part_end_pos..code_part_end_pos + command_start];
        parts.push(config.style.example_code.paint(code_part));
        parts.push(config.style.highlight.paint(command));

        code_part_end_pos += command_start + command.len();
    }
    parts.push(config.style.example_code.paint(&example_code[code_part_end_pos..]));
}

/// Format and highlight code examples including variables in {{ curly braces }}.
fn format_code(command: &str, text: &str, config: &Config) -> String {
    let mut parts = Vec::new();
    for between_variables in text.split("}}") {
        if let Some(variable_start) = between_variables.find("{{") {
            let example_code = &between_variables[..variable_start];
            let example_variable = &between_variables[variable_start + 2..];

            highlight_command(&command, &example_code, &config, &mut parts);
            parts.push(config.style.example_variable.paint(example_variable));
        } else {
            highlight_command(&command, &between_variables, &config, &mut parts);
        }
    }

    ANSIStrings(&parts).to_string()
}

/// Format and highlight description text.
fn format_description(description: &str, config: &Config) -> String {
    if let Some(first_space) = description.find(' ') {
        let mut highlighted_description = String::new();
        write!(
            highlighted_description,
            "{}",
            config.style.highlight.paint(&description[..first_space])
        ).unwrap();

        write!(
            highlighted_description,
            "{}",
            config.style.description.paint(&description[first_space..])
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
                debug!("Detected command name: {}", &command);
            },
            LineType::Description(text) => println!("  {}", format_description(&text, &config)),
            LineType::ExampleText(text) => println!("  {}", config.style.example_text.paint(text)),
            LineType::ExampleCode(text) => println!("      {}", &format_code(&command, &text, &config)),
            LineType::Other(text) => debug!("Unknown line type: {:?}", text),
        }
    }
    println!("");
}
