# Section: \[display\]

In the `display` section you can configure the output format.

## `use_pager`

Specifies whether the pager should be used by default or not (default `false`).

    [display]
    use_pager = true

When enabled, `less -R` is used as pager. To override the pager command used,
set the `PAGER` environment variable.

NOTE: This feature is not available on Windows.

## `compact`

Set this to enforce more compact output, where empty lines are stripped out
(default `false`).

    [display]
    compact = true

## `show_title`

Display the command name at the top of the page output (default `false`).

    [display]
    show_title = true

When enabled, the command name will be displayed at the top of the output,
styled with the `command_name` style configuration.