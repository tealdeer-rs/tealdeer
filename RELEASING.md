# Releasing

Run linting:

    $ cargo clean && cargo clippy

Set variables:

    $ export VERSION=X.Y.Z
    $ export GPG_KEY=20EE002D778AE197EF7D0D2CB993FF98A90C9AB1

Update version numbers:

    $ vim Cargo.toml
    $ cargo update -p tealdeer

Update docs:

    $ cargo run -- --help > docs/src/usage.txt

Update changelog:

    $ vim CHANGELOG.md

Commit & tag:

    $ git commit -S${GPG_KEY} -m "Release v${VERSION}"
    $ git tag -s -u ${GPG_KEY} v${VERSION} -m "Version ${VERSION}"

Publish:

    $ cargo publish
    $ git push && git push --tags

Then publish the release on GitHub.
