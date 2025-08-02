use std::{
    env, fmt,
    fs::{self, File},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use anyhow::{anyhow, bail, ensure, Context, Result};
use app_dirs::{get_app_root, AppDataType};
use serde::Serialize as _;
use serde_derive::{Deserialize, Serialize};
use yansi::{Color, Style};

use crate::{extensions::Dedup as _, types::PathSource};

pub const CONFIG_FILE_NAME: &str = "config.toml";
pub const MAX_CACHE_AGE: Duration = Duration::from_secs(2_592_000); // 30 days
const DEFAULT_UPDATE_INTERVAL_HOURS: u64 = MAX_CACHE_AGE.as_secs() / 3600; // 30 days
const SUPPORTED_TLS_BACKENDS: &[RawTlsBackend] = &[
    #[cfg(feature = "native-tls")]
    RawTlsBackend::NativeTls,
    #[cfg(feature = "rustls-with-webpki-roots")]
    RawTlsBackend::RustlsWithWebpkiRoots,
    #[cfg(feature = "rustls-with-native-roots")]
    RawTlsBackend::RustlsWithNativeRoots,
];

fn default_underline() -> bool {
    false
}

fn default_bold() -> bool {
    false
}

fn default_italic() -> bool {
    false
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RawColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Purple, // Backwards compatibility with ansi_term (until tealdeer 1.5.0)
    Cyan,
    White,
    Ansi(u8),
    Rgb { r: u8, g: u8, b: u8 },
}

impl From<RawColor> for Color {
    fn from(raw_color: RawColor) -> Self {
        match raw_color {
            RawColor::Black => Self::Black,
            RawColor::Red => Self::Red,
            RawColor::Green => Self::Green,
            RawColor::Yellow => Self::Yellow,
            RawColor::Blue => Self::Blue,
            RawColor::Magenta | RawColor::Purple => Self::Magenta,
            RawColor::Cyan => Self::Cyan,
            RawColor::White => Self::White,
            RawColor::Ansi(num) => Self::Fixed(num),
            RawColor::Rgb { r, g, b } => Self::Rgb(r, g, b),
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
    #[serde(default = "default_italic")]
    pub italic: bool,
}

#[allow(clippy::derivable_impls)] // Explicitly control defaults
impl Default for RawStyle {
    fn default() -> Self {
        Self {
            foreground: None,
            background: None,
            underline: false,
            bold: false,
            italic: false,
        }
    }
}

impl From<RawStyle> for Style {
    fn from(raw_style: RawStyle) -> Self {
        let mut style = Self::default();

        if let Some(foreground) = raw_style.foreground {
            style = style.fg(Color::from(foreground));
        }

        if let Some(background) = raw_style.background {
            style = style.bg(Color::from(background));
        }

        if raw_style.underline {
            style = style.underline();
        }

        if raw_style.bold {
            style = style.bold();
        }

        if raw_style.italic {
            style = style.italic();
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

impl From<&RawStyleConfig> for StyleConfig {
    fn from(raw_style_config: &RawStyleConfig) -> Self {
        Self {
            command_name: raw_style_config.command_name.into(),
            description: raw_style_config.description.into(),
            example_text: raw_style_config.example_text.into(),
            example_code: raw_style_config.example_code.into(),
            example_variable: raw_style_config.example_variable.into(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct RawDisplayConfig {
    #[serde(default)]
    pub compact: bool,
    #[serde(default)]
    pub use_pager: bool,
}

impl From<&RawDisplayConfig> for DisplayConfig {
    fn from(raw_display_config: &RawDisplayConfig) -> Self {
        Self {
            compact: raw_display_config.compact,
            use_pager: raw_display_config.use_pager,
        }
    }
}

/// Serde doesn't support default values yet (tracking issue:
/// <https://github.com/serde-rs/serde/issues/368>), so we need to wrap
/// `DEFAULT_UPDATE_INTERVAL_HOURS` in a function to be able to use
/// `#[serde(default = ...)]`
const fn default_auto_update_interval_hours() -> u64 {
    DEFAULT_UPDATE_INTERVAL_HOURS
}

fn default_archive_source() -> String {
    "https://github.com/tldr-pages/tldr/releases/latest/download/".to_owned()
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct RawUpdatesConfig {
    #[serde(default)]
    pub auto_update: bool,
    #[serde(default = "default_auto_update_interval_hours")]
    pub auto_update_interval_hours: u64,
    #[serde(default = "default_archive_source")]
    pub archive_source: String,
    #[serde(default)]
    pub tls_backend: RawTlsBackend,
}

impl Default for RawUpdatesConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            auto_update_interval_hours: DEFAULT_UPDATE_INTERVAL_HOURS,
            archive_source: default_archive_source(),
            tls_backend: RawTlsBackend::default(),
        }
    }
}

impl<'a> TryFrom<&'a RawUpdatesConfig> for UpdatesConfig<'a> {
    type Error = anyhow::Error;

    fn try_from(raw_updates_config: &'a RawUpdatesConfig) -> Result<Self> {
        let tls_backend = match raw_updates_config.tls_backend {
            #[cfg(feature = "native-tls")]
            RawTlsBackend::NativeTls => TlsBackend::NativeTls,
            #[cfg(feature = "rustls-with-webpki-roots")]
            RawTlsBackend::RustlsWithWebpkiRoots => TlsBackend::RustlsWithWebpkiRoots,
            #[cfg(feature = "rustls-with-native-roots")]
            RawTlsBackend::RustlsWithNativeRoots => TlsBackend::RustlsWithNativeRoots,
            // when compiling without all TLS backend features, we want to handle config error.
            #[allow(unreachable_patterns)]
            _ => return Err(anyhow!(
                "Unsupported TLS backend: {}. This tealdeer build has support for the following options: {}",
                raw_updates_config.tls_backend,
                SUPPORTED_TLS_BACKENDS.iter().map(std::string::ToString::to_string).collect::<Vec<String>>().join(", ")
            ))
        };

        Ok(Self {
            auto_update: raw_updates_config.auto_update,
            auto_update_interval: Duration::from_secs(
                raw_updates_config.auto_update_interval_hours * 3600,
            ),
            archive_source: &raw_updates_config.archive_source,
            tls_backend,
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
struct RawDirectoriesConfig {
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
    #[serde(default)]
    pub custom_pages_dir: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
struct RawConfig {
    style: RawStyleConfig,
    display: RawDisplayConfig,
    updates: RawUpdatesConfig,
    directories: RawDirectoriesConfig,
}

impl Default for RawConfig {
    fn default() -> Self {
        let mut raw_config = RawConfig {
            style: RawStyleConfig::default(),
            display: RawDisplayConfig::default(),
            updates: RawUpdatesConfig::default(),
            directories: RawDirectoriesConfig::default(),
        };

        // Set default config
        raw_config.style.example_text.foreground = Some(RawColor::Green);
        raw_config.style.command_name.foreground = Some(RawColor::Cyan);
        raw_config.style.example_code.foreground = Some(RawColor::Cyan);
        raw_config.style.example_variable.foreground = Some(RawColor::Cyan);
        raw_config.style.example_variable.underline = true;

        raw_config
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct StyleConfig {
    pub description: Style,
    pub command_name: Style,
    pub example_text: Style,
    pub example_code: Style,
    pub example_variable: Style,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct DisplayConfig {
    pub compact: bool,
    pub use_pager: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdatesConfig<'a> {
    pub auto_update: bool,
    pub auto_update_interval: Duration,
    pub archive_source: &'a str,
    pub tls_backend: TlsBackend,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathWithSource {
    pub path: PathBuf,
    pub source: PathSource,
}

impl PathWithSource {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for PathWithSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.path.display(), self.source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirectoriesConfig {
    pub cache_dir: PathWithSource,
    pub custom_pages_dir: Option<PathWithSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Language<'a>(pub &'a str);

fn get_languages<'a>(
    env_lang: Option<&'a str>,
    env_language: Option<&'a str>,
) -> Vec<Language<'a>> {
    // Language list according to
    // https://github.com/tldr-pages/tldr/blob/main/CLIENT-SPECIFICATION.md#language

    let Some(env_lang) = env_lang else {
        return vec![Language("en")];
    };

    // Create an iterator that contains $LANGUAGE (':' separated list) followed by $LANG (single language)
    let locales = env_language.unwrap_or("").split(':').chain([env_lang]);

    let mut lang_list = Vec::new();
    for locale in locales {
        // Language plus country code (e.g. `en_US`)
        if locale.len() >= 5 && locale.chars().nth(2) == Some('_') {
            lang_list.push(Language(&locale[..5]));
        }
        // Language code only (e.g. `en`)
        if locale.len() >= 2 && locale != "POSIX" {
            lang_list.push(Language(&locale[..2]));
        }
    }

    lang_list.push(Language("en"));
    lang_list.clear_duplicates();
    lang_list
}

pub fn get_languages_from_env<'a>() -> Vec<Language<'a>> {
    static LANG: LazyLock<Option<String>> = LazyLock::new(|| std::env::var("LANG").ok());
    static LANGUAGE: LazyLock<Option<String>> = LazyLock::new(|| std::env::var("LANGUAGE").ok());
    get_languages(
        LANG.as_ref().map(String::as_str),
        LANGUAGE.as_ref().map(String::as_str),
    )
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RawTlsBackend {
    /// Native TLS (`SChannel` on Windows, Secure Transport on macOS and OpenSSL otherwise)
    NativeTls,
    /// Rustls with `WebPKI` roots.
    RustlsWithWebpkiRoots,
    /// Rustls with native roots.
    RustlsWithNativeRoots,
}

impl Default for RawTlsBackend {
    fn default() -> Self {
        *SUPPORTED_TLS_BACKENDS.first().unwrap()
    }
}

impl std::fmt::Display for RawTlsBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.serialize(f)
    }
}

/// Allows choosing a `reqwest`'s TLS backend. Available TLS backends:
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TlsBackend {
    /// Native TLS (`SChannel` on Windows, Secure Transport on macOS and OpenSSL otherwise)
    #[cfg(feature = "native-tls")]
    NativeTls,
    /// Rustls with `WebPKI` roots.
    #[cfg(feature = "rustls-with-webpki-roots")]
    RustlsWithWebpkiRoots,
    /// Rustls with native roots.
    #[cfg(feature = "rustls-with-native-roots")]
    RustlsWithNativeRoots,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config<'a> {
    pub style: StyleConfig,
    pub display: DisplayConfig,
    pub updates: UpdatesConfig<'a>,
    pub directories: DirectoriesConfig,
    pub file_path: PathWithSource,
}

impl<'a> Config<'a> {
    /// Convert a `RawConfig` to a high-level `Config`.
    ///
    /// For this, some values need to be converted to other types and some
    /// defaults need to be set (sometimes based on env variables).
    fn from_raw(raw_config: &'a RawConfig, config_file_path: PathWithSource) -> Result<Self> {
        let style = (&raw_config.style).into();
        let display = (&raw_config.display).into();
        let updates = (&raw_config.updates).try_into()?;
        let relative_path_root = config_file_path
            .path()
            .parent()
            .context("Failed to get config directory")?;

        // Determine directories config. For this, we need to take some
        // additional factory into account, like env variables, or the
        // user config.
        let cache_dir_env_var = "TEALDEER_CACHE_DIR";
        let cache_dir = if let Ok(env_var) = env::var(cache_dir_env_var) {
            // For backwards compatibility reasons, the cache directory can be
            // overridden using an env variable. This is deprecated and will be
            // phased out in the future.
            eprintln!("Warning: The ${cache_dir_env_var} env variable is deprecated, use the `cache_dir` option in the config file instead.");
            PathWithSource {
                path: PathBuf::from(env_var),
                source: PathSource::EnvVar,
            }
        } else if let Some(config_value) = &raw_config.directories.cache_dir {
            // If the user explicitly configured a cache directory, use that.
            PathWithSource {
                // Resolve possible relative path. It would be nicer to clean up the path, but Rust stdlib
                // does not give any method for that that does not need the paths to exist.
                path: relative_path_root.join(config_value),
                source: PathSource::ConfigFile,
            }
        } else if let Ok(default_dir) = get_app_root(AppDataType::UserCache, &crate::APP_INFO) {
            // Otherwise, fall back to the default user cache directory.
            PathWithSource {
                path: default_dir,
                source: PathSource::OsConvention,
            }
        } else {
            // If everything fails, give up
            bail!("Could not determine user cache directory");
        };
        let custom_pages_dir = raw_config
            .directories
            .custom_pages_dir
            .as_ref()
            .map(|path| PathWithSource {
                // Resolve possible relative path.
                path: relative_path_root.join(path),
                source: PathSource::ConfigFile,
            })
            .or_else(|| {
                get_app_root(AppDataType::UserData, &crate::APP_INFO)
                    .map(|path| {
                        // Note: The `join("")` call ensures that there's a trailing slash
                        PathWithSource {
                            path: path.join("pages").join(""),
                            source: PathSource::OsConvention,
                        }
                    })
                    .ok()
            });
        let directories = DirectoriesConfig {
            cache_dir,
            custom_pages_dir,
        };

        Ok(Self {
            style,
            display,
            updates,
            directories,
            file_path: config_file_path,
        })
    }
}

/// The [`ConfigLoader`] is used to load a [`Config`] from a file.
///
/// Since the rich [`Config`] keeps references to [`RawConfig`], the raw config needs to be kept alive outside of the
/// [`Config`]. The [`ConfigLoader`] thus offers the following flow:
/// 1. Read a raw config using [`ConfigLoader::read`] or [`ConfigLoader::read_default_path`].
/// 2. Validate the contents to a [`Config`] that borrows the [`ConfigLoader`].
pub struct ConfigLoader {
    raw: RawConfig,
    path: PathWithSource,
}

impl ConfigLoader {
    fn read_internal(path: PathWithSource, allow_not_found: bool) -> Result<Self> {
        match fs::read_to_string(&path.path) {
            Ok(content) => Ok(Self {
                raw: toml::from_str(&content).with_context(|| {
                    format!(
                        "Could not parse config file contents as toml from {}.",
                        path.path.display()
                    )
                })?,
                path,
            }),
            Err(e) if allow_not_found && e.kind() == ErrorKind::NotFound => Ok(Self {
                raw: RawConfig::default(),
                path,
            }),
            Err(e) => Err(e).context(format!(
                "Could not read config file contents from {}.",
                path.path().display()
            )),
        }
    }

    /// Create a loader that uses the config at `path`.
    pub fn read(path: PathBuf) -> Result<Self> {
        Self::read_internal(
            PathWithSource {
                path,
                source: PathSource::Cli,
            },
            false,
        )
    }

    /// Create a loader that uses the default config file location. If no file is present at the default location, the
    /// default configuration is used.
    pub fn read_default_path() -> Result<Self> {
        let path = get_default_config_path().context("Could not determine default config path.")?;
        Self::read_internal(path, true)
    }

    /// Parse the read [`RawConfig`] into a [`Config`].
    pub fn load(&self) -> Result<Config<'_>> {
        Config::from_raw(&self.raw, self.path.clone())
            .context("Could not process raw config into rich config")
    }
}

/// Return the path to the config directory.
///
/// The config dir path can be overridden using the `TEALDEER_CONFIG_DIR` env
/// variable. Otherwise, the user config directory is returned.
///
/// Note that this function does not verify whether the directory at that
/// location exists, or is a directory.
pub fn get_config_dir() -> Result<(PathBuf, PathSource)> {
    // Allow overriding the config directory by setting the
    // $TEALDEER_CONFIG_DIR env variable.
    if let Ok(value) = env::var("TEALDEER_CONFIG_DIR") {
        return Ok((PathBuf::from(value), PathSource::EnvVar));
    }

    // Otherwise, fall back to the user config directory.
    let dirs = get_app_root(AppDataType::UserConfig, &crate::APP_INFO)
        .context("Failed to determine the user config directory")?;
    Ok((dirs, PathSource::OsConvention))
}

/// Return the path to the config file.
///
/// Note that this function does not verify whether the file at that location
/// exists, or is a file.
pub fn get_default_config_path() -> Result<PathWithSource> {
    let (config_dir, source) = get_config_dir()?;
    let config_file_path = config_dir.join(CONFIG_FILE_NAME);
    Ok(PathWithSource {
        path: config_file_path,
        source,
    })
}

/// Create default config file.
/// path: Can be specified to create the config in that path instead of
/// the default path.
pub fn make_default_config(path: Option<&Path>) -> Result<PathBuf> {
    let config_file_path = if let Some(p) = path {
        p.into()
    } else {
        let (config_dir, _) = get_config_dir()?;

        // Ensure that config directory exists
        if config_dir.exists() {
            ensure!(
                config_dir.is_dir(),
                "Config directory could not be created: {} already exists but is not a directory",
                config_dir.to_string_lossy(),
            );
        } else {
            fs::create_dir_all(&config_dir).context("Could not create config directory")?;
        }

        config_dir.join(CONFIG_FILE_NAME)
    };

    // Ensure that a config file doesn't get overwritten
    ensure!(
        !config_file_path.is_file(),
        "A configuration file already exists at {}, no action was taken.",
        config_file_path.to_str().unwrap()
    );

    // Create default config
    let serialized_config =
        toml::to_string(&RawConfig::default()).context("Failed to serialize default config")?;

    // Write default config
    let mut config_file =
        File::create(&config_file_path).context("Could not create config file")?;
    let _wc = config_file
        .write(serialized_config.as_bytes())
        .context("Could not write to config file")?;

    Ok(config_file_path)
}

#[test]
fn test_serialize_deserialize() {
    let raw_config = RawConfig::default();
    let serialized = toml::to_string(&raw_config).unwrap();
    let deserialized: RawConfig = toml::from_str(&serialized).unwrap();
    assert_eq!(raw_config, deserialized);
}

#[test]
fn test_relative_path_resolution() {
    let mut raw_config = RawConfig::default();
    raw_config.directories.cache_dir = Some("../cache".into());
    raw_config.directories.custom_pages_dir = Some("../custom_pages".into());

    let config = Config::from_raw(
        &raw_config,
        PathWithSource {
            path: PathBuf::from("/path/to/config/config.toml"),
            source: PathSource::OsConvention,
        },
    )
    .unwrap();

    assert_eq!(
        config.directories.cache_dir.path(),
        Path::new("/path/to/config/../cache")
    );
    assert_eq!(
        config.directories.custom_pages_dir.unwrap().path(),
        Path::new("/path/to/config/../custom_pages")
    );
}

#[cfg(test)]
mod test {
    use super::*;

    mod language {
        use super::*;

        #[test]
        fn missing_lang_env() {
            let lang_list = get_languages(None, Some("de:fr"));
            assert_eq!(lang_list, [Language("en")]);
            let lang_list = get_languages(None, None);
            assert_eq!(lang_list, [Language("en")]);
        }

        #[test]
        fn missing_language_env() {
            let lang_list = get_languages(Some("de"), None);
            assert_eq!(lang_list, [Language("de"), Language("en")]);
        }

        #[test]
        fn preference_order() {
            let lang_list = get_languages(Some("de"), Some("fr:cn"));
            assert_eq!(
                lang_list,
                [
                    Language("fr"),
                    Language("cn"),
                    Language("de"),
                    Language("en")
                ]
            );
        }

        #[test]
        fn country_code_expansion() {
            let lang_list = get_languages(Some("pt_BR"), None);
            assert_eq!(
                lang_list,
                [Language("pt_BR"), Language("pt"), Language("en")]
            );
        }

        #[test]
        fn ignore_posix_and_c() {
            let lang_list = get_languages(Some("POSIX"), None);
            assert_eq!(lang_list, [Language("en")]);
            let lang_list = get_languages(Some("C"), None);
            assert_eq!(lang_list, [Language("en")]);
        }

        #[test]
        fn no_duplicates() {
            let lang_list = get_languages(Some("de"), Some("fr:de:cn:de"));
            assert_eq!(
                lang_list,
                [
                    Language("fr"),
                    Language("de"),
                    Language("cn"),
                    Language("en")
                ]
            );
        }
    }
}
