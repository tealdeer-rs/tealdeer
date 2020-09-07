//! An implementation of [tldr](https://github.com/tldr-pages/tldr) in Rust.
//
// Copyright (c) 2015-2020 tealdeer developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be
// copied, modified, or distributed except according to those terms.

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::similar_names)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]

#[cfg(feature = "logging")]
extern crate env_logger;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process;

use ansi_term::{Color, Style};
use app_dirs::AppInfo;
use atty::Stream;
use docopt::Docopt;
#[cfg(not(target_os = "windows"))]
use pager::Pager;
use serde_derive::Deserialize;

mod cache;
mod config;
mod error;
mod formatter;
mod tokenizer;
mod types;

use crate::cache::Cache;
use crate::config::{get_config_path, make_default_config, Config, MAX_CACHE_AGE};
use crate::error::TealdeerError::{CacheError, ConfigError, UpdateError};
use crate::formatter::print_lines;
use crate::tokenizer::Tokenizer;
use crate::types::{ColorOptions, OsType};

const NAME: &str = "tealdeer";
const APP_INFO: AppInfo = AppInfo {
    name: NAME,
    author: NAME,
};
const VERSION: &str = env!("CARGO_PKG_VERSION");
const USAGE: &str = "
Usage:

    tldr [options] <command>...
    tldr [options]

Options:

    -h --help             Show this screen
    -v --version          Show version information
    -l --list             List all commands in the cache
    -f --render <file>    Render a specific markdown file
    -o --os <type>        Override the operating system [linux, osx, sunos, windows]
    -L --language <lang>  Override the language settings
    -u --update           Update the local cache
    -c --clear-cache      Clear the local cache
    -p --pager            Use a pager to page output
    -m --markdown         Display the raw markdown instead of rendering it
    -q --quiet            Suppress informational messages
    --config-path         Show config file path
    --seed-config         Create a basic config
    --color <when>        Control when to use color [always, auto, never] [default: auto]

Examples:

    $ tldr tar
    $ tldr --list

To control the cache:

    $ tldr --update
    $ tldr --clear-cache

To render a local file (for testing):

    $ tldr --render /path/to/file.md
";
const ARCHIVE_URL: &str = "https://github.com/tldr-pages/tldr/archive/master.tar.gz";
#[cfg(not(target_os = "windows"))]
const PAGER_COMMAND: &str = "less -R";

#[derive(Debug, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    arg_command: Option<Vec<String>>,
    flag_help: bool,
    flag_version: bool,
    flag_list: bool,
    flag_render: Option<String>,
    flag_os: Option<OsType>,
    flag_update: bool,
    flag_clear_cache: bool,
    flag_pager: bool,
    flag_quiet: bool,
    flag_config_path: bool,
    flag_seed_config: bool,
    flag_markdown: bool,
    flag_color: ColorOptions,
    flag_language: Option<String>,
}

/// Print page by path
fn print_page(path: &Path, enable_markdown: bool, config: &Config) -> Result<(), String> {
    // Open file
    let file = File::open(path).map_err(|msg| format!("Could not open file: {}", msg))?;
    let reader = BufReader::new(file);

    if enable_markdown {
        // Print the raw markdown of the file.
        for line in reader.lines() {
            println!("{}", line.unwrap());
        }
    } else {
        // Create tokenizer and print output
        let mut tokenizer = Tokenizer::new(reader);
        print_lines(&mut tokenizer, &config);
    };

    Ok(())
}

/// Set up display pager
#[cfg(not(target_os = "windows"))]
fn configure_pager() {
    Pager::with_default_pager(PAGER_COMMAND).setup();
}

#[cfg(target_os = "windows")]
fn configure_pager() {
    eprintln!("Warning: -p / --pager flag not available on Windows!");
}

fn should_update_cache(args: &Args, config: &Config) -> bool {
    args.flag_update
        || (config.updates.auto_update
            && Cache::last_update().map_or(true, |ago| ago >= config.updates.auto_update_interval))
}

/// Check the cache for freshness
fn check_cache(args: &Args, enable_styles: bool) {
    match Cache::last_update() {
        Some(ago) if ago > MAX_CACHE_AGE => {
            if args.flag_quiet {
                return;
            }

            // Only use color if enabled
            let warning_style = if enable_styles {
                Style::new().fg(Color::Yellow)
            } else {
                Style::default()
            };

            eprintln!(
                "{}",
                warning_style.paint(format!(
                    "The cache hasn't been updated for more than {} days.\n\
                         You should probably run `tldr --update` soon.",
                    MAX_CACHE_AGE.as_secs() / 24 / 3600
                ))
            );
        }
        Some(_) => {}
        None => {
            eprintln!("Cache not found. Please run `tldr --update`.");
            process::exit(1);
        }
    };
}

/// Clear the cache
fn clear_cache(quietly: bool) {
    Cache::clear().unwrap_or_else(|e| {
        match e {
            CacheError(msg) | ConfigError(msg) | UpdateError(msg) => {
                eprintln!("Could not delete cache: {}", msg)
            }
        };
        process::exit(1);
    });
    if !quietly {
        println!("Successfully deleted cache.");
    }
}

/// Update the cache
fn update_cache(cache: &Cache, quietly: bool) {
    cache.update().unwrap_or_else(|e| {
        match e {
            CacheError(msg) | ConfigError(msg) | UpdateError(msg) => {
                eprintln!("Could not update cache: {}", msg)
            }
        };
        process::exit(1);
    });
    if !quietly {
        println!("Successfully updated cache.");
    }
}

/// Show the config path
fn show_config_path() {
    match get_config_path() {
        Ok(config_file_path) => {
            println!("Config path is: {}", config_file_path.to_str().unwrap());
        }
        Err(ConfigError(msg)) => {
            eprintln!("Could not look up config_path: {}", msg);
            process::exit(1);
        }
        Err(_) => {
            eprintln!("Unknown error");
            process::exit(1);
        }
    }
}

/// Create seed config file and exit
fn create_config_and_exit() {
    match make_default_config() {
        Ok(config_file_path) => {
            println!(
                "Successfully created seed config file here: {}",
                config_file_path.to_str().unwrap()
            );
            process::exit(0);
        }
        Err(ConfigError(msg)) => {
            eprintln!("Could not create seed config: {}", msg);
            process::exit(1);
        }
        Err(_) => {
            eprintln!("Unknown error");
            process::exit(1);
        }
    }
}

#[cfg(feature = "logging")]
fn init_log() {
    env_logger::init();
}

#[cfg(not(feature = "logging"))]
fn init_log() {}

#[cfg(target_os = "linux")]
fn get_os() -> OsType {
    OsType::Linux
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "dragonfly"
))]
fn get_os() -> OsType {
    OsType::OsX
}

#[cfg(target_os = "windows")]
fn get_os() -> OsType {
    OsType::Windows
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
fn get_os() -> OsType {
    OsType::Other
}

fn get_languages(
    env_lang: Result<String, std::env::VarError>,
    env_language: Result<String, std::env::VarError>,
) -> Vec<String> {
    // Language list according to
    // https://github.com/tldr-pages/tldr/blob/master/CLIENT-SPECIFICATION.md#language

    if let Ok(lang) = env_lang {
        let language = env_language.unwrap_or_default();
        let mut locales: Vec<&str> = language.split(':').collect();
        locales.push(&lang);
        locales.push("en");

        let mut lang_list = Vec::new();
        let mut found_languages = HashSet::new();

        for locale in &locales {
            if locale.len() >= 5 && locale.chars().nth(2) == Some('_') {
                // Language with country code
                let lang = &locale[..5];
                if found_languages.insert(lang) {
                    lang_list.push(lang);
                }
            }
            if locale.len() >= 2 && *locale != "POSIX" {
                // Language code
                let lang = &locale[..2];
                if found_languages.insert(lang) {
                    lang_list.push(lang);
                }
            }
        }

        return lang_list.iter().map(|&s| String::from(s)).collect();
    }

    // Without the LANG environment variable, only English pages should be looked up.
    vec!["en".into()]
}

fn main() {
    // Initialize logger
    init_log();

    // Parse arguments
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // Show version and exit
    if args.flag_version {
        let os = get_os();
        println!("{} v{} ({})", NAME, VERSION, os);
        process::exit(0);
    }

    // Show config file and path, pass through
    if args.flag_config_path {
        show_config_path();
    }

    // Create a basic config and exit
    if args.flag_seed_config {
        create_config_and_exit();
    }

    // Determine the usage of styles
    #[cfg(target_os = "windows")]
    let ansi_support = ansi_term::enable_ansi_support().is_ok();
    #[cfg(not(target_os = "windows"))]
    let ansi_support = true;

    let enable_styles = match args.flag_color {
        // Attempt to use styling if instructed
        ColorOptions::Always => true,
        // Enable styling if:
        // * There is `ansi_support`
        // * NO_COLOR env var isn't set: https://no-color.org/
        // * The output stream is stdout (not being piped)
        ColorOptions::Auto => {
            ansi_support && env::var_os("NO_COLOR").is_none() && atty::is(Stream::Stdout)
        }
        // Disable styling
        ColorOptions::Never => false,
    };

    // Look up config file, if none is found fall back to default config.
    let config = match Config::load(enable_styles) {
        Ok(config) => config,
        Err(ConfigError(msg)) => {
            eprintln!("Could not load config: {}", msg);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Could not load config: {}", e);
            process::exit(1);
        }
    };

    if args.flag_pager || config.display.use_pager {
        configure_pager();
    }

    // Specify target OS
    let os: OsType = match args.flag_os {
        Some(os) => os,
        None => get_os(),
    };

    // Initialize cache
    let cache = Cache::new(ARCHIVE_URL, os);

    // Clear cache, pass through
    if args.flag_clear_cache {
        clear_cache(args.flag_quiet);
    }

    // Update cache, pass through
    let cache_updated = if should_update_cache(&args, &config) {
        update_cache(&cache, args.flag_quiet);
        true
    } else {
        false
    };

    // Render local file and exit
    if let Some(ref file) = args.flag_render {
        let path = PathBuf::from(file);
        if let Err(msg) = print_page(&path, args.flag_markdown, &config) {
            eprintln!("{}", msg);
            process::exit(1);
        } else {
            process::exit(0);
        };
    }

    // List cached commands and exit
    if args.flag_list {
        if !cache_updated {
            // Check cache for freshness
            check_cache(&args, enable_styles);
        }

        // Get list of pages
        let pages = cache.list_pages().unwrap_or_else(|e| {
            match e {
                CacheError(msg) | ConfigError(msg) | UpdateError(msg) => {
                    eprintln!("Could not get list of pages: {}", msg)
                }
            }
            process::exit(1);
        });

        // Print pages
        println!("{}", pages.join("\n"));
        process::exit(0);
    }

    // Show command from cache
    if let Some(ref command) = args.arg_command {
        let command = command.join("-");

        if !cache_updated {
            // Check cache for freshness
            check_cache(&args, enable_styles);
        }

        let languages = if let Some(ref lang) = args.flag_language {
            // Language overwritten by console argument
            vec![lang.clone()]
        } else {
            get_languages(std::env::var("LANG"), std::env::var("LANGUAGE"))
        };

        // Search for command in cache
        if let Some(path) = cache.find_page(&command, &languages) {
            if let Err(msg) = print_page(&path, args.flag_markdown, &config) {
                eprintln!("{}", msg);
                process::exit(1);
            } else {
                process::exit(0);
            }
        } else {
            if !args.flag_quiet {
                println!("Page {} not found in cache", &command);
                println!("Try updating with `tldr --update`, or submit a pull request to:");
                println!("https://github.com/tldr-pages/tldr");
            }
            process::exit(1);
        }
    }

    // Some flags can be run without a command.
    if !(args.flag_update || args.flag_clear_cache || args.flag_config_path) {
        eprintln!("{}", USAGE);
        process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use crate::{get_languages, Args, OsType, USAGE};
    use docopt::{Docopt, Error};

    fn test_helper(argv: &[&str]) -> Result<Args, Error> {
        Docopt::new(USAGE).and_then(|d| d.argv(argv.iter()).deserialize())
    }

    #[test]
    fn test_docopt_os_case_insensitive() {
        let argv = vec!["cp", "--os", "LiNuX"];
        let os = test_helper(&argv).unwrap().flag_os.unwrap();
        assert_eq!(OsType::Linux, os);
    }

    #[test]
    fn test_docopt_expect_error() {
        let argv = vec!["cp", "--os", "lindows"];
        assert!(!test_helper(&argv).is_ok());
    }

    #[test]
    fn test_language_missing_lang_env() {
        let lang_list = get_languages(Err(std::env::VarError::NotPresent), Ok("de:fr".into()));
        assert_eq!(lang_list, vec!["en"]);
        let lang_list = get_languages(
            Err(std::env::VarError::NotPresent),
            Err(std::env::VarError::NotPresent),
        );
        assert_eq!(lang_list, vec!["en"]);
    }

    #[test]
    fn test_language_missing_language_env() {
        let lang_list = get_languages(Ok("de".into()), Err(std::env::VarError::NotPresent));
        assert_eq!(lang_list, vec!["de", "en"]);
    }

    #[test]
    fn test_language_preference_order() {
        let lang_list = get_languages(Ok("de".into()), Ok("fr:cn".into()));
        assert_eq!(lang_list, vec!["fr", "cn", "de", "en"]);
    }

    #[test]
    fn test_language_country_code_expansion() {
        let lang_list = get_languages(Ok("pt_BR".into()), Err(std::env::VarError::NotPresent));
        assert_eq!(lang_list, vec!["pt_BR", "pt", "en"]);
    }

    #[test]
    fn test_language_ignore_posix_and_c() {
        let lang_list = get_languages(Ok("POSIX".into()), Err(std::env::VarError::NotPresent));
        assert_eq!(lang_list, vec!["en"]);
        let lang_list = get_languages(Ok("C".into()), Err(std::env::VarError::NotPresent));
        assert_eq!(lang_list, vec!["en"]);
    }

    #[test]
    fn test_language_no_duplicates() {
        let lang_list = get_languages(Ok("de".into()), Ok("fr:de:cn:de".into()));
        assert_eq!(lang_list, vec!["fr", "de", "cn", "en"]);
    }
}
