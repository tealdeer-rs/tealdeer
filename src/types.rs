//! Types used in the client.

#[derive(Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum OsType {
    Linux,
    OsX,
    SunOS,
    Other,
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
