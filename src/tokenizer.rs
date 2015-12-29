//! Code to tokenize a `BufRead` instance into an iterator of `LineType`s.

use std::io::BufRead;

use types::LineType;


#[derive(Debug)]
pub struct Tokenizer<R: BufRead> {
    reader: R,
    current_line: String,
}

impl<R> Tokenizer<R> where R: BufRead {

    pub fn new(reader: R) -> Tokenizer<R> {
        Tokenizer {
            reader: reader,
            current_line: String::new(),
        }
    }

    pub fn next(&mut self) -> Option<LineType> {
        self.current_line.clear();
        let bytes_read = self.reader.read_line(&mut self.current_line).unwrap();
        match bytes_read {
            0 => None,
            _ => Some(LineType::from(&self.current_line[..])),
        }
    }

}
