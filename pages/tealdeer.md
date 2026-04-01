# tldr

> This is a builtin page that shows information for your installed tealdeer version.
> More information: <https://tealdeer-rs.github.io/tealdeer/>.

> This page shows tealdeer specific functionality. See tldr tldr for more examples.

- Render a local markdown file as a tldr page:

`tldr --render {{path/to/file.md}}`

- Show the raw markdown source of a page instead of rendering it:

`tldr --raw {{command}}`

- Show file and directory paths used by tealdeer:

`tldr --show-paths`

- Create an initial config file:

`tldr --seed-config`

- Override config file location:

`tldr --config-path <FILE>`

- Open a custom page for a command in `$EDITOR` (creates it if it doesn't exist):

`tldr --edit-page {{command}}`

- Open a custom patch for a command in `$EDITOR` (appended to the existing page):

`tldr --edit-patch {{command}}`

- Clear the local cache:

`tldr --clear-cache`

- If auto update is configured, disable it for this run:

`tldr --no-auto-update`
