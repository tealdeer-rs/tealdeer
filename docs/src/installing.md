# Installing

There are a few different ways to install Tealdeer:

- Through [package managers](#package-managers)
- Through [static binaries](#static-binaries-linux)
- Through [cargo install](#through-cargo-install)
- By [building from source](#build-from-source)

Additionally, when not using system packages, you can [manually install
autocompletions](#autocompletion).

## Package Managers

Tealdeer has been added to a few package managers:

- Arch Linux: [`tealdeer`](https://archlinux.org/packages/community/x86_64/tealdeer/)
- Fedora: [`tealdeer`](https://src.fedoraproject.org/rpms/rust-tealdeer)
- FreeBSD: [`sysutils/tealdeer`](https://www.freshports.org/sysutils/tealdeer/)
- Funtoo: [`app-misc/tealdeer`](https://github.com/funtoo/core-kit/tree/1.4-release/app-misc/tealdeer)
- Homebrew: [`tealdeer`](https://formulae.brew.sh/formula/tealdeer)
- MacPorts: [`tealdeer`](https://ports.macports.org/port/tealdeer/)
- NetBSD: [`sysutils/tealdeer`](https://pkgsrc.se/sysutils/tealdeer)
- Nix: [`tealdeer`](https://nixos.org/nixos/packages.html#tealdeer)
- openSUSE: [`tealdeer`](https://software.opensuse.org/package/tealdeer?search_term=tealdeer)
- Scoop: [`tealdeer`](https://github.com/ScoopInstaller/Main/blob/master/bucket/tealdeer.json)
- Solus: [`tealdeer`](https://packages.getsol.us/shannon/t/tealdeer/)
- Void Linux: [`tealdeer`](https://github.com/void-linux/void-packages/tree/master/srcpkgs/tealdeer)

## Static Binaries (Linux)

Static binary builds (currently for Linux only) are available on the
[GitHub releases page](https://github.com/dbrgn/tealdeer/releases).
Simply download the binary for your platform and run it!

## Through `cargo install`

Build and install the tool via cargo...

    $ cargo install tealdeer

*(Note: You might need to install OpenSSL development headers, otherwise you get
a "failed to run custom build command for openssl-sys" error message. The
package is called `libssl-dev` on Ubuntu.)*

## Build From Source

Debug build with logging enabled:

    $ cargo build --features logging

Release build without logging:

    $ cargo build --release

To enable the log output, set the `RUST_LOG` env variable:

    $ export RUST_LOG=tldr=debug

## Autocompletion

Shell completion scripts are located in the folder `completion`. 
Just copy them to their designated location:

- *Bash*: `cp completion/bash_tealdeer /usr/share/bash-completion/completions/tldr`
- *Fish*: `cp completion/fish_tealdeer ~/.config/fish/completions/tldr.fish`
- *Zsh*: `cp completion/zsh_tealdeer /usr/share/zsh/site-functions/_tldr`
