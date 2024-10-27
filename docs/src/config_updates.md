# Section: \[updates\]

## Automatic updates

Tealdeer can refresh the cache automatically when it is outdated. This
behavior can be configured in the `updates` section and is disabled by
default.

### `auto_update`

Specifies whether the auto-update feature should be enabled (defaults to
`false`).

    [updates]
    auto_update = true

### `auto_update_interval_hours`

Duration, since the last cache update, after which the cache will be
refreshed (defaults to 720 hours). This parameter is ignored if `auto_update`
is set to `false`.

    [updates]
    auto_update = true
    auto_update_interval_hours = 24

### `tls_backend`

An advance option. Specifies which TLS backend to use. Only modify this if you encounter certificate errors.

Available options:
- `native-roots` - Rustls with native roots
- `webpki-roots` - Rustls with WebPK roots
- `native-tls` - Native TLS
  - SChannel on Windows
  - Secure Transport on macOS
  - OpenSSL on other platforms

    [updates]
    tls_backend = "native-tls"
