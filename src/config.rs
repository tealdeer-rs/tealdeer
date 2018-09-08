use std::env;
use std::fs;
use std::io::{Error as IoError, Read, Write};
use std::path::PathBuf;

use ansi_term::{Color, Style};
use toml;
use xdg::BaseDirectories;

use error::TealdeerError::{self, ConfigError};

pub const SYNTAX_CONFIG_FILE_NAME: &'static str = "config.toml";

#[serde(rename_all = "lowercase")]
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RawColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
}

impl From<RawColor> for Color {
    fn from(raw_color: RawColor) -> Color {
        match raw_color {
            RawColor::Black => Color::Black,
            RawColor::Red => Color::Red,
            RawColor::Green => Color::Green,
            RawColor::Yellow => Color::Yellow,
            RawColor::Blue => Color::Blue,
            RawColor::Purple => Color::Purple,
            RawColor::Cyan => Color::Cyan,
            RawColor::White => Color::White,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct RawStyle {
    pub foreground: Option<RawColor>,
    pub background: Option<RawColor>,
    pub underline: bool,
    pub bold: bool,
}

impl Default for RawStyle {
    fn default() -> RawStyle {
        RawStyle{
            foreground: None,
            background: None,
            underline: false,
            bold: false,
        }
    }
} // impl RawStyle

impl From<RawStyle> for Style {
    fn from(raw_style: RawStyle) -> Style {
        let mut style = Style::default();

        if let Some(foreground) = raw_style.foreground {
            style = style.fg(Color::from(foreground));
        }

        if let Some(background) = raw_style.background {
            style = style.on(Color::from(background));
        }

        if raw_style.underline {
            style = style.underline();
        }

        if raw_style.bold {
            style = style.bold();
        }

        style
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct RawStyleConfig {
    pub highlight: RawStyle,
    pub description: RawStyle,
    pub example_text: RawStyle,
    pub example_code: RawStyle,
    pub example_variable: RawStyle,
}


#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct RawConfig {
    style: RawStyleConfig,
}

impl RawConfig {
    fn new() -> RawConfig {
        let mut raw_config = RawConfig::default();

        raw_config.style.highlight.foreground = Some(RawColor::Red);
        raw_config.style.example_variable.underline = true;

        raw_config
    }
} // impl RawConfig

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StyleConfig {
    pub highlight: Style,
    pub description: Style,
    pub example_text: Style,
    pub example_code: Style,
    pub example_variable: Style,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    pub style: StyleConfig,
}

impl From<RawConfig> for Config {
    fn from(raw_config: RawConfig) -> Config {
        Config{
            style: StyleConfig{
                highlight: raw_config.style.highlight.into(),
                description: raw_config.style.description.into(),
                example_text: raw_config.style.example_text.into(),
                example_code: raw_config.style.example_code.into(),
                example_variable: raw_config.style.example_variable.into(),
            }
        }
    }
}

fn map_io_err_to_config_err(e: IoError) -> TealdeerError {
    ConfigError(format!("Io Error: {}", e))
}

impl Config {
    pub fn load() -> Result<Config, TealdeerError> {
        let raw_config = match get_syntax_config_path() {
            Ok(syntax_config_file_path) => {
                let mut syntax_config_file = fs::File::open(syntax_config_file_path)
                    .map_err(map_io_err_to_config_err)?;
                let mut contents = String::new();
                let _rc = syntax_config_file.read_to_string(&mut contents)
                    .map_err(map_io_err_to_config_err)?;

                toml::from_str(&contents).map_err(|err| ConfigError(format!("Failed to parse syntax config file: {}", err)))?
            }
            Err(ConfigError(_)) => RawConfig::new(),
            Err(_) => {
                return Err(ConfigError("Unknown error while looking up syntax config path".into()));
            }
        };

        Ok(Config::from(raw_config))
    }
} // impl Config

/// Return the path to the config directory.
pub fn get_config_dir() -> Result<PathBuf, TealdeerError> {
    // Allow overriding the config directory by setting the
    // $TEALDEER_CONFIG_DIR env variable.
    if let Ok(value) = env::var("TEALDEER_CONFIG_DIR") {
        let path = PathBuf::from(value);

        if path.exists() && path.is_dir() {
            return Ok(path)
        } else {
            return Err(ConfigError(
                "Path specified by $TEALDEER_CONFIG_DIR \
                 does not exist or is not a directory.".into()
            ));
        }
    };

    // Otherwise, fall back to $XDG_CONFIG_HOME/tealdeer.
    let xdg_dirs = match BaseDirectories::with_prefix(::NAME) {
        Ok(dirs) => dirs,
        Err(_) => return Err(ConfigError("Could not determine XDG base directory.".into())),
    };
    Ok(xdg_dirs.get_config_home())
}

/// Return the path to the syntax config file.
pub fn get_syntax_config_path() -> Result<PathBuf, TealdeerError> {
    let config_dir = get_config_dir()?;
    let syntax_config_file_path = config_dir.join(SYNTAX_CONFIG_FILE_NAME);

    if syntax_config_file_path.is_file() {
        Ok(syntax_config_file_path)
    } else {
        Err(ConfigError(format!("{} is not a file path", syntax_config_file_path.to_str().unwrap())))
    }
}

/// Create default syntax config file.
pub fn make_default_syntax_config() -> Result<PathBuf, TealdeerError> {
    let config_dir = get_config_dir()?;
    if !config_dir.is_dir() {
        if let Err(e) = fs::create_dir_all(&config_dir) {
            return Err(ConfigError(format!("Could not create config directory: {}", e)));
        }
    }

    let serialized_syntax_config = toml::to_string(&RawConfig::new())
        .map_err(|err| ConfigError(format!("Failed to serialize default syntax config: {}", err)))?;

    let syntax_config_file_path = config_dir.join(SYNTAX_CONFIG_FILE_NAME);
    let mut syntax_config_file = fs::File::create(&syntax_config_file_path)
        .map_err(map_io_err_to_config_err)?;
    let _wc = syntax_config_file.write(serialized_syntax_config.as_bytes())
        .map_err(map_io_err_to_config_err)?;

    Ok(syntax_config_file_path)
}

#[test]
fn test_serialize_deserialize() {
    let raw_config = RawConfig::new();
    let serialized = toml::to_string(&raw_config).unwrap();
    let deserialized: RawConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(raw_config, deserialized);
}
