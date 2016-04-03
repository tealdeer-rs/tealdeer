//! Integration tests.

extern crate tempdir;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use tempdir::TempDir;


struct TestEnv {
    cache_dir: TempDir,
    bin_path: PathBuf,
}

impl TestEnv {
    fn new() -> Self {
        // Initialize tempdir for cache
        let dir = TempDir::new(".tldr.test").unwrap();

        // Determine binary path
        let lib_path = env::current_exe().unwrap();
        let bin_dir = lib_path.parent().unwrap();
        let bin_path = bin_dir.join("tldr");

        TestEnv {
            cache_dir: dir,
            bin_path: bin_path,
        }
    }

    /// Return a new Command instance with the base binary and env vars set.
    fn cmd(&self) -> Command {
        let mut cmd = Command::new(&self.bin_path);
        cmd.env("TLDR_RS_CACHE_DIR", self.cache_dir.path());
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
