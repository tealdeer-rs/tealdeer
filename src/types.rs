//! Shared types used in tealdeer.

use std::{fmt, str};

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum PlatformType {
    Linux,
    OsX,
    Windows,
    SunOs,
    Android,
    FreeBsd,
    NetBsd,
    OpenBsd,
    Common,
}

impl fmt::Display for PlatformType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Linux => write!(f, "Linux"),
            Self::OsX => write!(f, "macOS / BSD"),
            Self::Windows => write!(f, "Windows"),
            Self::SunOs => write!(f, "SunOS"),
            Self::Android => write!(f, "Android"),
            Self::FreeBsd => write!(f, "FreeBSD"),
            Self::NetBsd => write!(f, "NetBSD"),
            Self::OpenBsd => write!(f, "OpenBSD"),
            Self::Common => write!(f, "Common"),
        }
    }
}

impl clap::ValueEnum for PlatformType {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Self::Linux,
            Self::OsX,
            Self::SunOs,
            Self::Windows,
            Self::Android,
            Self::FreeBsd,
            Self::NetBsd,
            Self::OpenBsd,
            Self::Common,
        ]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::Linux => Some(clap::builder::PossibleValue::new("linux")),
            Self::OsX => Some(clap::builder::PossibleValue::new("macos").alias("osx")),
            Self::Windows => Some(clap::builder::PossibleValue::new("windows")),
            Self::SunOs => Some(clap::builder::PossibleValue::new("sunos")),
            Self::Android => Some(clap::builder::PossibleValue::new("android")),
            Self::FreeBsd => Some(clap::builder::PossibleValue::new("freebsd")),
            Self::NetBsd => Some(clap::builder::PossibleValue::new("netbsd")),
            Self::OpenBsd => Some(clap::builder::PossibleValue::new("openbsd")),
            Self::Common => Some(clap::builder::PossibleValue::new("common")),
        }
    }
}

impl PlatformType {
    #[cfg(target_os = "linux")]
    pub fn current() -> Self {
        Self::Linux
    }

    #[cfg(any(target_os = "macos", target_os = "dragonfly"))]
    pub fn current() -> Self {
        Self::OsX
    }

    #[cfg(target_os = "windows")]
    pub fn current() -> Self {
        Self::Windows
    }

    #[cfg(target_os = "android")]
    pub fn current() -> Self {
        Self::Android
    }

    #[cfg(target_os = "freebsd")]
    pub fn current() -> Self {
        Self::FreeBsd
    }

    #[cfg(target_os = "netbsd")]
    pub fn current() -> Self {
        Self::NetBsd
    }

    #[cfg(target_os = "openbsd")]
    pub fn current() -> Self {
        Self::OpenBsd
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly",
        target_os = "windows",
        target_os = "android",
    )))]
    pub fn current() -> Self {
        Self::Other
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ColorOptions {
    Always,
    Auto,
    Never,
}

impl Default for ColorOptions {
    fn default() -> Self {
        Self::Auto
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
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PathSource {
    /// OS convention (e.g. XDG on Linux)
    OsConvention,
    /// Env variable (TEALDEER_*)
    EnvVar,
    /// Config file
    ConfigFile,
    /// CLI argument override
    Cli,
}

impl fmt::Display for PathSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::OsConvention => "OS convention",
                Self::EnvVar => "env variable",
                Self::ConfigFile => "config file",
                Self::Cli => "command line argument",
            }
        )
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
