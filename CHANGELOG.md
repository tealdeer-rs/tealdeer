# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.


### [v1.2.0][v1.2.0] (2019-08-10)

- [added] Add Windows support ([#77][i77])
- [added] Add support for spaces in commands ([#75][i75])
- [added] Add support for Fish-based autocompletion ([#71][i71])
- [added] Add pager support ([#44][i44])
- [added] Print detected OS with `-v` / `--version` ([#57][i57])
- [changed] OS detection: Treat BSDs as "osx" ([#58][i58])
- [changed] Move from curl to reqwest ([#61][i61])
- [changed] Move to Rust 2018, require Rust 1.32 ([#69][i69] / [#84][i84])
- [fixed] Add (back) support for proxies ([#68][i68])

Contributors to this version:

- [@aldanor][@aldanor]
- [@Bassets][@Bassets]
- [@das-g][@das-g]
- [@jcgruenhage][@jcgruenhage]
- [@jdvr][@jdvr]
- [@jedahan][@jedahan]
- [@mystal][@mystal]
- [@natpen][@natpen]

Thanks!


### [v1.1.0][v1.1.0] (2018-10-22)

- [added] Configuration file support ([#43][i43])
- [added] Allow configuration of colors/style ([#43][i43])
- [added] New `--quiet` / `-q` option to suppress most non-error messages ([#48][i48])
- [changed] Require at least Rust 1.28 to build (previous: 1.19)
- [fixed] Fix building on systems with openssl 1.1.1 ([#47][i47])

Contributors to this version:

- [@equal-l2][@equal-l2]
- [Jonathan Dahan][@jedahan]
- [Lukas Bergdoll][@Voultapher]

Thanks!


### [v1.0.0][v1.0.0] (2018-02-11)

- [added] Include bash completions ([#34][i34])
- [changed] Update all dependencies
- [changed] Require at least Rust 1.19 to build (previous: 1.9)
- [changed] Improved unit/integration testing


### v0.4.0 (2016-11-25)

- [added] Support for new page format
- [changed] Update all dependencies


### v0.3.0 (2016-08-01)

- [changed] Update curl dependency


### v0.2.0 (2016-04-16)

- First crates.io release


[@equal-l2]: https://github.com/equal-l2
[@jedahan]: https://github.com/jedahan
[@Voultapher]: https://github.com/Voultapher

[v1.0.0]: https://github.com/dbrgn/tealdeer/compare/v0.4.0...v1.0.0
[v1.1.0]: https://github.com/dbrgn/tealdeer/compare/v1.0.0...v1.1.0
[v1.2.0]: https://github.com/dbrgn/tealdeer/compare/v1.1.0...v1.2.0

[i34]: https://github.com/dbrgn/tealdeer/issues/34
[i43]: https://github.com/dbrgn/tealdeer/issues/43
[i44]: https://github.com/dbrgn/tealdeer/issues/44
[i47]: https://github.com/dbrgn/tealdeer/issues/47
[i48]: https://github.com/dbrgn/tealdeer/issues/48
[i57]: https://github.com/dbrgn/tealdeer/issues/57
[i58]: https://github.com/dbrgn/tealdeer/issues/58
[i61]: https://github.com/dbrgn/tealdeer/issues/61
[i68]: https://github.com/dbrgn/tealdeer/issues/68
[i69]: https://github.com/dbrgn/tealdeer/issues/69
[i75]: https://github.com/dbrgn/tealdeer/issues/75
[i77]: https://github.com/dbrgn/tealdeer/issues/77
[i84]: https://github.com/dbrgn/tealdeer/issues/84
