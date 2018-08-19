//! Functions related to formatting and printing lines from a `Tokenizer`.

use std::fmt::Write;
use std::io::BufRead;

use ansi_term::ANSIStrings;

use config::Config;
use tokenizer::Tokenizer;
use types::LineType;

/// Format and highlight code examples including variables in {{ curly braces }}.
fn format_code(command: &str, text: &str, config: &Config) -> String {
    let mut parts = Vec::new();
    for between_commands in text.split(&command) {
        parts.push(config.highlight_style.paint(command));

        for between_variables in between_commands.split("}}") {
            if let Some(variable_start) = between_variables.find("{{") {
                let example_code = &between_variables[..variable_start];
                let example_variable = &between_variables[variable_start + 2..];

                parts.push(config.example_code_style.paint(example_code));
                parts.push(config.example_variable_style.paint(example_variable));
            }
            else {
                parts.push(config.example_code_style.paint(between_variables));
            }
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
