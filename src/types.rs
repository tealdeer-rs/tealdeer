//! Shared types used in tealdeer.

use std::{fmt, str};

use serde::Deserialize;

/// The platform types supported by tldr.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(dead_code)]
pub enum PlatformType {
    Linux,
    OsX,
    SunOs,
    Windows,
}

impl fmt::Display for PlatformType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Linux => write!(f, "Linux"),
            Self::OsX => write!(f, "macOS / BSD"),
            Self::SunOs => write!(f, "SunOS"),
            Self::Windows => write!(f, "Windows"),
        }
    }
}

/// The platform lookup strategy.
///
/// Includes both the platform type, as well as
#[derive(Debug, Copy, Clone)]
pub struct PlatformStrategy {
    /// The platform type that should be looked up.
    pub platform_type: PlatformType,
    /// Flag indicating whether all pages should be listed or not. This is only
    /// used when the special platform type `all` is specified by the user.
    pub list_all: bool,
}

impl PlatformStrategy {
    pub fn new(platform_type: PlatformType) -> Self {
        Self {
            platform_type,
            list_all: false,
        }
    }

    /// Return a `PlatformStrategy` containing the current platform as the
    /// target platform type.
    pub fn current() -> Self {
        Self {
            platform_type: PlatformType::current(),
            list_all: false,
        }
    }

    /// Like `current()`, but when listing the pages, return the pages for all
    /// platforms, not just for the current platform.
    pub fn all() -> Self {
        Self {
            platform_type: PlatformType::current(),
            list_all: true,
        }
    }
}

impl str::FromStr for PlatformStrategy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "linux" => Ok(PlatformStrategy::new(PlatformType::Linux)),
            "osx" | "macos" => Ok(PlatformStrategy::new(PlatformType::OsX)),
            "windows" => Ok(PlatformStrategy::new(PlatformType::Windows)),
            "sunos" => Ok(PlatformStrategy::new(PlatformType::SunOs)),
            "current" => Ok(PlatformStrategy::current()),
            "all" => Ok(PlatformStrategy::all()),
            other => Err(format!(
                "Unknown platform: {}. Possible values: linux, macos, osx, windows, sunos, current, all",
                other
            )),
        }
    }
}

impl PlatformType {
    #[cfg(target_os = "linux")]
    pub fn current() -> Self {
        Self::Linux
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    pub fn current() -> Self {
        Self::OsX
    }

    #[cfg(target_os = "windows")]
    pub fn current() -> Self {
        Self::Windows
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "macos",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly",
        target_os = "windows"
    )))]
    pub fn current() -> Self {
        Self::Other
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorOptions {
    Always,
    Auto,
    Never,
}

impl str::FromStr for ColorOptions {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "always" => Ok(Self::Always),
            "auto" => Ok(Self::Auto),
            "never" => Ok(Self::Never),
            other => Err(format!(
                "Unknown color option: {}. Possible values: always, auto, never",
                other
            )),
        }
    }
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
