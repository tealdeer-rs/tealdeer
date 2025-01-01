//! Integration tests.

use std::{
    fs::{self, create_dir_all, File},
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime},
};

use assert_cmd::prelude::*;
use predicates::{
    boolean::PredicateBooleanExt,
    prelude::predicate::str::{contains, diff, is_empty, is_match},
};
use tempfile::{Builder as TempfileBuilder, TempDir};

// TODO: Should be 'cache::CACHE_DIR_ENV_VAR'. This requires to have a library crate for the logic.
static CACHE_DIR_ENV_VAR: &str = "TEALDEER_CACHE_DIR";

pub static TLDR_PAGES_DIR: &str = "tldr-pages";

struct TestEnv {
    pub cache_dir: TempDir,
    pub custom_pages_dir: TempDir,
    pub config_dir: TempDir,
    pub default_features: bool,
    pub features: Vec<String>,
}

impl TestEnv {
    fn new() -> Self {
        TestEnv {
            cache_dir: TempfileBuilder::new()
                .prefix(".tldr.test.cache")
                .tempdir()
                .unwrap(),
            config_dir: TempfileBuilder::new()
                .prefix(".tldr.test.conf")
                .tempdir()
                .unwrap(),
            custom_pages_dir: TempfileBuilder::new()
                .prefix(".tldr.test.custom-pages")
                .tempdir()
                .unwrap(),
            default_features: true,
            features: vec![],
        }
    }

    /// Write `content` to "config.toml" in the `config_dir` directory
    fn write_config(&self, content: impl AsRef<str>) {
        let config_file_name = self.config_dir.path().join("config.toml");
        println!("Config path: {config_file_name:?}");

        fs::write(config_file_name, content.as_ref().as_bytes()).unwrap();
    }

    /// Add entry for that environment to the "common" pages.
    fn add_entry(&self, name: &str, contents: &str) {
        self.add_os_entry("common", name, contents);
    }

    /// Add entry for that environment to an OS-specific subfolder.
    fn add_os_entry(&self, os: &str, name: &str, contents: &str) {
        let dir = self
            .cache_dir
            .path()
            .join(TLDR_PAGES_DIR)
            .join("pages")
            .join(os);
        create_dir_all(&dir).unwrap();

        let mut file = File::create(dir.join(format!("{name}.md"))).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    /// Add custom patch entry to the custom_pages_dir
    fn add_page_entry(&self, name: &str, contents: &str) {
        let dir = self.custom_pages_dir.path();
        create_dir_all(dir).unwrap();
        let mut file = File::create(dir.join(format!("{name}.page.md"))).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    /// Add custom patch entry to the custom_pages_dir
    fn add_patch_entry(&self, name: &str, contents: &str) {
        let dir = self.custom_pages_dir.path();
        create_dir_all(dir).unwrap();
        let mut file = File::create(dir.join(format!("{name}.patch.md"))).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    /// Disable default features.
    fn no_default_features(mut self) -> Self {
        self.default_features = false;
        self
    }

    /// Add the specified feature.
    fn with_feature<S: Into<String>>(mut self, feature: S) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Return a new `Command` with env vars set.
    fn command(&self) -> Command {
        let mut build = escargot::CargoBuild::new()
            .bin("tldr")
            .arg("--color=never")
            .current_release()
            .current_target();
        if !self.default_features {
            build = build.no_default_features();
        }
        if !self.features.is_empty() {
            build = build.features(self.features.join(" "))
        }
        let run = build.run().expect("Failed to build tealdeer for testing");
        let mut cmd = run.command();
        cmd.env(CACHE_DIR_ENV_VAR, self.cache_dir.path().to_str().unwrap());
        cmd.env(
            "TEALDEER_CONFIG_DIR",
            self.config_dir.path().to_str().unwrap(),
        );
        cmd
    }

    fn install_default_cache(self) -> Self {
        copy_recursively(
            &PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "tests", "cache"]),
            &self.cache_dir.path().join(TLDR_PAGES_DIR),
        )
        .expect("Failed to copy the cache to the test environment");

        self
    }

    fn install_default_custom_pages(self) -> Self {
        copy_recursively(
            &PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "tests", "custom-pages"]),
            self.custom_pages_dir.path(),
        )
        .expect("Failed to copy the custom pages to the test environment");

        self.write_custom_pages_config()
    }

    fn write_custom_pages_config(self) -> Self {
        self.write_config(format!(
            "[directories]\ncustom_pages_dir = '{}'",
            self.custom_pages_dir.path().to_str().unwrap()
        ));

        self
    }
}

fn copy_recursively(source: &Path, destination: &Path) -> io::Result<()> {
    if source.is_dir() {
        fs::create_dir_all(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_recursively(&entry.path(), &destination.join(entry.file_name()))?;
        }
    } else {
        fs::copy(source, destination)?;
    }

    Ok(())
}

#[test]
#[should_panic]
fn test_cannot_build_without_tls_feature() {
    let _ = TestEnv::new().no_default_features().command();
}

#[test]
fn test_missing_cache() {
    TestEnv::new()
        .command()
        .args(["sl"])
        .assert()
        .failure()
        .stderr(contains("Page cache not found. Please run `tldr --update`"));
}

#[cfg_attr(feature = "ignore-online-tests", ignore = "online test")]
#[test]
fn test_update_cache_default_features() {
    let testenv = TestEnv::new();

    testenv
        .command()
        .args(["sl"])
        .assert()
        .failure()
        .stderr(contains("Page cache not found. Please run `tldr --update`"));

    testenv
        .command()
        .args(["--update"])
        .assert()
        .success()
        .stderr(contains("Successfully updated cache."));

    testenv.command().args(["sl"]).assert().success();
}

#[cfg_attr(feature = "ignore-online-tests", ignore = "online test")]
#[test]
fn test_update_cache_rustls_webpki() {
    let testenv = TestEnv::new()
        .no_default_features()
        .with_feature("webpki-roots");

    testenv
        .command()
        .args(["sl"])
        .assert()
        .failure()
        .stderr(contains("Page cache not found. Please run `tldr --update`"));

    testenv
        .command()
        .args(["--update"])
        .assert()
        .success()
        .stderr(contains("Successfully updated cache."));

    testenv.command().args(["sl"]).assert().success();
}

#[cfg_attr(feature = "ignore-online-tests", ignore = "online test")]
#[test]
fn test_quiet_cache() {
    let testenv = TestEnv::new();
    testenv
        .command()
        .args(["--update", "--quiet"])
        .assert()
        .success()
        .stdout(is_empty());

    testenv
        .command()
        .args(["--clear-cache", "--quiet"])
        .assert()
        .success()
        .stdout(is_empty());
}

#[test]
fn test_quiet_failures() {
    let testenv = TestEnv::new().install_default_cache();

    testenv
        .command()
        .args(["fakeprogram", "-q"])
        .assert()
        .failure()
        .stdout(is_empty());
}

#[test]
fn test_quiet_old_cache() {
    let testenv = TestEnv::new().install_default_cache();

    filetime::set_file_mtime(
        testenv.cache_dir.path().join(TLDR_PAGES_DIR),
        filetime::FileTime::from_unix_time(1, 0),
    )
    .unwrap();

    testenv
        .command()
        .args(["which"])
        .assert()
        .success()
        .stderr(contains("The cache hasn't been updated for "));

    testenv
        .command()
        .args(["which", "--quiet"])
        .assert()
        .success()
        .stderr(contains("The cache hasn't been updated for ").not());
}

#[cfg_attr(feature = "ignore-online-tests", ignore = "online test")]
#[test]
fn test_create_cache_directory_path() {
    let testenv = TestEnv::new();
    let cache_dir = testenv.cache_dir.path();
    let internal_cache_dir = cache_dir.join("internal");

    let mut command = testenv.command();
    command.env(CACHE_DIR_ENV_VAR, internal_cache_dir.to_str().unwrap());

    assert!(!internal_cache_dir.exists());

    command
        .arg("--update")
        .assert()
        .success()
        .stderr(contains(format!(
            "Successfully created cache directory path `{}`.",
            internal_cache_dir.to_str().unwrap()
        )))
        .stderr(contains("Successfully updated cache."));

    assert!(internal_cache_dir.is_dir());
}

#[cfg_attr(feature = "ignore-online-tests", ignore = "online test")]
#[test]
fn test_cache_location_not_a_directory() {
    let testenv = TestEnv::new();
    let cache_dir = testenv.cache_dir.path();
    let internal_file = cache_dir.join("internal");
    File::create(&internal_file).unwrap();

    let mut command = testenv.command();
    command.env(CACHE_DIR_ENV_VAR, internal_file.to_str().unwrap());

    command
        .arg("--update")
        .assert()
        .failure()
        .stderr(contains(format!(
            "Cache directory path `{}` is not a directory",
            internal_file.display(),
        )))
        .stderr(contains(
            "Warning: The $TEALDEER_CACHE_DIR env variable is deprecated",
        ));
}

#[test]
fn test_cache_location_source() {
    let testenv = TestEnv::new();
    let default_cache_dir = testenv.cache_dir.path();
    let tmp_cache_dir = TempfileBuilder::new()
        .prefix(".tldr.test.cache_dir")
        .tempdir()
        .unwrap();

    // Source: Default (OS convention)
    let mut command = testenv.command();
    command.env_remove(CACHE_DIR_ENV_VAR);
    command
        .arg("--show-paths")
        .assert()
        .success()
        .stdout(is_match("\nCache dir:        [^(]* \\(OS convention\\)\n").unwrap());

    // Source: Config variable
    let mut command = testenv.command();
    command.env_remove(CACHE_DIR_ENV_VAR);
    testenv.write_config(format!(
        "[directories]\ncache_dir = '{}'",
        tmp_cache_dir.path().to_str().unwrap(),
    ));
    command
        .arg("--show-paths")
        .assert()
        .success()
        .stdout(is_match("\nCache dir:        [^(]* \\(config file\\)\n").unwrap());

    // Source: Env var
    let mut command = testenv.command();
    command.env(CACHE_DIR_ENV_VAR, default_cache_dir.to_str().unwrap());
    command
        .arg("--show-paths")
        .assert()
        .success()
        .stdout(is_match("\nCache dir:        [^(]* \\(env variable\\)\n").unwrap());
}

#[test]
fn test_setup_seed_config() {
    let testenv = TestEnv::new();

    testenv
        .command()
        .args(["--seed-config"])
        .assert()
        .success()
        .stderr(contains("Successfully created seed config file here"));

    assert!(testenv.config_dir.path().join("config.toml").is_file());
}

#[test]
fn test_show_paths() {
    let testenv = TestEnv::new();

    // Show general commands
    testenv
        .command()
        .args(["--show-paths"])
        .assert()
        .success()
        .stdout(contains(format!(
            "Config dir:       {}",
            testenv.config_dir.path().to_str().unwrap(),
        )))
        .stdout(contains(format!(
            "Config path:      {}",
            testenv
                .config_dir
                .path()
                .join("config.toml")
                .to_str()
                .unwrap(),
        )))
        .stdout(contains(format!(
            "Cache dir:        {}",
            testenv.cache_dir.path().to_str().unwrap(),
        )))
        .stdout(contains(format!(
            "Pages dir:        {}",
            testenv
                .cache_dir
                .path()
                .join(TLDR_PAGES_DIR)
                .to_str()
                .unwrap(),
        )));

    // Set custom pages directory
    testenv.write_config(format!(
        "[directories]\ncustom_pages_dir = '{}'",
        testenv.custom_pages_dir.path().to_str().unwrap()
    ));

    // Now ensure that this path is contained in the output
    testenv
        .command()
        .args(["--show-paths"])
        .assert()
        .success()
        .stdout(contains(format!(
            "Custom pages dir: {}",
            testenv.custom_pages_dir.path().to_str().unwrap(),
        )));
}

#[test]
fn test_os_specific_page() {
    let testenv = TestEnv::new();

    testenv.add_os_entry("sunos", "truss", "contents");

    testenv
        .command()
        .args(["--platform", "sunos", "truss"])
        .assert()
        .success();
}

#[test]
fn test_markdown_rendering() {
    let testenv = TestEnv::new().install_default_cache();

    let expected = include_str!("cache/pages/common/which.md");
    testenv
        .command()
        .args(["--raw", "which"])
        .assert()
        .success()
        .stdout(diff(expected));
}

fn _test_correct_rendering(page: &str, expected: &'static str, additional_args: &[&str]) {
    let testenv = TestEnv::new().install_default_cache();

    testenv
        .command()
        .args(additional_args)
        .arg(page)
        .assert()
        .success()
        .stdout(diff(expected));
}

/// An end-to-end integration test for direct file rendering (v1 syntax).
#[test]
fn test_correct_rendering_v1() {
    _test_correct_rendering(
        "inkscape-v1",
        include_str!("rendered/inkscape-default.expected"),
        &["--color", "always"],
    );
}

/// An end-to-end integration test for direct file rendering (v2 syntax).
#[test]
fn test_correct_rendering_v2() {
    _test_correct_rendering(
        "inkscape-v2",
        include_str!("rendered/inkscape-default.expected"),
        &["--color", "always"],
    );
}

#[test]
/// An end-to-end integration test for direct file rendering with the `--color auto` option. This
/// will not use styling since output is not stdout.
fn test_rendering_color_auto() {
    _test_correct_rendering(
        "inkscape-v2",
        include_str!("rendered/inkscape-default-no-color.expected"),
        &["--color", "auto"],
    );
}

#[test]
/// An end-to-end integration test for direct file rendering with the `--color never` option.
fn test_rendering_color_never() {
    _test_correct_rendering(
        "inkscape-v2",
        include_str!("rendered/inkscape-default-no-color.expected"),
        &["--color", "never"],
    );
}

#[test]
fn test_rendering_i18n() {
    _test_correct_rendering(
        "apt",
        include_str!("rendered/apt.ja.expected"),
        &["--color", "always", "--language", "ja"],
    );
}

/// An end-to-end integration test for rendering with custom syntax config.
#[test]
fn test_correct_rendering_with_config() {
    let testenv = TestEnv::new().install_default_cache();

    // Setup config file
    // TODO should be config::CONFIG_FILE_NAME
    fs::write(
        testenv.config_dir.path().join("config.toml"),
        include_bytes!("config.toml"),
    )
    .unwrap();

    let expected = include_str!("rendered/inkscape-with-config.expected");

    testenv
        .command()
        .args(["--color", "always", "inkscape-v2"])
        .assert()
        .success()
        .stdout(diff(expected));
}

#[test]
fn test_spaces_find_command() {
    let testenv = TestEnv::new().install_default_cache();

    testenv
        .command()
        .args(["git", "checkout"])
        .assert()
        .success();
}

#[test]
fn test_pager_flag_enable() {
    let testenv = TestEnv::new().install_default_cache();

    testenv
        .command()
        .args(["--pager", "which"])
        .assert()
        .success();
}

#[test]
fn test_multiple_platform_command_search() {
    let testenv = TestEnv::new();
    testenv.add_os_entry("linux", "linux-only", "this command only exists for linux");
    testenv.add_os_entry(
        "linux",
        "windows-and-linux",
        "# windows-and-linux \n\n > linux version",
    );
    testenv.add_os_entry(
        "windows",
        "windows-and-linux",
        "# windows-and-linux \n\n > windows version",
    );

    testenv
        .command()
        .args(["--platform", "windows", "--platform", "linux", "linux-only"])
        .assert()
        .success();

    // test order of platforms supplied if preserved
    testenv
        .command()
        .args([
            "--platform",
            "windows",
            "--platform",
            "linux",
            "windows-and-linux",
        ])
        .assert()
        .success()
        .stdout(contains("windows version"));

    testenv
        .command()
        .args([
            "--platform",
            "linux",
            "--platform",
            "windows",
            "windows-and-linux",
        ])
        .assert()
        .success()
        .stdout(contains("linux version"));
}

#[test]
fn test_multiple_platform_command_search_not_found() {
    let testenv = TestEnv::new();
    testenv.add_os_entry(
        "windows",
        "windows-only",
        "this command only exists for Windows",
    );

    testenv
        .command()
        .args(["--platform", "macos", "--platform", "linux", "windows-only"])
        .assert()
        .stderr(contains("Page `windows-only` not found in cache."));
}

#[test]
fn test_list_flag_rendering() {
    let testenv = TestEnv::new().write_custom_pages_config();

    testenv
        .command()
        .args(["--list"])
        .assert()
        .failure()
        .stderr(contains("Page cache not found. Please run `tldr --update`"));

    testenv.add_entry("foo", "");

    testenv
        .command()
        .args(["--list"])
        .assert()
        .success()
        .stdout("foo\n");

    testenv.add_entry("bar", "");
    testenv.add_entry("baz", "");
    testenv.add_entry("qux", "");
    testenv.add_page_entry("faz", "");
    testenv.add_page_entry("bar", "");
    testenv.add_page_entry("fiz", "");
    testenv.add_patch_entry("buz", "");

    testenv
        .command()
        .args(["--list"])
        .assert()
        .success()
        .stdout("bar\nbaz\nfaz\nfiz\nfoo\nqux\n");
}

#[test]
fn test_multi_platform_list_flag_rendering() {
    let testenv = TestEnv::new().write_custom_pages_config();

    testenv.add_entry("common", "");

    testenv
        .command()
        .args(["--list"])
        .assert()
        .success()
        .stdout("common\n");

    testenv
        .command()
        .args(["--platform", "linux", "--list"])
        .assert()
        .success()
        .stdout("common\n");

    testenv
        .command()
        .args(["--platform", "windows", "--list"])
        .assert()
        .success()
        .stdout("common\n");

    testenv.add_os_entry("linux", "rm", "");
    testenv.add_os_entry("linux", "ls", "");
    testenv.add_os_entry("windows", "del", "");
    testenv.add_os_entry("windows", "dir", "");
    testenv.add_os_entry("linux", "winux", "");
    testenv.add_os_entry("windows", "winux", "");

    // test `--list` for `--platform linux` by itself
    testenv
        .command()
        .args(["--platform", "linux", "--list"])
        .assert()
        .success()
        .stdout("common\nls\nrm\nwinux\n");

    // test `--list` for `--platform windows` by itself
    testenv
        .command()
        .args(["--platform", "windows", "--list"])
        .assert()
        .success()
        .stdout("common\ndel\ndir\nwinux\n");

    // test `--list` for `--platform linux --platform windows`
    testenv
        .command()
        .args(["--platform", "linux", "--platform", "windows", "--list"])
        .assert()
        .success()
        .stdout("common\ndel\ndir\nls\nrm\nwinux\n");

    // test `--list` for `--platform windows --platform linux`
    testenv
        .command()
        .args(["--platform", "linux", "--platform", "windows", "--list"])
        .assert()
        .success()
        .stdout("common\ndel\ndir\nls\nrm\nwinux\n");
}

#[cfg_attr(feature = "ignore-online-tests", ignore = "online test")]
#[test]
fn test_autoupdate_cache() {
    let testenv = TestEnv::new();

    // The first time, if automatic updates are disabled, the cache should not be found
    testenv
        .command()
        .args(["--list"])
        .assert()
        .failure()
        .stderr(contains("Page cache not found. Please run `tldr --update`"));

    let config_file_path = testenv.config_dir.path().join("config.toml");
    let cache_file_path = testenv.cache_dir.path().join(TLDR_PAGES_DIR);

    // Activate automatic updates, set the auto-update interval to 24 hours
    let mut config_file = File::create(config_file_path).unwrap();
    config_file
        .write_all(b"[updates]\nauto_update = true\nauto_update_interval_hours = 24")
        .unwrap();
    config_file.flush().unwrap();

    // Helper function that runs `tldr --list` and asserts that the cache is automatically updated
    // or not, depending on the value of `expected`.
    let check_cache_updated = |expected| {
        let assert = testenv.command().args(["--list"]).assert().success();
        let pred = contains("Successfully updated cache");
        if expected {
            assert.stderr(pred)
        } else {
            assert.stderr(pred.not())
        };
    };

    // The cache is updated the first time we run `tldr --list`
    check_cache_updated(true);

    // The cache is not updated with a subsequent call
    check_cache_updated(false);

    // We update the modification and access times such that they are about 23 hours from now.
    // auto-update interval is 24 hours, the cache should not be updated
    let new_mtime = SystemTime::now() - Duration::from_secs(82_800);
    filetime::set_file_mtime(&cache_file_path, new_mtime.into()).unwrap();
    check_cache_updated(false);

    // We update the modification and access times such that they are about 25 hours from now.
    // auto-update interval is 24 hours, the cache should be updated
    let new_mtime = SystemTime::now() - Duration::from_secs(90_000);
    filetime::set_file_mtime(&cache_file_path, new_mtime.into()).unwrap();
    check_cache_updated(true);

    // The cache is not updated with a subsequent call
    check_cache_updated(false);
}

/// End-end test to ensure .page.md files overwrite pages in cache_dir
#[test]
fn test_custom_page_overwrites() {
    let testenv = TestEnv::new().write_custom_pages_config();

    // Add file that should be ignored to the cache dir
    testenv.add_entry("inkscape-v2", "");
    // Add .page.md file to custom_pages_dir
    testenv.add_page_entry(
        "inkscape-v2",
        include_str!("cache/pages/common/inkscape-v2.md"),
    );

    // Load expected output
    let expected = include_str!("rendered/inkscape-default-no-color.expected");

    testenv
        .command()
        .args(["inkscape-v2", "--color", "never"])
        .assert()
        .success()
        .stdout(diff(expected));
}

/// End-End test to ensure that .patch.md files are appended to pages in the cache_dir
#[test]
fn test_custom_patch_appends_to_common() {
    let testenv = TestEnv::new()
        .install_default_cache()
        .install_default_custom_pages();

    // Load expected output
    let expected = include_str!("rendered/inkscape-patched-no-color.expected");

    testenv
        .command()
        .args(["inkscape-v2", "--color", "never"])
        .assert()
        .success()
        .stdout(diff(expected));
}

/// End-End test to ensure that .patch.md files are not appended to .page.md files in the custom_pages_dir
/// Maybe this interaction should change but I put this test here for the coverage
#[test]
fn test_custom_patch_does_not_append_to_custom() {
    let testenv = TestEnv::new()
        .install_default_cache()
        .install_default_custom_pages();

    // In addition to the page in the cache, add the same page as a custom page.
    testenv.add_page_entry(
        "inkscape-v2",
        include_str!("cache/pages/common/inkscape-v2.md"),
    );

    // Load expected output
    let expected = include_str!("rendered/inkscape-default-no-color.expected");

    testenv
        .command()
        .args(["inkscape-v2", "--color", "never"])
        .assert()
        .success()
        .stdout(diff(expected));
}

#[test]
#[cfg(target_os = "windows")]
fn test_pager_warning() {
    let testenv = TestEnv::new().install_default_cache();

    // Regular call should not show a "pager flag not available on windows" warning
    testenv
        .command()
        .args(["which"])
        .assert()
        .success()
        .stderr(contains("pager flag not available on Windows").not());

    // But it should be shown if the pager flag is true
    testenv
        .command()
        .args(["--pager", "which"])
        .assert()
        .success()
        .stderr(contains("pager flag not available on Windows"));
}

/// Ensure that page lookup is case insensitive, so a page lookup for `eyed3`
/// and `eyeD3` should return the same page.
#[test]
fn test_lowercased_page_lookup() {
    let testenv = TestEnv::new();

    // Lookup `eyed3`, initially fails
    testenv.command().args(["eyed3"]).assert().failure();

    // Add entry
    testenv.add_entry("eyed3", "contents");

    // Lookup `eyed3` again
    testenv.command().args(["eyed3"]).assert().success();

    // Lookup `eyeD3`, should succeed as well
    testenv.command().args(["eyeD3"]).assert().success();
}

/// Regression test for #219: It should be possible to combine `--raw` and `-f`.
#[test]
fn test_raw_render_file() {
    let testenv = TestEnv::new().install_default_cache();

    let path = testenv
        .cache_dir
        .path()
        .join(TLDR_PAGES_DIR)
        .join("pages/common/inkscape-v1.md");
    let mut args = vec!["--color", "never", "-f", &path.to_str().unwrap()];

    // Default render
    testenv
        .command()
        .args(&args)
        .assert()
        .success()
        .stdout(diff(include_str!(
            "rendered/inkscape-default-no-color.expected"
        )));

    // Raw render
    args.push("--raw");
    testenv
        .command()
        .args(&args)
        .assert()
        .success()
        .stdout(diff(include_str!("cache/pages/common/inkscape-v1.md")));
}
