# tldr

> A fast tldr client written in Rust.
> More information: <https://tealdeer-rs.github.io/tealdeer/>.

- Show the tldr page for a command:

`tldr {{command}}`

- Show the tldr page for a subcommand:

`tldr {{command}} {{subcommand}}`

- Show the tldr page for a specific platform:

`tldr --platform {{android|common|freebsd|linux|netbsd|openbsd|osx|sunos|windows}} {{command}}`

- Show the tldr page in a specific language:

`tldr --language {{language_code}} {{command}}`

- Download or update the local page cache:

`tldr --update`

- List all pages in the cache:

`tldr --list`

- Render a local markdown file as a tldr page:

`tldr --render {{path/to/file.md}}`

- Show the raw markdown source of a page instead of rendering it:

`tldr --raw {{command}}`

- Show file and directory paths used by tealdeer:

`tldr --show-paths`

- Create an initial config file:

`tldr --seed-config`

- Open a custom page for a command in `$EDITOR` (creates it if it doesn't exist):

`tldr --edit-page {{command}}`

- Open a custom patch for a command in `$EDITOR` (appended to the existing page):

`tldr --edit-patch {{command}}`

- Clear the local cache:

`tldr --clear-cache`
