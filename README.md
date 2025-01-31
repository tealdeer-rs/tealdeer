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
binaries on the [GitHub releases page](https://github.com/tealdeer-rs/tealdeer/releases/)!


## Docs (Installing, Usage, Configuration)

User documentation is available at <https://tealdeer-rs.github.io/tealdeer/>!

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

| Client (50 runs, 31.01.2025)      | Programming Language | Mean in ms | Deviation in ms | Comments                         |
| :---:                             | :---:                | :---:      | :---:           | :---:                            |
| [`outfieldr`][outfieldr-gh]       | Zig                  | ???        | ???             | lacks maintenance and features   |
| `tealdeer`                        | Rust                 | 2.0        | 0.4             |                                  |
| [`fast-tldr`][fast-tldr-gh]       | Haskell              | 2.9        | 0.4             | no example highlighting          |
| [`tldr-c`][c-gh]                  | C                    | 4.9        | 0.6             |                                  |
| [`tldr-hs`][hs-gh]                | Haskell              | 12.4       | 0.2             | no example highlighting          |
| [`tldr-python-client`][python-gh] | Python               | 59.3       | 2.5             |                                  |
| [`tldr-bash`][bash-gh]            | Bash                 | 100.5      | 7.1             |                                  |
| [`tldr-node-client`][node-gh]     | JavaScript / NodeJS  | 282.8      | 10.7            |                                  |


As you can see, `tealdeer` is one of the fastest of the tested clients.
However, we strive for useful features and code quality over raw performance, even if that means that we don't come out on top in this friendly competition.
That said, we are still optimizing the code and prioritizing maintainability.

For example, while `outfieldr` shows potential, it currently lacks maintenance and cannot even be compiled without downgrading `clang`, `llvm`, and other dependencies, which poses challenges for users on rolling-release distributions like Arch Linux.


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
[bash-gh]: https://github.com/raylee/tldr-sh-client
[outfieldr-gh]: https://gitlab.com/ve-nt/outfieldr
[python-gh]: https://github.com/tldr-pages/tldr-python-client

[benchmark-dockerfile]: https://github.com/tealdeer-rs/tealdeer/blob/main/benchmarks/Dockerfile
[client-spec]: https://github.com/tldr-pages/tldr/blob/main/CLIENT-SPECIFICATION.md
[hyperfine-gh]: https://github.com/sharkdp/hyperfine
[outfieldr-comment-tls]: https://github.com/tealdeer-rs/tealdeer/issues/129#issuecomment-833596765

<!-- Badges -->
[github-actions]: https://github.com/tealdeer-rs/tealdeer/actions?query=branch%3Amain
[github-actions-badge]: https://github.com/tealdeer-rs/tealdeer/actions/workflows/ci.yml/badge.svg?branch=main
[crates-io]: https://crates.io/crates/tealdeer
[crates-io-badge]: https://img.shields.io/crates/v/tealdeer.svg
