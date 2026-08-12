#!/bin/sh

set -ex

wget -O mdbook.tar.gz https://github.com/rust-lang/mdBook/releases/download/v0.5.4/mdbook-v0.5.4-x86_64-unknown-linux-musl.tar.gz
echo "5222beabd3e37dc5be0d18ff99b79058469354db5c220153a1b92db5ba12be89  mdbook.tar.gz" > sha256sums
sha256sum --check sha256sums
tar xvf mdbook.tar.gz
