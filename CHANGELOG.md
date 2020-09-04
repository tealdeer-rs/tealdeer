# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.


### [v1.4.1][v1.4.1] (2020-09-04)

- [fixed] Syntax error in zsh completion file ([#138][i138])

Contributors to this version:

- [Francesco][@BachoSeven]
- [Bruno A. Muciño][@mucinoab]

Thanks!


### [v1.4.0][v1.4.0] (2020-09-03)

- [added] Configurable automatic cache updates ([#115][i115])
- [added] Improved color detection and support for `--color` argument and
  `NO_COLOR` env variable ([#111][i111])
- [changed] Make `--list` option comply with official spec ([#112][i112])
- [changed] Move cache age warning to stderr ([#113][i113])

Contributors to this version:

- [Atul Bhosale][@Atul9]
- [Danny Mösch][@SimplyDanny]
- [Ilaï Deutel][@ilai-deutel]
- [Kornel][@kornelski]
- [@LovecraftianHorror][@LovecraftianHorror]
- [@michaeldel][@michaeldel]
- [Niklas Mohrin][@niklasmohrin]

Thanks!


### [v1.3.0][v1.3.0] (2020-02-28)

- [added] New config option for compact output mode ([#89][i89])
- [added] New -m/--markdown parameter for raw rendering ([#95][i95])
- [added] Provide zsh autocompletion ([#86][i86])
- [changed] Require at least Rust 1.39 to build (previous: 1.32)
- [changed] Switch to GitHub actions, CI testing now covers Windows as well ([#99][i99])
- [changed] Tweak the "outdated cache" warning message ([#97][i97])
- [changed] General maintenance: Upgrade dependencies, fix linter warnings
- [fixed] Fix Fish autocompletion on macOS ([#87][i87])
- [fixed] Fix compilation on Windows by disabling pager ([#99][i99])

Contributors to this version:

- [@Calinou][@Calinou]
- [@Delapouite][@Delapouite]
- [@james2doyle][@james2doyle]
- [@jesdazrez][@jesdazrez]
- [@korrat][@korrat]
- [@ma-renaud][@ma-renaud]
- [@Plommonsorbet][@Plommonsorbet]

Thanks!


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


[@aldanor]: https://github.com/aldanor
[@Atul9]: https://github.com/Atul9
[@BachoSeven]: https://github.com/BachoSeven
[@Bassets]: https://github.com/Bassets
[@Calinou]: https://github.com/Calinou
[@das-g]: https://github.com/das-g
[@Delapouite]: https://github.com/Delapouite
[@equal-l2]: https://github.com/equal-l2
[@ilai-deutel]: https://github.com/ilai-deutel
[@james2doyle]: https://github.com/james2doyle
[@jcgruenhage]: https://github.com/jcgruenhage
[@jdvr]: https://github.com/jdvr
[@jedahan]: https://github.com/jedahan
[@jesdazrez]: https://github.com/jesdazrez
[@kornelski]: https://github.com/kornelski
[@korrat]: https://github.com/korrat
[@LovecraftianHorror]: https://github.com/LovecraftianHorror
[@ma-renaud]: https://github.com/ma-renaud
[@michaeldel]: https://github.com/michaeldel
[@mucinoab]: https://github.com/mucinoab
[@mystal]: https://github.com/mystal
[@natpen]: https://github.com/natpen
[@niklasmohrin]: https://github.com/niklasmohrin
[@Plommonsorbet]: https://github.com/Plommonsorbet
[@SimplyDanny]: https://github.com/SimplyDanny
[@Voultapher]: https://github.com/Voultapher

[v1.0.0]: https://github.com/dbrgn/tealdeer/compare/v0.4.0...v1.0.0
[v1.1.0]: https://github.com/dbrgn/tealdeer/compare/v1.0.0...v1.1.0
[v1.2.0]: https://github.com/dbrgn/tealdeer/compare/v1.1.0...v1.2.0
[v1.3.0]: https://github.com/dbrgn/tealdeer/compare/v1.2.0...v1.3.0
[v1.4.0]: https://github.com/dbrgn/tealdeer/compare/v1.3.0...v1.4.0
[v1.4.1]: https://github.com/dbrgn/tealdeer/compare/v1.4.0...v1.4.1

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
[i71]: https://github.com/dbrgn/tealdeer/issues/71
[i75]: https://github.com/dbrgn/tealdeer/issues/75
[i77]: https://github.com/dbrgn/tealdeer/issues/77
[i84]: https://github.com/dbrgn/tealdeer/issues/84
[i86]: https://github.com/dbrgn/tealdeer/issues/86
[i87]: https://github.com/dbrgn/tealdeer/issues/87
[i89]: https://github.com/dbrgn/tealdeer/issues/89
[i95]: https://github.com/dbrgn/tealdeer/issues/95
[i97]: https://github.com/dbrgn/tealdeer/issues/97
[i99]: https://github.com/dbrgn/tealdeer/issues/99
[i111]: https://github.com/dbrgn/tealdeer/issues/111
[i112]: https://github.com/dbrgn/tealdeer/issues/112
[i113]: https://github.com/dbrgn/tealdeer/issues/113
[i115]: https://github.com/dbrgn/tealdeer/issues/115
[i138]: https://github.com/dbrgn/tealdeer/issues/138
