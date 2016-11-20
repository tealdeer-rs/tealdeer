//! Integration tests.

extern crate tempdir;

use std::env;
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::PathBuf;
use std::process::Command;

use tempdir::TempDir;

struct TestEnv {
    cache_dir: TempDir,
    bin_path: PathBuf,
    pub tests_path: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        // Initialize tempdir for cache
        let dir = TempDir::new(".tldr.test").unwrap();

        // Determine binary path
        let lib_path = env::current_exe().unwrap();
        let bin_dir = lib_path.parent().unwrap();
        let bin_path = bin_dir.join("tldr");

        // Copy test files
        let tests_path = bin_dir.parent()
                                .and_then(|d| d.parent())
                                .map(|d| d.join("tests"))
                                .expect("Could not find tests directory path");

        TestEnv {
            cache_dir: dir,
            bin_path: bin_path,
            tests_path: tests_path,
        }
    }

    /// Return a new Command instance with the base binary and env vars set.
    fn cmd(&self) -> Command {
        let mut cmd = Command::new(&self.bin_path);
        cmd.env("TEALDEER_CACHE_DIR", self.cache_dir.path());
        cmd
    }
}

#[test]
fn test_missing_cache() {
    let testenv = TestEnv::new();

    let out = testenv.cmd()
                     .arg("sl")
                     .output()
                     .expect(&format!("Could not launch tldr binary ({:?})", &testenv.bin_path));
    assert_eq!(out.status.success(), false);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, "Cache not found. Please run `tldr --update`.\n");
}

#[test]
fn test_update_cache() {
    let testenv = TestEnv::new();

    let out1 = testenv.cmd()
                      .arg("sl")
                      .output()
                      .expect(&format!("Could not launch tldr binary ({:?})", &testenv.bin_path));
    assert_eq!(out1.status.success(), false);

    let out2 = testenv.cmd()
                      .arg("--update")
                      .output()
                      .expect(&format!("Could not launch tldr binary ({:?})", &testenv.bin_path));
    let stdout = String::from_utf8(out2.stdout).unwrap();
    println!("{}", stdout);
    assert_eq!(out2.status.success(), true);
    assert_eq!(stdout, "Successfully updated cache.\n");

    let out3 = testenv.cmd()
                      .arg("sl")
                      .output()
                      .expect(&format!("Could not launch tldr binary ({:?})", &testenv.bin_path));
    assert_eq!(out3.status.success(), true);
}

fn _test_correct_rendering(filename: &str) {
    let testenv = TestEnv::new();
    let testfile = testenv.tests_path.join(filename);
    let testfile_expected = testenv.tests_path.join("inkscape.expected");

    let out: Vec<u8> = testenv.cmd()
        .arg("-f").arg(testfile)
        .output()
        .expect(&format!("Could not launch tldr binary ({:?})", &testenv.bin_path))
        .stdout;

    let mut expected = Vec::<u8>::new();
    BufReader::new(File::open(testfile_expected).unwrap()).read_to_end(&mut expected).unwrap();

    assert_eq!(out, expected);
}

/// An end-to-end integration test for direct file rendering.
#[test]
fn test_correct_rendering_v1() {
    _test_correct_rendering("inkscape-v1.md");
}

/// An end-to-end integration test for direct file rendering.
#[test]
fn test_correct_rendering_v2() {
    _test_correct_rendering("inkscape-v2.md");
}
