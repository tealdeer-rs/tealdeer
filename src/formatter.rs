//! Functions related to formatting and printing lines from a `Tokenizer`.

use std::io::BufRead;

use ansi_term::Colour;

use tokenizer::Tokenizer;
use types::LineType;

/// Print a token stream to an ANSI terminal.
pub fn print_lines<R>(tokenizer: &mut Tokenizer<R>) where R: BufRead {
    while let Some(token) = tokenizer.next() {
        match token {
            LineType::Empty => println!(""),
            LineType::Title(_) => debug!("Ignoring title"),
            LineType::Description(text) => println!("  {}", text),
            LineType::ExampleText(text) => println!("  {}", Colour::Green.paint(format!("- {}", text))),
            LineType::ExampleCode(text) => println!("  {}", Colour::Cyan.paint(format!("  {}", text))),
            LineType::Other(text) => debug!("Unknown line type: {:?}", text),
        }
    }
}
