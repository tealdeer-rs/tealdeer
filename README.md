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
- [x] Command line interface similar or equivalent to the [NodeJS client][tldr-node-client]
- [x] Be fast

A tool like `tldr` should be as frictionless as possible to use. It should be
easy to invoke (just `tldr tar`, not using another subcommand like `tldr find
tar`) and it should show the output as fast as possible.

tealdeer reaches these goals. During a (highly non-scientific) test (see
[#38](https://github.com/dbrgn/tealdeer/issues/38) for details), I tested the
invocation speed of `tldr <command>` for a few of the existing clients:

| Client | Times (ms) | Avg of 5 (ms) |
| --- | --- | --- |
| [Tealdeer](https://github.com/dbrgn/tealdeer/) | `15/11/5/5/11` | `9.4` (100%) |
| [C client](https://github.com/tldr-pages/tldr-cpp-client) | `11/5/12/11/15` | `10.8` (115%) |
| [Bash client](https://github.com/pepa65/tldr-bash-client) | `15/19/22/25/24` | `21.0` (223%) |
| [Go client by k3mist](https://github.com/k3mist/tldr/) | `98/96/100/95/101` | `98.8` (1'051%) |
| [Python client](https://github.com/lord63/tldr.py) | `152/148/151/158/140` | `149.8` (1'594%) |
| [NodeJS client](https://github.com/tldr-pages/tldr-node-client) | `169/171/170/170/170` | `170.0` (1'809%) |

tealdeer was the winner here, although the C client and the Bash client are in
the same speed class. Interpreted languages are clearly much slower to invoke,
a delay of 170 milliseconds is definitely noticeable and increases friction for
the user.

These are the clients I tried but failed to compile or run:
[Haskell client](https://github.com/psibi/tldr-hs),
[Ruby client](https://github.com/YellowApple/tldrb),
[Perl client](https://github.com/skaji/perl-tldr),
[Go client by anoopengineer](https://github.com/anoopengineer/tldr/),
[PHP client](https://github.com/BrainMaestro/tldr-php).


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

Tealdeer will not bump the MSRV requirement in patch versions, but it may
increase it in minor versions. The reason is that many important libraries
(e.g. the Tokio ecosystem, which is a dependency of reqwest, which is used for
downloading the cache) do not follow a static MSRV, but instead follow a
"stable + last n releases" approach. Trying to guarantee the same MSRV across
all minor releases would be a futile attempt.


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

Thanks to @SShrike for coming up with the name "tealdeer"!


[tldr-node-client]: https://github.com/tldr-pages/tldr-node-client

<!-- Badges -->
[github-actions]: https://github.com/dbrgn/tealdeer/actions?query=branch%3Amaster
[github-actions-badge]: https://github.com/dbrgn/tealdeer/workflows/CI/badge.svg
[crates-io]: https://crates.io/crates/tealdeer
[crates-io-badge]: https://img.shields.io/crates/v/tealdeer.svg
