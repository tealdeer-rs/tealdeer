//! An implementation of [tldr](https://github.com/tldr-pages/tldr) in Rust.
//
// Copyright (c) 2015-2018 tealdeer developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be
// copied, modified, or distributed except according to those terms.

#![deny(missing_docs, missing_debug_implementations,
        unsafe_code,
        unused_import_braces, unused_qualifications)]
#![warn(trivial_casts, trivial_numeric_casts,
        missing_copy_implementations,
        unused_extern_crates, unused_results)]

#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]
#![cfg_attr(feature = "dev", warn(cast_possible_truncation, cast_possible_wrap, cast_precision_loss, cast_sign_loss,
                                  mut_mut, non_ascii_literal, option_unwrap_used, result_unwrap_used,
                                  shadow_reuse, shadow_same, unicode_not_nfc,
                                  wrong_self_convention, wrong_pub_self_convention))]

#[macro_use] extern crate log;
#[cfg(feature = "logging")] extern crate env_logger;
extern crate docopt;
extern crate ansi_term;
extern crate flate2;
extern crate tar;
extern crate xdg;
extern crate curl;
extern crate time;
extern crate toml;
extern crate walkdir;
#[macro_use]
extern crate serde_derive;

use std::io::BufReader;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process;

use docopt::Docopt;
use ansi_term::Color;

mod types;
mod tokenizer;
mod formatter;
mod cache;
mod config;
mod error;

use tokenizer::Tokenizer;
use cache::Cache;
use config::{get_config_dir, make_default_syntax_config, Config};
use error::TealdeerError::{CacheError, ConfigError, UpdateError};
use formatter::print_lines;
use types::OsType;

const NAME: &'static str = "tealdeer";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
Usage:

    tldr [options] <command>
    tldr [options]

Options:

    -h --help           Show this screen
    -v --version        Show version information
    -l --list           List all commands in the cache
    -f --render <file>  Render a specific markdown file
    -o --os <type>      Override the operating system [linux, osx, sunos]
    -u --update         Update the local cache
    -c --clear-cache    Clear the local cache
    --config-path       Show config directory path
    --seed-config       Create a basic config

Examples:

    $ tldr tar
    $ tldr --list

To control the cache:

    $ tldr --update
    $ tldr --clear-cache

To render a local file (for testing):

    $ tldr --render /path/to/file.md
";
const ARCHIVE_URL: &'static str = "https://github.com/tldr-pages/tldr/archive/master.tar.gz";
const MAX_CACHE_AGE: i64 = 2592000; // 30 days

#[derive(Debug, Deserialize)]
struct Args {
    arg_command: Option<String>,
    flag_help: bool,
    flag_version: bool,
    flag_list: bool,
    flag_render: Option<String>,
    flag_os: Option<OsType>,
    flag_update: bool,
    flag_clear_cache: bool,
    flag_config_path: bool,
    flag_seed_config: bool,
}

/// Print page by path
fn print_page(path: &Path) -> Result<(), String> {
    // Open file
    let file = try!(
        File::open(path).map_err(|msg| format!("Could not open file: {}", msg))
    );
    let reader = BufReader::new(file);

    // Look up config file, if none is found fall back to default config.
    let config = match Config::load() {
        Ok(config) => config,
        Err(ConfigError(msg)) => {
            eprintln!("Failed to create config: {}", msg);
            process::exit(1);
        }
        Err(_) => {
            eprintln!("Unknown error while creating config");
            process::exit(1);
        }
    };

    // Create tokenizer and print output
    let mut tokenizer = Tokenizer::new(reader);
    print_lines(&mut tokenizer, &config);

    Ok(())
}

/// Check the cache for freshness
fn check_cache(args: &Args, cache: &Cache) {
    if !args.flag_update {
        match cache.last_update() {
            Some(ago) if ago > MAX_CACHE_AGE => {
                println!("{}", Color::Red.paint(format!(
                    "Cache wasn't updated in {} days.\n\
                    You should probably run `tldr --update` soon.\n",
                    MAX_CACHE_AGE / 24 / 3600
                )));
            },
            None => {
                eprintln!("Cache not found. Please run `tldr --update`.");
                process::exit(1);
            },
            _ => {},
        }
    };
}

#[cfg(feature = "logging")]
fn init_log() {
    env_logger::init().unwrap();
}

#[cfg(not(feature = "logging"))]
fn init_log() { }

#[cfg(target_os = "linux")]
fn get_os() -> OsType { OsType::Linux }

#[cfg(target_os = "macos")]
fn get_os() -> OsType { OsType::OsX }

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_os() -> OsType { OsType::Other }

fn main() {
    // Initialize logger
    init_log();

    // Parse arguments
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    // Show version and exit
    if args.flag_version {
        println!("{} v{}", NAME, VERSION);
        process::exit(0);
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
        cache.clear().unwrap_or_else(|e| {
            match e {
                CacheError(msg) | ConfigError(msg) | UpdateError(msg) =>
                    eprintln!("Could not delete cache: {}", msg),
            };
            process::exit(1);
        });
        println!("Successfully deleted cache.");
    }

    // Update cache, pass through
    if args.flag_update {
        cache.update().unwrap_or_else(|e| {
            match e {
                CacheError(msg) | ConfigError(msg) | UpdateError(msg) =>
                    eprintln!("Could not update cache: {}", msg),
            };
            process::exit(1);
        });
        println!("Successfully updated cache.");
    }

    // Render local file and exit
    if let Some(ref file) = args.flag_render {
        let path = PathBuf::from(file);
        if let Err(msg) = print_page(&path) {
            eprintln!("{}", msg);
            process::exit(1);
        } else {
            process::exit(0);
        };
    }

    // List cached commands and exit
    if args.flag_list {
        // Check cache for freshness
        check_cache(&args, &cache);

        // Get list of pages
        let pages = cache.list_pages().unwrap_or_else(|e| {
            match e {
                CacheError(msg) | ConfigError(msg) | UpdateError(msg) =>
                    eprintln!("Could not get list of pages: {}", msg),
            }
            process::exit(1);
        });

        // Print pages
        println!("{}", pages.join(", "));
        process::exit(0);
    }

    // Show command from cache
    if let Some(ref command) = args.arg_command {
        // Check cache for freshness
        check_cache(&args, &cache);

        // Search for command in cache
        if let Some(path) = cache.find_page(&command) {
            if let Err(msg) = print_page(&path) {
                eprintln!("{}", msg);
                process::exit(1);
            } else {
                process::exit(0);
            }
        } else {
            println!("Page {} not found in cache", &command);
            println!("Try updating with `tldr --update`, or submit a pull request to:");
            eprintln!("https://github.com/tldr-pages/tldr");
            process::exit(1);
        }
    }

    // Show config file and path and exit
    if args.flag_config_path {
        match get_config_dir() {
            Ok(config_file_path) => {
                println!("Config directory path is: {}", config_file_path.to_str().unwrap());
                process::exit(0);
            },
            Err(ConfigError(msg)) => {
                eprintln!("Could not look up syntax_config_path: {}", msg);
                process::exit(1);
            },
            Err(_) => {
                eprintln!("Unknown error");
                process::exit(1);
            },
        }
    }

    // Create a basic config and exit
    if args.flag_seed_config {
        match make_default_syntax_config() {
            Ok(syntax_config_file_path) => {
                println!("Successfully created seed syntax config file here: {}", syntax_config_file_path.to_str().unwrap());
                process::exit(0);
            },
            Err(ConfigError(msg)) => {
                eprintln!("Could not create seed config: {}", msg);
                process::exit(1);
            },
            Err(_) => {
                eprintln!("Unkown error");
                process::exit(1);
            },
        }
    }

    // Some flags can be run without a command.
    if !(args.flag_update || args.flag_clear_cache) {
        eprintln!("{}", USAGE);
        process::exit(1);
    }
}

#[cfg(test)]
mod test {
    use Args;
    use OsType;
    use USAGE;
    use docopt::Docopt;
    use docopt::Error;

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
}
