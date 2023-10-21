# tealdeer

![teal deer](docs/src/deer.png)

|Crate|CI (Linux/macOS/Windows)|
|:---:|:---:|
|[![Crates.io][crates-io-badge]][crates-io]|[![GitHub CI][github-actions-badge]][github-actions]|

A very fast implementation of [tldr](https://github.com/tldr-pages/tldr) in
Rust: Simplified, example based and community-driven man pages.

<img src="docs/src/screenshot-default.png" alt="Screenshot of tldr command" width="600">

If you pronounce "tldr" in English, it sounds somewhat like "tealdeer". Hence the project name :)

In case you're in a hurry and just want to quickly try tealdeer, you can find static
binaries on the [GitHub releases page](https://github.com/dbrgn/tealdeer/releases/)!


## Docs (Installing, Usage, Configuration)

User documentation is available at <https://dbrgn.github.io/tealdeer/>!

The docs are generated using [mdbook](https://rust-lang.github.io/mdBook/index.html).
They can be edited through the markdown files in the `docs/src/` directory.


## Goals

High level project goals:

- [x] Download and cache pages
- [x] Don't require a network connection for anything besides updating the cache
- [x] Command line interface similar or equivalent to the [NodeJS client][node-gh]
- [x] Comply with the [tldr client specification][client-spec]
- [x] Advanced highlighting and configuration
- [x] Be fast

A tool like `tldr` should be as frictionless as possible to use and show the
output as fast as possible.

We think that `tealdeer` reaches these goals. We put together a (more or less)
reproducible benchmark that compiles a handful of clients from source and
measures the execution times on a cold disk cache. The benchmarking is run in a
Docker container using sharkdp's [`hyperfine`][hyperfine-gh]
([Dockerfile][benchmark-dockerfile]).

| Client (50 runs, 17.10.2021)      | Programming Language | Mean in ms | Deviation in ms | Comments                |
| :---:                             | :---:                | :---:      | :---:           | :---:                   |
| [`outfieldr`][outfieldr-gh]       | Zig                  | 9.1        | 0.5             | no user configuration   |
| `tealdeer`                        | Rust                 | 13.2       | 0.5             |                         |
| [`fast-tldr`][fast-tldr-gh]       | Haskell              | 17.0       | 0.6             | no example highlighting |
| [`tldr-hs`][hs-gh]                | Haskell              | 25.1       | 0.5             | no example highlighting |
| [`tldr-bash`][bash-gh]            | Bash                 | 30.0       | 0.8             |                         |
| [`tldr-c`][c-gh]                  | C                    | 38.4       | 1.0             |                         |
| [`tldr-python-client`][python-gh] | Python               | 87.0       | 2.4             |                         |
| [`tldr-node-client`][node-gh]     | JavaScript / NodeJS  | 407.1      | 12.9            |                         |

As you can see, `tealdeer` is one of the fastest of the tested clients.
However, we strive for useful features and code quality over raw performance,
even if that means that we don't come out on top in this friendly competition.
That said, we are still optimizing the code, for example when the `outfieldr`
developers [suggested to switch][outfieldr-comment-tls] to a native TLS
implementation instead of the native libraries.

## Development

Creating a debug build with logging enabled:

    $ cargo build --features logging

Release build without logging:

    $ cargo build --release

To enable the log output, set the `RUST_LOG` env variable:

    $ export RUST_LOG=tldr=debug

To run tests:

    $ cargo test

To run lints:

    $ rustup component add clippy
    $ cargo clean && cargo clippy


## MSRV (Minimally Supported Rust Version)

When publishing a tealdeer release, the Rust version required to build it
should be stable for at least a month.


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

Thanks to @severen for coming up with the name "tealdeer"!


[node-gh]: https://github.com/tldr-pages/tldr-node-client
[c-gh]: https://github.com/tldr-pages/tldr-c-client
[hs-gh]: https://github.com/psibi/tldr-hs
[fast-tldr-gh]: https://github.com/gutjuri/fast-tldr
[bash-gh]: https://4e4.win/tldr
[outfieldr-gh]: https://gitlab.com/ve-nt/outfieldr
[python-gh]: https://github.com/tldr-pages/tldr-python-client

[benchmark-dockerfile]: https://github.com/dbrgn/tealdeer/blob/main/benchmarks/Dockerfile
[client-spec]: https://github.com/tldr-pages/tldr/blob/main/CLIENT-SPECIFICATION.md
[hyperfine-gh]: https://github.com/sharkdp/hyperfine
[outfieldr-comment-tls]: https://github.com/dbrgn/tealdeer/issues/129#issuecomment-833596765

<!-- Badges -->
[github-actions]: https://github.com/dbrgn/tealdeer/actions?query=branch%3Amain
[github-actions-badge]: https://github.com/dbrgn/tealdeer/actions/workflows/ci.yml/badge.svg?branch=main
[crates-io]: https://crates.io/crates/tealdeer
[crates-io-badge]: https://img.shields.io/crates/v/tealdeer.svg
