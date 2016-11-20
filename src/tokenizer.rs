//! Code to tokenize a `BufRead` instance into an iterator of `LineType`s.

use std::io::BufRead;

use types::LineType;

/// A tokenizer is initialized with a BufReader instance that contains the
/// entire Tldr page. It then returns tokens as `Option<LineType>`.
#[derive(Debug)]
pub struct Tokenizer<R: BufRead> {
    /// An instance of `R: BufRead`.
    reader: R,
    /// Whether the first line has already been tokenized or not.
    first_line: bool,
    /// Buffer for the current line. Used internally.
    current_line: String,
}

impl<R> Tokenizer<R> where R: BufRead {
    pub fn new(reader: R) -> Tokenizer<R> {
        Tokenizer {
            reader: reader,
            first_line: true,
            current_line: String::new(),
        }
    }

    pub fn next_token(&mut self) -> Option<LineType> {
        self.current_line.clear();
        let bytes_read = self.reader.read_line(&mut self.current_line);
        match bytes_read {
            Ok(0) => None,
            Err(e) => {
                warn!("Could not read line from token reader: {:?}", e);
                None
            },
            Ok(_) => {
                // Handle new titles
                if self.first_line && !self.current_line.starts_with("#") {
                    // It's the new format! Drop next line.
                    // (Hmm, is there a way to do this without an allocation?)
                    let mut devnull = String::new();
                    if let Err(e) = self.reader.read_line(&mut devnull) {
                        warn!("Could not read line from token reader: {:?}", e);
                        return None;
                    }
                    self.first_line = false;
                    return Some(LineType::Title(self.current_line.trim_right().to_string()));
                }

                // Clear `first_line` flag
                self.first_line = false;

                // Convert line to a `LineType` instance
                Some(LineType::from(&self.current_line[..]))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Tokenizer;
    use types::LineType;

    #[test]
    fn test_first_line_old_format() {
        let input = "# The Title\n\n";
        let mut tokenizer = Tokenizer::new(input.as_bytes());
        let title = tokenizer.next_token().unwrap();
        assert_eq!(title, LineType::Title("The Title".to_string()));
        let empty = tokenizer.next_token().unwrap();
        assert_eq!(empty, LineType::Empty);
    }

    #[test]
    fn test_first_line_new_format() {
        let input = "The Title\n=========\n\n";
        let mut tokenizer = Tokenizer::new(input.as_bytes());
        let title = tokenizer.next_token().unwrap();
        assert_eq!(title, LineType::Title("The Title".to_string()));
        let empty = tokenizer.next_token().unwrap();
        assert_eq!(empty, LineType::Empty);
    }
}
