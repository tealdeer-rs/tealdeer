# Section: \[updates\]

This config section contains settings related to updating the tealdeer cache.

## Automatic updates

Tealdeer can refresh the cache automatically when it is outdated. This
behavior can be configured in the `updates` section and is disabled by
default.

### `auto_update`

Specifies whether the auto-update feature should be enabled (defaults to
`false`).

```toml
[updates]
auto_update = true
```

### `auto_update_interval_hours`

Duration, since the last cache update, after which the cache will be
refreshed (defaults to 720 hours). This parameter is ignored if `auto_update`
is set to `false`.

```toml
[updates]
auto_update = true
auto_update_interval_hours = 24
```

## Download configuration

### `download_languages`

The list of languages which should be downloaded when updating.
If unspecified, the languages listed in the `search.languages` setting are used.
Thus, this setting is the most useful to instruct tealdeer to download pages in additional languages that are not searched by default.
Either way, the language used can be overwritten using the `--language` command line flag.

```toml
[search]
languages = ["de", "en"]

[updates]
# sometimes I like to read the Italian description
download_languages = ["de", "en", "it"]
```

### `archive_source`

URL for the location of the tldr pages archive. By default the pages are
fetched from the latest `tldr-pages/tldr` GitHub release.

```toml
[updates]
archive_source = https://my-company.example.com/tldr/
```

### `tls_backend`

Specifies which TLS backend to use. Try changing this setting if you encounter certificate errors.

Available options:
- `rustls-with-native-roots` - [Rustls][rustls] (a TLS library in Rust) with native roots
- `rustls-with-webpki-roots` - Rustls with [WebPKI][rustls-webpki] roots
- `native-tls` - Native TLS
  - SChannel on Windows
  - Secure Transport on macOS
  - OpenSSL on other platforms

```toml
[updates]
tls_backend = "native-tls"
```

[rustls]: https://github.com/rustls/rustls
[rustls-webpki]: https://github.com/rustls/webpki
