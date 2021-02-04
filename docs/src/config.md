# Configuration

Tealdeer can be customized with a config file called `config.toml`.  Creating
the config file can be done manually or with the help of `tldr`:

    $ tldr --seed-config

The configuration file path follows OS conventions. It can be queried with the
following command:

    $ tldr --show-paths

On Linux, this will usually be `~/.config/tealdeer/config.toml`.

## Override Config Directory

The directory where the configuration file resides may be overwritten by the
environment variable `TEALDEER_CONFIG_DIR`. Remember to use an absolute path.
Variable expansion will not be performed on the path.

## Override Cache Directory

Similarly, the cache directory where the pages are downloaded to, also follows
OS conventions. On Linux, it will usually be at `~/.cache/tealdeer/`. The path
can be overwritten using the environment variable `TEALDEER_CACHE_DIR`.
Remember to use an absolute path. Variable expansion will not be performed on
the path.

## Config Example

```toml
[display]
compact = false
use_pager = true

[style.command_name]
foreground = "red"

[style.example_text]
foreground = "green"

[style.example_code]
foreground = "blue"

[style.example_variable]
foreground = "blue"
underline = true

[updates]
auto_update = true
```
