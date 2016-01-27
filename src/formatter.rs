//! Functions related to formatting and printing lines from a `Tokenizer`.

use std::io::BufRead;

use ansi_term::{Colour, ANSIStrings};

use tokenizer::Tokenizer;
use types::LineType;


/// Provide formatting for {{ curly braces }} in ExampleCode lines
fn format_braces(text: &str) -> String {
    let parts = text.split("{{").flat_map(|s| s.split("}}"))
                    .enumerate()
                    .map(|(i, v)| {
                        if i % 2 == 0 {
                            Colour::Cyan.paint(v)
                        } else {
                            Colour::Cyan.underline().paint(v)
                        }
                    })
                    .collect::<Vec<_>>();
    ANSIStrings(&parts).to_string()
}

/// Print a token stream to an ANSI terminal.
pub fn print_lines<R>(tokenizer: &mut Tokenizer<R>) where R: BufRead {
    while let Some(token) = tokenizer.next_token() {
        match token {
            LineType::Empty => println!(""),
            LineType::Title(_) => debug!("Ignoring title"),
            LineType::Description(text) => println!("  {}", text),
            LineType::ExampleText(text) => println!("  {}", Colour::Green.paint(format!("- {}", text))),
            LineType::ExampleCode(text) => println!("  {}", &format_braces(&text)),
            LineType::Other(text) => debug!("Unknown line type: {:?}", text),
        }
    }
    println!("");
}
