//! An implementation of [tldr](https://github.com/tldr-pages/tldr) in Rust.

#[macro_use] extern crate log;
#[cfg(feature = "logging")]extern crate env_logger;
extern crate docopt;
extern crate ansi_term;
extern crate flate2;
extern crate tar;
extern crate curl;
extern crate rustc_serialize;
extern crate time;

use std::io::BufReader;
use std::fs::File;
use std::process;

use docopt::Docopt;

mod types;
mod tokenizer;
mod formatter;
mod updater;
mod error;

use tokenizer::Tokenizer;
use updater::Updater;
use error::TldrError;
use formatter::print_lines;


const NAME: &'static str = "tldr-rs";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const USAGE: &'static str = "
Usage:

    tldr <command>
    tldr [options]

Options:

    -h --help           Show this screen
    -v --version        Show version information
    -l --list           List all commands in the cache
    -f --render <file>  Render a specific markdown file
    -o --os <type>      Override the operating system [linux, osx, sunos]
    -u --update         Update the local cache
    -c --clear-cache    Clear the local cache

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


#[derive(Debug, RustcDecodable)]
struct Args {
    arg_command: Option<String>,
    flag_help: bool,
    flag_version: bool,
    flag_list: bool,
    flag_render: Option<String>,
    flag_os: Option<String>,  // TODO enum
    flag_update: bool,
    flag_clear_cache: bool,
}


/// Open file, return a `BufRead` instance
fn get_file_reader(filepath: &str) -> Result<BufReader<File>, String> {
    let file = try!(
        File::open(filepath)
            .map_err(|msg| format!("Could not open file: {}", msg))
    );
    Ok(BufReader::new(file))
}


#[cfg(feature = "logging")]
fn init_log() {
    env_logger::init().unwrap();
}

#[cfg(not(feature = "logging"))]
fn init_log() { }


fn main() {

    // Initialize logger
    init_log();

    // Parse arguments
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    // Show version and exit
    if args.flag_version {
        println!("{} v{}", NAME, VERSION);
        process::exit(0);
    }

    // Initialize updater
    let dl = Updater::new(ARCHIVE_URL);

    // Clear cache, pass through
    if args.flag_clear_cache {
        println!("Flag --clear-cache not yet implemented.");
        process::exit(1);
    }

    // Update cache, pass through
    if args.flag_update {
        dl.update().unwrap_or_else(|e| {
            match e {
                TldrError::UpdateError(msg) => println!("Could not update cache: {}", msg),
            };
            process::exit(1);
        });
        println!("Successfully updated cache.");
    }

    // Render local file and exit
    if let Some(file) = args.flag_render {
        // Open file
        let reader = get_file_reader(&file).unwrap_or_else(|msg| {
            println!("{}", msg);
            process::exit(1);
        });

        // Create tokenizer and print output
        let mut tokenizer = Tokenizer::new(reader);
        print_lines(&mut tokenizer);

        process::exit(0);
    }

    // List cached commands and exit
    if args.flag_list {
        println!("Flag --list not yet implemented.");
        process::exit(1);
    }

    // Override OS and exit
    if let Some(os) = args.flag_os {
        println!("Flag --os not yet implemented.");
    }

    // Show command from cache
    if let Some(command) = args.arg_command {
        if !args.flag_update {
            match dl.last_update() {
                Some(ago) if ago > MAX_CACHE_AGE => {
                    println!("Cache wasn't updated in {} days.", MAX_CACHE_AGE / 24 / 3600);
                    println!("You should probably run `tldr --update` soon."); 
                },
                None => {
                    println!("Cache not found. Please run `tldr --update`.");
                    process::exit(1);
                },
                _ => {},
            }
        }
        println!("Page rendering from cache not yet implemented.");
        process::exit(1);
    }

    // Some flags can be run without a command.
    if !args.flag_update {
        println!("{}", USAGE);
        process::exit(1);
    }
}


#[cfg(test)]
mod test {
    use types::LineType;

    #[test]
    fn test_linetype_from_str() {
        assert_eq!(LineType::from(""), LineType::Empty);
        assert_eq!(LineType::from(" \n \r"), LineType::Empty);
        assert_eq!(LineType::from("# Hello there"), LineType::Title("Hello there".into()));
        assert_eq!(LineType::from("> tis a description \n"), LineType::Description("tis a description".into()));
        assert_eq!(LineType::from("- some command"), LineType::ExampleText("some command".into()));
        assert_eq!(LineType::from("`$ cargo run`"), LineType::ExampleCode("$ cargo run".into()));
        assert_eq!(LineType::from("`$ cargo run"), LineType::Other("`$ cargo run".into()));
        assert_eq!(LineType::from("jklö"), LineType::Other("jklö".into()));
    }
}
