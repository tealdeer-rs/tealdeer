# tealdeer

![teal deer](deer.png)

|Crate|CI (Linux/macOS/Windows)|
|:---:|:---:|
|[![Crates.io][crates-io-badge]][crates-io]|[![GitHub CI][github-actions-badge]][github-actions]|

A very fast implementation of [tldr](https://github.com/tldr-pages/tldr) in
Rust: Simplified, example based and community-driven man pages.

<img src="screenshot-default.png" alt="Screenshot of tldr command" width="600">

If you pronounce "tldr" in English, it sounds somewhat like "tealdeer". Hence the project name :)

In case you're in a hurry and just want to quickly try tealdeer, you can find static
binaries on the [GitHub releases page](https://github.com/dbrgn/tealdeer/releases/)!


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


## Usage

    tldr [options] <command>
    tldr [options]

    Options:

        -h --help            Show this screen
        -v --version         Show version information
        -l --list            List all commands in the cache
        -f --render <file>   Render a specific markdown file
        -o --os <type>       Override the operating system [linux, osx, sunos, windows]
        -u --update          Update the local cache
        -c --clear-cache     Clear the local cache
        -p --pager           Use a pager to page output
        -m --markdown        Display the raw markdown instead of rendering it
        -q --quiet           Suppress informational messages
        --config-path        Show config file path
        --seed-config        Create a basic config
        --color <when>       Control when to use color [always, auto, never] [default: auto]

    Examples:

        $ tldr tar
        $ tldr --list

    To control the cache:

        $ tldr --update
        $ tldr --clear-cache

    To render a local file (for testing):

        $ tldr --render /path/to/file.md


## Installing

### Static Binaries (Linux)

Static binary builds (currently for Linux only) are available on the
[GitHub releases page](https://github.com/dbrgn/tealdeer/releases).
Simply download the binary for your platform and run it!

Builds for other platforms are planned.

### Cargo Install (any platform)

Build and install the tool via cargo...

    $ cargo install tealdeer

*(Note: You might need to install OpenSSL development headers, otherwise you get
a "failed to run custom build command for openssl-sys" error message. The
package is called `libssl-dev` on Ubuntu.)*

### From Package Manager

tealdeer has been added to a few package managers:

- Arch Linux AUR: [`tealdeer`](https://aur.archlinux.org/packages/tealdeer/),
  [`tealdeer-bin`](https://aur.archlinux.org/packages/tealdeer-bin/) or
  [`tealdeer-git`](https://aur.archlinux.org/packages/tealdeer-git/)
- Fedora: [`tealdeer`](https://src.fedoraproject.org/rpms/rust-tealdeer)
- FreeBSD: [`sysutils/tealdeer`](https://www.freshports.org/sysutils/tealdeer/)
- macOS Homebrew: [`tealdeer`](https://formulae.brew.sh/formula/tealdeer)
- NetBSD: [`sysutils/tealdeer`](https://pkgsrc.se/sysutils/tealdeer)
- Nix: [`tealdeer`](https://nixos.org/nixos/packages.html#tealdeer)
- openSUSE: [`tealdeer`](https://software.opensuse.org/package/tealdeer?search_term=tealdeer)
- Solus: [`tealdeer`](https://packages.getsol.us/shannon/t/tealdeer/)
- Void Linux: [`tealdeer`](https://github.com/void-linux/void-packages/tree/master/srcpkgs/tealdeer)

### From Source (any platform)

tealdeer requires at least Rust 1.39.

Debug build with logging enabled:

    $ cargo build --features logging

Release build without logging:

    $ cargo build --release

To enable the log output, set the `RUST_LOG` env variable:

    $ export RUST_LOG=tldr=debug


## Configuration

The tldr command can be customized with a config file called `config.toml`.
Creating the config file can be done manually or with the help of tldr:

    $ tldr --seed-config

The configuration file path follows OS conventions. It can be queried with the following command:

    $ tldr --config-path

The directory where the configuration file resides may be overwritten by the
environment variable `TEALDEER_CONFIG_DIR`. Remember to use an absolute path.
Variable expansion will not be performed on the path.

### Style

Using the config file, the style (e.g. colors or underlines) can be customized.

Possible styles:

- `description`: The initial description text
- `command_name`: The command name as part of the example code
- `example_text`: The text that describes an example
- `example_code`: The example itself, except the `command_name` and `example_variable`
- `example_variable`: The variables in the example

Currently supported attributes:

- `foreground` (color string, ANSI code, or RGB, see below)
- `background` (color string, ANSI code, or RGB, see below)
- `underline` (`true` or `false`)
- `bold` (`true` or `false`)

Colors can be specified in one of three ways:

- Color string (`black`, `red`, `green`, `yellow`, `blue`, `purple`, `cyan`, `white`)
- 256 color ANSI code (e.g. `foreground = { ansi = 4 }`)
- 24-bit RGB color (e.g. `background = { rgb = { r = 255, g = 255, b = 255 } }`)

Example customization:

<img src="screenshot-custom.png" alt="Screenshot of customized version" width="600">

### Display

In the `display` section you can configure the output format.

#### `use_pager`

Specifies whether the pager should be used by default or not (default `false`).

    [display]
    use_pager = true

When enabled, `less -R` is used as pager. To override the pager command used,
set the `PAGER` environment variable.

NOTE: This feature is not available on Windows.

#### `compact`

Set this to enforce more compact output, where empty lines are stripped out
(default `false`).

    [display]
    compact = true


### Automatic updates

tealdeer can refresh the cache automatically when it is outdated. This
behavior can be configured in the `updates` section and is disabled by
default.

#### `auto_update`

Specifies whether the auto-update feature should be enabled (defaults to
`false`).

    [updates]
    auto_update = true

#### `auto_update_interval_hours`

Duration, since the last cache update, after which the cache will be
refreshed (defaults to 720 hours). This parameter is ignored if `auto_update`
is set to `false`.

    [updates]
    auto_update = true
    auto_update_interval_hours = 24


## Autocompletion

- *Bash*: copy `bash_tealdeer` to `/usr/share/bash-completion/completions/tldr`
- *Fish*: copy `fish_tealdeer` to `~/.config/fish/completions/tldr.fish`
- *Zsh*: copy `zsh_tealdeer` to `/usr/share/zsh/site-functions/_tldr`


## Development

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
