extern crate ansi_term;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::{env, process};
use std::convert::From;


#[derive(Debug, Eq, PartialEq)]
enum LineType {
    Empty,
    Title(String),
    Description(String),
    ExampleText(String),
    ExampleCode(String),
    Other(String),
}

impl<'a> From<&'a str> for LineType {
    /// Convert a string slice to a LineType. Newlines and whitespace are trimmed.
    fn from(line: &'a str) -> LineType {
        let trimmed = line.trim();
        let mut chars = trimmed.chars();
        match chars.next() {
            None => LineType::Empty,
            Some('#') => LineType::Title(trimmed.trim_left_matches(|chr: char| chr == '#' || chr.is_whitespace()).into()),
            Some('>') => LineType::Description(trimmed.trim_left_matches(|chr: char| chr == '>' || chr.is_whitespace()).into()),
            Some('-') => LineType::ExampleText(trimmed.trim_left_matches(|chr: char| chr == '-' || chr.is_whitespace()).into()),
            Some('`') if chars.last() == Some('`') => LineType::ExampleCode(trimmed.trim_matches(|chr: char| chr == '`' || chr.is_whitespace()).into()),
            _ => LineType::Other(trimmed.into()),
        }
    }
}


#[derive(Debug)]
struct Tokenizer<R: BufRead> {
    reader: R,
    current_line: String,
}

impl<R> Tokenizer<R> where R: BufRead {

    fn new(reader: R) -> Tokenizer<R> {
        Tokenizer {
            reader: reader,
            current_line: String::new(),
        }
    }

    fn next(&mut self) -> Option<LineType> {
        self.current_line.clear();
        let bytes_read = self.reader.read_line(&mut self.current_line).unwrap();
        match bytes_read {
            0 => None,
            _ => Some(LineType::from(&self.current_line[..])),
        }
    }

}

/// Open file, return a `BufRead` instance
fn get_file_reader(filepath: &str) -> Result<BufReader<File>, String> {
    let file = try!(
        File::open(filepath)
            .map_err(|msg| format!("Could not open file: {}", msg))
    );
    Ok(BufReader::new(file))
}


fn main() {

    // Parse arguments
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <command>", args[0]);
        process::exit(1);
    }

    // Open file
    let reader = get_file_reader(&args[1]).unwrap_or_else(|msg| {
        println!("{}", msg);
        process::exit(1);
    });

    // Create tokenizer
    let mut tokenizer = Tokenizer::new(reader);

    // Tokenize and print output
    while let Some(token) = tokenizer.next() {
        println!("{:?}", token);
    }

}


#[cfg(test)]
mod test {
    use super::LineType;

    #[test]
    fn test_linetype_from_str() {
        assert_eq!(LineType::from(""), LineType::Empty);
        assert_eq!(LineType::from(" \n \r"), LineType::Empty);
        assert_eq!(LineType::from("# Hello there"), LineType::Title("Hello there".into()));
        assert_eq!(LineType::from("> tis a description \n"), LineType::Description("tis a description".into()));
        assert_eq!(LineType::from("- some command"), LineType::ExampleText("some command".into()));
        assert_eq!(LineType::from("`$ cargo run`"), LineType::ExampleCode("$ cargo run".into()));
        assert_eq!(LineType::from("`$ cargo run"), LineType::Other("`$ cargo run".into()));
        assert_eq!(LineType::from("asdf"), LineType::Other("asdf".into()));
    }
}
