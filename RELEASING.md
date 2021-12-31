# Releasing

Run linting:

    $ cargo clean && cargo clippy

Set variables:

    $ export VERSION=X.Y.Z
    $ export GPG_KEY=EA456E8BAF0109429583EED83578F667F2F3A5FA

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
