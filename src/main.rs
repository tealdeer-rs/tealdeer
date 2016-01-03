//! An implementation of [tldr](https://github.com/tldr-pages/tldr) in Rust.

#[macro_use] extern crate log;
#[cfg(feature = "logging")]extern crate env_logger;
extern crate ansi_term;
extern crate flate2;
extern crate tar;
extern crate tempdir;
extern crate curl;
extern crate rustc_serialize;

use std::io::{BufRead, BufReader};
use std::fs::File;
use std::{env, process};

use ansi_term::Colour;

mod types;
mod tokenizer;
mod updater;
mod error;

use types::LineType;
use tokenizer::Tokenizer;
use updater::Updater;
use error::TldrError;


/// Open file, return a `BufRead` instance
fn get_file_reader(filepath: &str) -> Result<BufReader<File>, String> {
    let file = try!(
        File::open(filepath)
            .map_err(|msg| format!("Could not open file: {}", msg))
    );
    Ok(BufReader::new(file))
}


/// Print a token stream to an ANSI terminal.
fn print_lines<R>(tokenizer: &mut Tokenizer<R>) where R: BufRead {
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


#[cfg(feature = "logging")]
fn init_log() {
    env_logger::init().unwrap();
}

#[cfg(not(feature = "logging"))]
fn init_log() { }


fn main() {

    // Initialize logger
    init_log();

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

    // Print output
    print_lines(&mut tokenizer);
    println!("");

    let dl = Updater::new("https://github.com/tldr-pages/tldr/archive/master.tar.gz".into());
    let copied = dl.update().unwrap_or_else(|e| {
        match e {
            TldrError::UpdateError(msg) => println!("Could not update cache: {}", msg),
        };
        process::exit(1);
    });
    println!("Cached {} tldr pages.", copied);

}


#[cfg(test)]
mod test {
    use types::LineType;

    #[test]
    fn test_linetype_from_str() {
        assert_eq!(LineType::from(""), LineType::Empty);
        assert_eq!(LineType::from(" \n \r"), LineType::Empty);
        assert_eq!(LineType::from("# Hello there"), LineType::Title("Hello there".into()));
        assert_eq!(LineType::from("> tis a description \n"), LineType::Description("tis a description".into()));
        assert_eq!(LineType::from("- some command"), LineType::ExampleText("some command".into()));
        assert_eq!(LineType::from("`$ cargo run`"), LineType::ExampleCode("$ cargo run".into()));
        assert_eq!(LineType::from("`$ cargo run"), LineType::Other("`$ cargo run".into()));
        assert_eq!(LineType::from("jklö"), LineType::Other("jklö".into()));
    }
}
