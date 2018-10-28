//! Types used in the client.

use std::fmt;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum OsType {
    Linux,
    OsX,
    SunOs,
    Other,
}

impl fmt::Display for OsType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OsType::Linux => write!(f, "Linux"),
            OsType::OsX => write!(f, "macOS / BSD"),
            OsType::SunOs => write!(f, "SunOS"),
            OsType::Other => write!(f, "Unknown OS"),
        }
    }
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
    /// Convert a string slice to a LineType. Newlines and trailing whitespace are trimmed.
    fn from(line: &'a str) -> LineType {
        let trimmed: &str = line.trim_right();
        let mut chars = trimmed.chars();
        match chars.next() {
            None => LineType::Empty,
            Some('#') => LineType::Title(
                trimmed
                    .trim_left_matches(|chr: char| chr == '#' || chr.is_whitespace())
                    .into(),
            ),
            Some('>') => LineType::Description(
                trimmed
                    .trim_left_matches(|chr: char| chr == '>' || chr.is_whitespace())
                    .into(),
            ),
            Some(' ') => LineType::ExampleCode(
                trimmed
                    .trim_left_matches(|chr: char| chr.is_whitespace())
                    .into(),
            ),
            _ => LineType::ExampleText(trimmed.into()),
        }
    }
}

impl LineType {
    /// Support for old format.
    /// TODO: Remove once old format has been phased out!
    pub fn from_v1(line: &str) -> LineType {
        let trimmed = line.trim();
        let mut chars = trimmed.chars();
        match chars.next() {
            None => LineType::Empty,
            Some('#') => LineType::Title(
                trimmed
                    .trim_left_matches(|chr: char| chr == '#' || chr.is_whitespace())
                    .into(),
            ),
            Some('>') => LineType::Description(
                trimmed
                    .trim_left_matches(|chr: char| chr == '>' || chr.is_whitespace())
                    .into(),
            ),
            Some('-') => LineType::ExampleText(
                trimmed
                    .trim_left_matches(|chr: char| chr == '-' || chr.is_whitespace())
                    .into(),
            ),
            Some('`') if chars.last() == Some('`') => LineType::ExampleCode(
                trimmed
                    .trim_matches(|chr: char| chr == '`' || chr.is_whitespace())
                    .into(),
            ),
            _ => LineType::Other(trimmed.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::LineType;

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
}
