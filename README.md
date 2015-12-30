# tldr-rs

[![Build status](https://img.shields.io/travis/dbrgn/tldr-rs/master.svg)](https://travis-ci.org/dbrgn/tldr-rs)

An implementation of [tldr](https://github.com/tldr-pages/tldr) in Rust.

## Building

Debug build with logging enabled:

    $ cargo build --features logging

Release build without logging:

    $ cargo build --release

To enable the log output, set the `RUST_LOG` env variable:

    $ export RUST_LOG=tldr=debug

## License

MIT
