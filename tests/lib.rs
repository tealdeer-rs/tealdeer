//! Integration tests.

extern crate assert_cli;
extern crate tempdir;
extern crate utime;

use std::fs::File;
use std::io::Write;

use assert_cli::{Assert, Environment};
use tempdir::TempDir;

struct TestEnv {
    pub cache_dir: TempDir,
    pub config_dir: TempDir,
    pub input_dir: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        TestEnv {
            cache_dir: TempDir::new(".tldr.test.cache").unwrap(),
            config_dir: TempDir::new(".tldr.test.config").unwrap(),
            input_dir: TempDir::new(".tldr.test.input").unwrap(),
        }
    }

    /// Return a new [`Assert`](../assert_cli/struct.Assert.html) instance for
    /// the main binary with env vars set.
    fn assert(&self) -> Assert {
        let env = Environment::inherit()
            .insert("TEALDEER_CACHE_DIR", self.cache_dir.path().to_str().unwrap())
            .insert("TEALDEER_CONFIG_DIR", self.config_dir.path().to_str().unwrap());
        Assert::main_binary()
            .with_env(env)
    }
}

#[test]
fn test_missing_cache() {
    TestEnv::new()
        .assert()
        .with_args(&["sl"])
        .fails()
        .stderr().contains("Cache not found. Please run `tldr --update`.")
        .unwrap();
}

#[test]
fn test_update_cache() {
    let testenv = TestEnv::new();

    testenv
        .assert()
        .with_args(&["sl"])
        .fails()
        .stderr().contains("Cache not found. Please run `tldr --update`.")
        .unwrap();

    testenv
        .assert()
        .with_args(&["--update"])
        .succeeds()
        .stdout().contains("Successfully updated cache.")
        .unwrap();

    testenv
        .assert()
        .with_args(&["sl"])
        .succeeds()
        .unwrap();
}

#[test]
fn test_quiet_cache() {
    let testenv = TestEnv::new();
    testenv
        .assert()
        .with_args(&["--update", "--quiet"])
        .succeeds()
        .stdout().is("")
        .unwrap();

    testenv
        .assert()
        .with_args(&["--clear-cache", "--quiet"])
        .succeeds()
        .stdout().is("")
        .unwrap();
}

#[test]
fn test_quiet_failures() {
    let testenv = TestEnv::new();

    testenv
        .assert()
        .with_args(&["--update", "-q"])
        .succeeds()
        .stdout().is("")
        .unwrap();

    testenv
        .assert()
        .with_args(&["fakeprogram", "-q"])
        .fails()
        .stdout().is("")
        .unwrap();
}

#[test]
fn test_quiet_old_cache() {
    let testenv = TestEnv::new();

    testenv
        .assert()
        .with_args(&["--update", "-q"])
        .succeeds()
        .stdout().is("")
        .unwrap();

    let _ = utime::set_file_times(testenv.cache_dir.path().join("tldr-master"), 1, 1).unwrap();

    testenv
        .assert()
        .with_args(&["tldr"])
        .succeeds()
        .stdout().contains("Cache wasn't updated in ")
        .unwrap();

    testenv
        .assert()
        .with_args(&["tldr", "--quiet"])
        .succeeds()
        .stdout().doesnt_contain("Cache wasn't updated in ")
        .unwrap();
}

#[test]
fn test_setup_seed_config() {
    let testenv = TestEnv::new();

    testenv
        .assert()
        .with_args(&["--seed-config"])
        .succeeds()
        .stdout().contains("Successfully created seed config file")
        .unwrap();
}

fn _test_correct_rendering(input_file: &str, filename: &str) {
    let testenv = TestEnv::new();

    // Create input file
    let file_path = testenv.input_dir.path().join(filename);
    println!("Testfile path: {:?}", &file_path);
    let mut file = File::create(&file_path).unwrap();
    file.write_all(input_file.as_bytes()).unwrap();

    // Load expected output
    let expected = include_str!("inkscape-default.expected");

    testenv
        .assert()
        .with_args(&["-f", &file_path.to_str().unwrap()])
        .succeeds()
        .stdout().is(expected)
        .unwrap();
}

/// An end-to-end integration test for direct file rendering (v1 syntax).
#[test]
fn test_correct_rendering_v1() {
    _test_correct_rendering(include_str!("inkscape-v1.md"), "inkscape-v1.md");
}

/// An end-to-end integration test for direct file rendering (v2 syntax).
#[test]
fn test_correct_rendering_v2() {
    _test_correct_rendering(include_str!("inkscape-v2.md"), "inkscape-v2.md");
}

/// An end-to-end integration test for rendering with constom syntax config.
#[test]
fn test_correct_rendering_with_config() {
    let testenv = TestEnv::new();

    // Setup config file
    // TODO should be config::CONFIG_FILE_NAME
    let config_file_path = testenv.config_dir.path().join("config.toml");
    println!("Config path: {:?}", &config_file_path);

    let mut config_file = File::create(&config_file_path).unwrap();
    config_file
        .write(include_str!("config.toml").as_bytes())
        .unwrap();

    // Create input file
    let file_path = testenv.input_dir.path().join("inkscape-v2.md");
    println!("Testfile path: {:?}", &file_path);

    let mut file = File::create(&file_path).unwrap();
    file.write_all(include_str!("inkscape-v2.md").as_bytes()).unwrap();

    // Load expected output
    let expected = include_str!("inkscape-with-config.expected");

    testenv
        .assert()
        .with_args(&["-f", &file_path.to_str().unwrap()])
        .succeeds()
        .stdout().is(expected)
        .unwrap();
}
