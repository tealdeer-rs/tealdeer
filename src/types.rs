//! Types used in the client.

use rustc_serialize::{Decodable, Decoder};


#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum OsType {
    Linux,
    OsX,
    SunOs,
    Other,
}


/// Custom Decodable implementation, so that we can parse command line arguments
/// directly into an `OsType` instance.
impl Decodable for OsType {
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, D::Error> {
        d.read_str().and_then(|input| {
            let lowercase = input.to_lowercase();
            match &lowercase[..] {
                "linux" => Ok(OsType::Linux),
                "osx" => Ok(OsType::OsX),
                "sunos" => Ok(OsType::SunOs),
                "other" => Ok(OsType::Other),
                _ => Err(d.error(&format!("Invalid OS type: '{}'. Choose one of 'linux', \
                                           'osx', 'sunos' or 'other'.", lowercase)))
            }
        })
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

#[cfg(test)]
mod test {
    extern crate rustc_serialize;
    extern crate docopt;

    use super::OsType::{self, Linux, OsX, SunOs, Other};
    use rustc_serialize::json;

    #[test]
    fn test_os_type_decoding_regular() {
        assert_eq!(json::decode::<OsType>("\"linux\"").unwrap(), Linux);
        assert_eq!(json::decode::<OsType>("\"osx\"").unwrap(), OsX);
        assert_eq!(json::decode::<OsType>("\"sunos\"").unwrap(), SunOs);
        assert_eq!(json::decode::<OsType>("\"other\"").unwrap(), Other);
    }

    #[test]
    fn test_os_type_decoding_uppercase() {
        assert_eq!(json::decode::<OsType>("\"Linux\"").unwrap(), Linux);
        assert_eq!(json::decode::<OsType>("\"LINUX\"").unwrap(), Linux);
    }
#[test]
    fn test_os_type_decoding_unknown() {
        assert!(json::decode::<OsType>("\"lindows\"").is_err());
    }
}
