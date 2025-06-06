//! An implementation of [tldr](https://github.com/tldr-pages/tldr) in Rust.
//
// Copyright (c) 2015-2021 tealdeer developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be
// copied, modified, or distributed except according to those terms.

#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::enum_glob_use)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]

#[cfg(not(any(
    feature = "native-tls",
    feature = "rustls-with-webpki-roots",
    feature = "rustls-with-native-roots",
)))]
compile_error!(
    "at least one of the features \"native-tls\", \"rustls-with-webpki-roots\" or \"rustls-with-native-roots\" must be enabled"
);

use std::{
    env,
    fs::create_dir_all,
    io::{self, IsTerminal},
    path::Path,
    process::{Command, ExitCode},
    sync::LazyLock,
};

use anyhow::{anyhow, Context, Result};
use app_dirs::AppInfo;
use cache::{CacheConfig, Language};
use clap::Parser;
use config::{StyleConfig, TlsBackend};
use log::debug;

mod cache;
mod cli;
mod config;
pub mod extensions;
mod formatter;
mod line_iterator;
mod output;
mod types;
mod utils;

use crate::{
    cache::{Cache, PageLookupResult, TLDR_PAGES_DIR},
    cli::Cli,
    config::{get_config_dir, make_default_config, Config, PathWithSource},
    extensions::Dedup,
    output::print_page,
    types::{ColorOptions, PlatformType},
    utils::{print_error, print_warning},
};

const NAME: &str = "tealdeer";
const APP_INFO: AppInfo = AppInfo {
    name: NAME,
    author: NAME,
};

/// The cache should be updated if it was explicitly requested,
/// or if an automatic update is due and allowed.
fn should_update_cache(cache: &Cache, args: &Cli, config: &Config) -> bool {
    if args.update {
        return true;
    }
    if args.no_auto_update || !config.updates.auto_update {
        return false;
    }

    matches!(cache.age(), Ok(age) if age >= config.updates.auto_update_interval)
}

#[derive(PartialEq)]
enum CheckCacheResult {
    CacheFound,
    CacheMissing,
}

/// Check the cache for freshness. If it's stale or missing, show a warning.
fn check_cache(cache: &Cache, args: &Cli, enable_styles: bool) -> CheckCacheResult {
    match cache.age() {
        Ok(age) => {
            if age > config::MAX_CACHE_AGE && !args.quiet {
                print_warning(
                    enable_styles,
                    &format!(
                        "The cache hasn't been updated for {} days.\n\
                     You should probably run `tldr --update` soon.",
                        age.as_secs() / 24 / 3600
                    ),
                );
            }

            CheckCacheResult::CacheFound
        }
        Err(_) => {
            print_error(
                enable_styles,
                &anyhow::anyhow!(
                    "Page cache not found. Please run `tldr --update` to download the cache."
                ),
            );
            println!("\nNote: You can optionally enable automatic cache updates by adding the");
            println!("following config to your config file:\n");
            println!("  [updates]");
            println!("  auto_update = true\n");
            println!("The path to your config file can be looked up with `tldr --show-paths`.");
            println!("To create an initial config file, use `tldr --seed-config`.\n");
            println!("You can find more tips and tricks in our docs:\n");
            println!("  https://tealdeer-rs.github.io/tealdeer/config_updates.html");

            CheckCacheResult::CacheMissing
        }
    }
}

/// Clear the cache
fn clear_cache(cache: Cache, quietly: bool) -> Result<()> {
    let cache_dir = cache.config().pages_directory.display();
    cache.clear().context("Could not clear cache")?;
    if !quietly {
        eprintln!("Successfully cleared cache at `{cache_dir}`.");
    }
    Ok(())
}

/// Update the cache
fn update_cache(
    cache: &mut Cache,
    archive_source: &str,
    tls_backend: TlsBackend,
    quietly: bool,
) -> Result<()> {
    cache
        .update(archive_source, tls_backend)
        .context("Could not update cache")?;
    if !quietly {
        eprintln!("Successfully updated cache.");
    }
    Ok(())
}

/// Show file paths
fn show_paths(config: &Config) {
    let config_dir = get_config_dir().map_or_else(
        |e| format!("[Error: {e}]"),
        |(mut path, source)| {
            path.push(""); // Trailing path separator
            match path.to_str() {
                Some(path) => format!("{path} ({source})"),
                None => "[Invalid]".to_string(),
            }
        },
    );
    let config_path = config.file_path.to_string();
    let cache_dir = config.directories.cache_dir.to_string();
    let pages_dir = {
        let mut path = config.directories.cache_dir.path.clone();
        path.push(TLDR_PAGES_DIR);
        path.push(""); // Trailing path separator
        path.display().to_string()
    };
    let custom_pages_dir = match config.directories.custom_pages_dir {
        Some(ref path_with_source) => path_with_source.to_string(),
        None => "[None]".to_string(),
    };
    println!("Config dir:       {config_dir}");
    println!("Config path:      {config_path}");
    println!("Cache dir:        {cache_dir}");
    println!("Pages dir:        {pages_dir}");
    println!("Custom pages dir: {custom_pages_dir}");
}

fn create_config(path: Option<&Path>) -> Result<()> {
    let config_file_path = make_default_config(path).context("Could not create seed config")?;
    eprintln!(
        "Successfully created seed config file here: {}",
        config_file_path.to_str().unwrap()
    );
    Ok(())
}

#[cfg(feature = "logging")]
fn init_log() {
    env_logger::init();
}

#[cfg(not(feature = "logging"))]
fn init_log() {}

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

fn get_languages_from_env<'a>() -> Vec<Language<'a>> {
    static LANG: LazyLock<Option<String>> = LazyLock::new(|| std::env::var("LANG").ok());
    static LANGUAGE: LazyLock<Option<String>> = LazyLock::new(|| std::env::var("LANGUAGE").ok());
    get_languages(
        LANG.as_ref().map(String::as_str),
        LANGUAGE.as_ref().map(String::as_str),
    )
}

fn spawn_editor(custom_pages_dir: &Path, file_name: &str) -> Result<()> {
    create_dir_all(custom_pages_dir).context("Failed to create custom pages directory")?;

    let custom_page_path = custom_pages_dir.join(file_name);
    let Some(custom_page_path) = custom_page_path.to_str() else {
        return Err(anyhow!("`custom_page_path.to_str()` failed"));
    };
    let Ok(editor) = env::var("EDITOR") else {
        return Err(anyhow!(
            "To edit a custom page, please set the `EDITOR` environment variable."
        ));
    };
    println!("Editing {custom_page_path:?}");

    let status = Command::new(&editor).arg(custom_page_path).status()?;
    if !status.success() {
        return Err(anyhow!("{editor} exit with code {:?}", status.code()));
    }
    Ok(())
}

fn main() -> ExitCode {
    // Initialize logger
    init_log();

    // Parse arguments
    let args = Cli::parse();

    // Determine the usage of styles
    let enable_styles = match args.color.unwrap_or_default() {
        // Attempt to use styling if instructed
        ColorOptions::Always => {
            yansi::enable(); // disable yansi's automatic detection for ANSI support on Windows
            true
        }
        // Enable styling if:
        // * NO_COLOR env var isn't set: https://no-color.org/
        // * The output stream is stdout (not being piped)
        ColorOptions::Auto => env::var_os("NO_COLOR").is_none() && io::stdout().is_terminal(),
        // Disable styling
        ColorOptions::Never => false,
    };

    try_main(args, enable_styles).unwrap_or_else(|error| {
        print_error(enable_styles, &error);
        ExitCode::FAILURE
    })
}

fn try_main(args: Cli, enable_styles: bool) -> Result<ExitCode> {
    // Look up config file, if none is found fall back to default config.
    debug!("Loading config");
    let mut config = match &args.config_path {
        Some(path) if !args.seed_config => {
            Config::load(path).context("Could not load config from given path")?
        }
        _ => Config::load_default_path().context("Could not load config from default path")?,
    };

    // Override styles if needed
    if !enable_styles {
        config.style = StyleConfig::default();
    }

    let custom_pages_dir = config
        .directories
        .custom_pages_dir
        .as_ref()
        .map(PathWithSource::path);

    // Note: According to the TLDR client spec, page names must be transparently
    // lowercased before lookup:
    // https://github.com/tldr-pages/tldr/blob/main/CLIENT-SPECIFICATION.md#page-names
    let command = args.command.join("-").to_lowercase();

    if args.edit_patch || args.edit_page {
        let file_name = if args.edit_patch {
            format!("{command}.patch.md")
        } else {
            format!("{command}.page.md")
        };

        custom_pages_dir
            .context("To edit custom pages/patches, please specify a custom pages directory.")
            .and_then(|custom_pages_dir| spawn_editor(custom_pages_dir, &file_name))?;

        return Ok(ExitCode::SUCCESS);
    }

    // Show various paths
    if args.show_paths {
        show_paths(&config);
    }

    // Create a basic config and exit
    if args.seed_config {
        create_config(args.config_path.as_deref())?;
        return Ok(ExitCode::SUCCESS);
    }

    // If a local file was passed in, render it and exit
    if let Some(file) = args.render {
        let path = PageLookupResult::with_page(file);
        print_page(&path, args.raw, enable_styles, args.pager, &config)?;
        return Ok(ExitCode::SUCCESS);
    }

    let platforms = compute_platforms(args.platforms.as_ref());
    let languages = args
        .language
        .as_deref()
        .map_or_else(get_languages_from_env, |lang| vec![Language(lang)]);

    let cache_config = CacheConfig {
        pages_directory: &config.directories.cache_dir.path().join(TLDR_PAGES_DIR),
        custom_pages_directory: config
            .directories
            .custom_pages_dir
            .as_ref()
            .map(PathWithSource::path),
        platforms: &platforms,
        languages: &languages,
    };

    // TODO: check for TLDR_OLD_PAGES_DIR

    if args.clear_cache {
        if let Some(cache) = Cache::open(cache_config)? {
            clear_cache(cache, args.quiet)?;
        };
        return Ok(ExitCode::SUCCESS);
    }

    let mut cache = Cache::open_or_create(cache_config)?;

    if should_update_cache(&cache, &args, &config) {
        update_cache(
            &mut cache,
            &config.updates.archive_source,
            config.updates.tls_backend,
            args.quiet,
        )?;
    } else if (args.list || !args.command.is_empty())
        && check_cache(&cache, &args, enable_styles) == CheckCacheResult::CacheMissing
    {
        // Cache is needed, but missing
        return Ok(ExitCode::FAILURE);
    }

    if args.list {
        for page in cache.list_pages()? {
            println!("{}", page);
        }

        return Ok(ExitCode::SUCCESS);
    }

    // Show command from cache
    if !command.is_empty() {
        // TODO: Remove this check 1 year after version 1.7.0 was released
        if cache.old_custom_pages_exist()? {
            print_warning(
                enable_styles,
                &format!(
                    "Custom pages using the old naming convention were found in {}.\n\
                     Please rename them to follow the new convention:\n\
                     - `<name>.page` → `<name>.page.md`\n\
                     - `<name>.patch` → `<name>.patch.md`",
                    cache
                        .config()
                        .custom_pages_directory
                        .expect("Old custom pages can only exist in custom pages directory")
                        .display(),
                ),
            );
        }

        let Some(lookup_result) = cache.find_page(&command) else {
            if !args.quiet {
                print_warning(
                    enable_styles,
                    &format!(
                        "Page `{}` not found in cache.\n\
                         Try updating with `tldr --update`, or submit a pull request to:\n\
                         https://github.com/tldr-pages/tldr",
                        &command
                    ),
                );
            }

            return Ok(ExitCode::FAILURE);
        };

        print_page(&lookup_result, args.raw, enable_styles, args.pager, &config)?;
    }

    Ok(ExitCode::SUCCESS)
}

/// Returns the passed or default platform types and appends `PlatformType::Common` as fallback.
fn compute_platforms(platforms: Option<&Vec<PlatformType>>) -> Vec<PlatformType> {
    match platforms {
        Some(p) => {
            let mut result = p.clone();
            if !result.contains(&PlatformType::Common) {
                result.push(PlatformType::Common);
            }
            result
        }
        None => vec![PlatformType::current(), PlatformType::Common],
    }
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
