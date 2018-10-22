use std::env;
use std::fs;
use std::io::{Error as IoError, Read, Write};
use std::path::PathBuf;

use ansi_term::{Color, Style};
use toml;
use xdg::BaseDirectories;

use error::TealdeerError::{self, ConfigError};

pub const CONFIG_FILE_NAME: &str = "config.toml";

fn default_underline() -> bool {
    false
}

fn default_bold() -> bool {
    false
}

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
    #[serde(default = "default_underline")]
    pub underline: bool,
    #[serde(default = "default_bold")]
    pub bold: bool,
}

impl Default for RawStyle {
    fn default() -> RawStyle {
        RawStyle {
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
    #[serde(default)]
    pub description: RawStyle,
    #[serde(default)]
    pub command_name: RawStyle,
    #[serde(default)]
    pub example_text: RawStyle,
    #[serde(default)]
    pub example_code: RawStyle,
    #[serde(default)]
    pub example_variable: RawStyle,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct RawConfig {
    style: RawStyleConfig,
}

impl RawConfig {
    fn new() -> RawConfig {
        let mut raw_config = RawConfig::default();

        // Set default config
        raw_config.style.example_text.foreground = Some(RawColor::Green);
        raw_config.style.command_name.foreground = Some(RawColor::Cyan);
        raw_config.style.example_code.foreground = Some(RawColor::Cyan);
        raw_config.style.example_variable.foreground = Some(RawColor::Cyan);
        raw_config.style.example_variable.underline = true;

        raw_config
    }
} // impl RawConfig

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StyleConfig {
    pub description: Style,
    pub command_name: Style,
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
        Config {
            style: StyleConfig {
                command_name: raw_config.style.command_name.into(),
                description: raw_config.style.description.into(),
                example_text: raw_config.style.example_text.into(),
                example_code: raw_config.style.example_code.into(),
                example_variable: raw_config.style.example_variable.into(),
            },
        }
    }
}

fn map_io_err_to_config_err(e: IoError) -> TealdeerError {
    ConfigError(format!("Io Error: {}", e))
}

impl Config {
    pub fn load() -> Result<Config, TealdeerError> {
        debug!("Loading config");

        // Determine path
        let config_file_path = get_config_path()
            .map_err(|e| ConfigError(format!("Could not determine config path: {}", e)))?;

        // Load config
        let raw_config: RawConfig = if config_file_path.exists() && config_file_path.is_file() {
            let mut config_file =
                fs::File::open(config_file_path).map_err(map_io_err_to_config_err)?;
            let mut contents = String::new();
            let _ = config_file
                .read_to_string(&mut contents)
                .map_err(map_io_err_to_config_err)?;
            toml::from_str(&contents)
                .map_err(|err| ConfigError(format!("Failed to parse config file: {}", err)))?
        } else {
            RawConfig::new()
        };

        Ok(Config::from(raw_config))
    }
} // impl Config

/// Return the path to the config directory.
///
/// The config dir path can be overridden using the `TEALDEER_CONFIG_DIR` env
/// variable. Otherwise, `$XDG_CONFIG_hOME/tealdeer` is returned.
///
/// Note that this function does not verify whether the directory at that
/// loation exists, or is a directory.
pub fn get_config_dir() -> Result<PathBuf, TealdeerError> {
    // Allow overriding the config directory by setting the
    // $TEALDEER_CONFIG_DIR env variable.
    if let Ok(value) = env::var("TEALDEER_CONFIG_DIR") {
        return Ok(PathBuf::from(value));
    };

    // Otherwise, fall back to $XDG_CONFIG_HOME/tealdeer.
    let xdg_dirs = match BaseDirectories::with_prefix(::NAME) {
        Ok(dirs) => dirs,
        Err(_) => {
            return Err(ConfigError("Could not determine XDG base directory.".into()))
        }
    };
    Ok(xdg_dirs.get_config_home())
}

/// Return the path to the config file.
///
/// Note that this function does not verify whether the file at that location
/// exists, or is a file.
pub fn get_config_path() -> Result<PathBuf, TealdeerError> {
    let config_dir = get_config_dir()?;
    let config_file_path = config_dir.join(CONFIG_FILE_NAME);
    Ok(config_file_path)
}

/// Create default config file.
pub fn make_default_config() -> Result<PathBuf, TealdeerError> {
    let config_dir = get_config_dir()?;

    // Ensure that config directory exists
    if !config_dir.exists() {
        if let Err(e) = fs::create_dir_all(&config_dir) {
            return Err(ConfigError(format!(
                "Could not create config directory: {}",
                e
            )));
        }
    } else if !config_dir.is_dir() {
        return Err(ConfigError(format!(
            "Config directory could not be created: {} already exists but is not a directory",
            config_dir.to_string_lossy(),
        )));
    }

    // Ensure that a config file doesn't get overwritten
    let config_file_path = config_dir.join(CONFIG_FILE_NAME);
    if config_file_path.is_file() {
        return Err(ConfigError(format!(
            "A configuration file already exists at {}, no action was taken.",
            config_file_path.to_str().unwrap()
        )));
    }

    // Create default config
    let serialized_config = toml::to_string(&RawConfig::new())
        .map_err(|err| ConfigError(format!("Failed to serialize default config: {}", err)))?;

    // Write default config
    let mut config_file = fs::File::create(&config_file_path).map_err(map_io_err_to_config_err)?;
    let _wc = config_file
        .write(serialized_config.as_bytes())
        .map_err(map_io_err_to_config_err)?;

    Ok(config_file_path)
}

#[test]
fn test_serialize_deserialize() {
    let raw_config = RawConfig::new();
    let serialized = toml::to_string(&raw_config).unwrap();
    let deserialized: RawConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(raw_config, deserialized);
}
