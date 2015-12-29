extern crate ansi_term;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::{env, process};


enum LineType {
    Title(String),
    Empty(String),
    Description(String),
    ExampleText(String),
    ExampleCode(String),
}

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

    fn next(&mut self) -> Option<&str> {
        self.current_line.clear();
        let bytes_read = self.reader.read_line(&mut self.current_line).unwrap();
        match bytes_read {
            0 => None,
            _ => Some(&self.current_line),
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
        print!("{}", token);
    }

}
