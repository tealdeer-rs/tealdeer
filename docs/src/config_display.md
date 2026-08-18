# Section: \[display\]

In the `display` section you can configure the output format.

## `use_pager`

Specifies whether the pager should be used by default or not (default `false`).

```toml
[display]
use_pager = true
```

When enabled, `less -R` is used as pager. To override the pager command used,
set the `PAGER` environment variable.

NOTE: This feature is not available on Windows.

## `compact`

Set this to enforce more compact output, where empty lines are stripped out
(default `false`).

```toml
[display]
compact = true
```

## `show_title`

Display the command name at the top of the page output (default `false`).

```toml
[display]
show_title = true
```

When enabled, the command name will be displayed at the top of the output,
styled with the `command_name` style configuration.

## `indent`

Controls the indentation of the output via two sub-keys.

### `indent.base`

Specifies the number of spaces used to indent descriptions, example text, and titles (default `2`).

```toml
[display.indent]
base = 2
```

### `indent.command`

Specifies the number of spaces used to indent example code lines (default `6`).

```toml
[display.indent]
command = 6
```

You can also configure both subkeys in a single line like this:

```toml
[display]
indent = {
  base = 2,
  command = 6,
}
```

## `placeholder_format`

Display the short and/or long variants of placeholders, if available.
Possible values: `"short"`, `"long"`, or `"both"` (default `"long"`).
This behavior can be overridden with the `--short-options` and `--long-options` flags.

```toml
[display]
# Display only short variants
placeholder_format = "short"
```

For example, when displaying the builtin page with `tldr tealdeer`, the `-f` / `--render` flag is displayed as follows:
- `-f`, if `placeholder_format = "short"`
- `--render`, if `placeholder_format = "long"`
- `[-f|--render]`, if `placeholder_format = "both"`
