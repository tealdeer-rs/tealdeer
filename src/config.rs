use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use ansi_term::{Colour, Style};
use toml;
use xdg::BaseDirectories;

use error::TealdeerError::{self, ConfigError};

pub const SYNTAX_CONFIG_FILE_NAME: &'static str = "syntax.toml";

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RawColour {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
}

impl From<RawColour> for Colour {
    fn from(raw_colour: RawColour) -> Colour {
        match raw_colour {
            RawColour::Black => Colour::Black,
            RawColour::Red => Colour::Red,
            RawColour::Green => Colour::Green,
            RawColour::Yellow => Colour::Yellow,
            RawColour::Blue => Colour::Blue,
            RawColour::Purple => Colour::Purple,
            RawColour::Cyan => Colour::Cyan,
            RawColour::White => Colour::White,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct RawStyle {
    pub foreground: Option<RawColour>,
    pub background: Option<RawColour>,
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
            style = style.fg(Colour::from(foreground));
        }

        if let Some(background) = raw_style.background {
            style = style.on(Colour::from(background));
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

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct RawConfig {
    pub highlight_style: RawStyle,
    pub description_style: RawStyle,
    pub example_text_style: RawStyle,
    pub example_code_style: RawStyle,
    pub example_variable_style: RawStyle,
}

impl RawConfig {
    fn new() -> RawConfig {
        let mut raw_config = RawConfig::default();

        raw_config.highlight_style.foreground = Some(RawColour::Red);
        raw_config.example_variable_style.underline = true;

        raw_config
    }
} // impl RawConfig

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    pub highlight_style: Style,
    pub description_style: Style,
    pub example_text_style: Style,
    pub example_code_style: Style,
    pub example_variable_style: Style,
}

impl From<RawConfig> for Config {
    fn from(raw_config: RawConfig) -> Config {
        Config{
            highlight_style: Style::from(raw_config.highlight_style),
            description_style: Style::from(raw_config.description_style),
            example_text_style: Style::from(raw_config.example_text_style),
            example_code_style: Style::from(raw_config.example_code_style),
            example_variable_style: Style::from(raw_config.example_variable_style),
        }
    }
}

impl Config {
    pub fn new() -> Result<Config, TealdeerError> {
        let raw_config = match get_syntax_config_path() {
            Ok(syntax_config_file_path) => {
                let mut syntax_config_file = fs::File::open(syntax_config_file_path)?;
                let mut contents = String::new();
                let _rc = syntax_config_file.read_to_string(&mut contents)?;

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
    let mut syntax_config_file = fs::File::create(&syntax_config_file_path)?;
    let _wc = syntax_config_file.write(serialized_syntax_config.as_bytes())?;

    Ok(syntax_config_file_path)
}

#[test]
fn test_serialize_deserialize() {
    let raw_config = RawConfig::new();
    let serialized = toml::to_string(&raw_config).unwrap();
    let deserialized: RawConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(raw_config, deserialized);
}
