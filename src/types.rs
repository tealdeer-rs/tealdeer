//! Shared types used in tealdeer.

use std::convert::TryFrom;
use std::fmt;
use std::fs::File;
use std::io::Read;

use crate::PageLookupResult;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum OsType {
    Linux,
    OsX,
    SunOs,
    Windows,
    Other,
}

impl fmt::Display for OsType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Linux => write!(f, "Linux"),
            Self::OsX => write!(f, "macOS / BSD"),
            Self::SunOs => write!(f, "SunOS"),
            Self::Windows => write!(f, "Windows"),
            Self::Other => write!(f, "Unknown OS"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorOptions {
    Always,
    Auto,
    Never,
}

#[derive(Debug, Eq, PartialEq)]
pub enum LineType {
    Empty,
    Title(String),
    Description(String),
    ExampleText(String),
    ExampleCode(String),
    Other(String),
}

impl<'a> From<&'a str> for LineType {
    /// Convert a string slice to a `LineType`. Newlines and trailing whitespace are trimmed.
    fn from(line: &'a str) -> Self {
        let trimmed: &str = line.trim_end();
        let mut chars = trimmed.chars();
        match chars.next() {
            None => Self::Empty,
            Some('#') => Self::Title(
                trimmed
                    .trim_start_matches(|chr: char| chr == '#' || chr.is_whitespace())
                    .into(),
            ),
            Some('>') => Self::Description(
                trimmed
                    .trim_start_matches(|chr: char| chr == '>' || chr.is_whitespace())
                    .into(),
            ),
            Some(' ') => Self::ExampleCode(trimmed.trim_start_matches(char::is_whitespace).into()),
            Some(_) => Self::ExampleText(trimmed.into()),
        }
    }
}

impl LineType {
    /// Support for old format.
    /// TODO: Remove once old format has been phased out!
    pub fn from_v1(line: &str) -> Self {
        let trimmed = line.trim();
        let mut chars = trimmed.chars();
        match chars.next() {
            None => Self::Empty,
            Some('#') => Self::Title(
                trimmed
                    .trim_start_matches(|chr: char| chr == '#' || chr.is_whitespace())
                    .into(),
            ),
            Some('>') => Self::Description(
                trimmed
                    .trim_start_matches(|chr: char| chr == '>' || chr.is_whitespace())
                    .into(),
            ),
            Some('-') => Self::ExampleText(
                trimmed
                    .trim_start_matches(|chr: char| chr == '-' || chr.is_whitespace())
                    .into(),
            ),
            Some('`') if chars.last() == Some('`') => Self::ExampleCode(
                trimmed
                    .trim_matches(|chr: char| chr == '`' || chr.is_whitespace())
                    .into(),
            ),
            Some(_) => Self::Other(trimmed.into()),
        }
    }
}

/// The reason why a certain path (e.g. config path or cache dir) was chosen.
#[derive(Debug, PartialEq)]
pub enum PathSource {
    /// OS convention (e.g. XDG on Linux)
    OsConvention,
    /// Env variable (TEALDEER_*)
    EnvVar,

    #[allow(dead_code)] // Waiting for Pull Request #141
    /// Config file variable
    ConfigVar,
}

impl fmt::Display for PathSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::OsConvention => "OS convention",
                Self::EnvVar => "env variable",
                Self::ConfigVar => "config file variable",
            }
        )
    }
}

/// This struct implements the Read Trait such that the `files` will be read in order as if they
/// were 1 contiguous File.
/// Within Tealdeer this is meant to cohesively interact with the `PageLookupResult` in order to
/// process pages / patches as if they were 1 file.
pub struct SeqFileReader {
    files: Vec<File>,
    curr_file_idx: usize,
}

impl SeqFileReader {
    fn new(files: Vec<File>) -> Self {
        Self {
            files,
            curr_file_idx: 0,
        }
    }
}

impl TryFrom<PageLookupResult> for SeqFileReader {
    type Error = std::io::Error;
    fn try_from(lookup_result: PageLookupResult) -> std::io::Result<SeqFileReader> {
        let files = lookup_result
            .paths()
            .map(File::open)
            .collect::<std::io::Result<Vec<File>>>()?;
        Ok(Self::new(files))
    }
}

impl Read for SeqFileReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Base Case: No more files to read.
        if self.curr_file_idx >= self.files.len() {
            log::info!("All files have been exhausted to EOF");
            return Ok(0);
        }

        let bytes_read = self.files[self.curr_file_idx].read(buf)?;

        if bytes_read != 0 {
            // Successful read, not EOF
            return Ok(bytes_read);
        } else if buf.is_empty() {
            // Buffer is full. Cannot be filled.
            return Ok(0);
        }

        // Current file reached EOF, go to next file
        self.curr_file_idx += 1;
        self.read(buf)
    }
}

#[cfg(test)]
mod test {
    use super::LineType;
    use super::SeqFileReader;
    use std::fs::File;
    use std::io::{Read, Write};

    #[test]
    fn test_linetype_from_str() {
        assert_eq!(LineType::from(""), LineType::Empty);
        assert_eq!(LineType::from(" \n \r"), LineType::Empty);
        assert_eq!(
            LineType::from("# Hello there"),
            LineType::Title("Hello there".into())
        );
        assert_eq!(
            LineType::from("> tis a description \n"),
            LineType::Description("tis a description".into())
        );
        assert_eq!(
            LineType::from("some command "),
            LineType::ExampleText("some command".into())
        );
        assert_eq!(
            LineType::from("    $ cargo run "),
            LineType::ExampleCode("$ cargo run".into())
        );
    }

    #[test]
    fn test_seq_file_reader() {
        let dir = tempfile::tempdir().unwrap();
        {
            let mut f1 = File::create(dir.path().join("testfile1")).unwrap();
            f1.write_all(b"Hello\n").unwrap();
            let mut f2 = File::create(dir.path().join("testfile2")).unwrap();
            f2.write_all(b"World").unwrap();
        }
        let f1 = File::open(dir.path().join("testfile1")).unwrap();
        let f2 = File::open(dir.path().join("testfile2")).unwrap();

        let mut sfr = SeqFileReader {
            files: vec![f1, f2],
            curr_file_idx: 0,
        };

        let mut buf = Vec::new();
        let bytes = sfr.read_to_end(&mut buf).unwrap();
        assert_eq!(&buf, b"Hello\nWorld");
        assert_eq!(bytes, 11);
    }
}
