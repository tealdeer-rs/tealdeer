//! Types used in the client.

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum OsType {
    Linux,
    OsX,
    SunOs,
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
    /// Convert a string slice to a LineType. Newlines and trailing whitespace are trimmed.
    fn from(line: &'a str) -> LineType {
        let trimmed: &str = line.trim_right();
        let mut chars = trimmed.chars();
        match chars.next() {
            None => LineType::Empty,
            Some('#') => LineType::Title(trimmed.trim_left_matches(|chr: char| chr == '#' || chr.is_whitespace()).into()),
            Some('>') => LineType::Description(trimmed.trim_left_matches(|chr: char| chr == '>' || chr.is_whitespace()).into()),
            Some(' ') => LineType::ExampleCode(trimmed.trim_left_matches(|chr: char| chr.is_whitespace()).into()),
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
            Some('#') => LineType::Title(trimmed.trim_left_matches(|chr: char| chr == '#' || chr.is_whitespace()).into()),
            Some('>') => LineType::Description(trimmed.trim_left_matches(|chr: char| chr == '>' || chr.is_whitespace()).into()),
            Some('-') => LineType::ExampleText(trimmed.trim_left_matches(|chr: char| chr == '-' || chr.is_whitespace()).into()),
            Some('`') if chars.last() == Some('`') => LineType::ExampleCode(trimmed.trim_matches(|chr: char| chr == '`' || chr.is_whitespace()).into()),
            _ => LineType::Other(trimmed.into()),
        }
    }
}

#[cfg(test)]
mod test {
    extern crate serde_json;

    use super::OsType::{self, Linux, OsX, SunOs, Other};
    use super::LineType;
    use self::serde_json::from_str;

    #[test]
    fn test_os_type_decoding_regular() {
        assert_eq!(from_str::<OsType>("\"linux\"").unwrap(), Linux);
        assert_eq!(from_str::<OsType>("\"osx\"").unwrap(), OsX);
        assert_eq!(from_str::<OsType>("\"sunos\"").unwrap(), SunOs);
        assert_eq!(from_str::<OsType>("\"other\"").unwrap(), Other);
    }

    // REVIEW: Not sure how to do case-insensitive deserialization with serde
    //         json. Ok to ditch support for this?
    //#[test]
    //fn test_os_type_decoding_uppercase() {
    //    assert_eq!(from_str::<OsType>("\"Linux\"").unwrap(), Linux);
    //    assert_eq!(from_str::<OsType>("\"LINUX\"").unwrap(), Linux);
    //}

    #[test]
    fn test_os_type_decoding_unknown() {
        assert!(from_str::<OsType>("\"lindows\"").is_err());
    }

    #[test]
    fn test_linetype_from_str() {
        assert_eq!(LineType::from(""), LineType::Empty);
        assert_eq!(LineType::from(" \n \r"), LineType::Empty);
        assert_eq!(LineType::from("# Hello there"), LineType::Title("Hello there".into()));
        assert_eq!(LineType::from("> tis a description \n"), LineType::Description("tis a description".into()));
        assert_eq!(LineType::from("some command "), LineType::ExampleText("some command".into()));
        assert_eq!(LineType::from("    $ cargo run "), LineType::ExampleCode("$ cargo run".into()));
    }
}
