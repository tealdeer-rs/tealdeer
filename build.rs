use std::io::Error;

use clap::{App, IntoApp};
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, Zsh},
    Generator,
};

#[path = "src/cli.rs"]
mod cli;
#[path = "src/types.rs"]
mod types;

fn generate<T: Generator>(shell: T, app: &mut App) -> Result<(), Error> {
    println!(
        "cargo:warning=completion file {:?} is generated",
        generate_to(shell, app, "tldr", "completion")?
    );
    Ok(())
}

fn main() -> Result<(), Error> {
    if std::env::var("PROFILE").unwrap() == "release" {
        let mut app = cli::Args::into_app();

        generate(Bash, &mut app)?;
        generate(Fish, &mut app)?;
        generate(Zsh, &mut app)?;
    }

    Ok(())
}
