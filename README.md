# tldr-rs

[![Build status](https://img.shields.io/travis/dbrgn/tldr-rs/master.svg)](https://travis-ci.org/dbrgn/tldr-rs)

An implementation of [tldr](https://github.com/tldr-pages/tldr) in Rust.

High level project goals:

- [x] Download and cache pages
- [x] Don't require a network connection for anything besides updating the cache
- [x] Command line interface similar or equivalent to the [NodeJS client][tldr-node-client]
- [x] Be fast

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

    Examples:

        $ tldr tar
        $ tldr --list

    To control the cache:

        $ tldr --update
        $ tldr --clear-cache

    To render a local file (for testing):

        $ tldr --render /path/to/file.md


## Installing

Build and install the tool from source (see section "Building" in this README).

If you're an Arch Linux user, you can also install the package from the AUR:

    $ yaourt -S tldr-rs-git


## Building

tldr-rs requires at least Rust 1.5.

Debug build with logging enabled:

    $ cargo build --features logging

Release build without logging:

    $ cargo build --release

To enable the log output, set the `RUST_LOG` env variable:

    $ export RUST_LOG=tldr=debug


## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.


[tldr-node-client]: https://github.com/tldr-pages/tldr-node-client
