# Section: \[search\]

This config section is used to configure the page search in the cache.
The settings apply to `tldr <page>` and `tldr --list`.

## `languages`

The list of languages that should be considered when searching.
If unspecified, the list of languages will be inferred from the `LANG` and `LANGUAGE` environment variables.
Either way, the language used can be overwritten using the `--language` command line flag.

    [search]
    # Show pages in German if available, otherwise show in English
    languages = ["de", "en"]

## `try_all_platforms`

Whether or not all known platforms should be tried when searching for pages.
For example, pages for Windows would also be found when running on Linux.
This option is enabled by default.
Either way, the platforms used can be overwritten using the `--platform` command line flag.

    [search]
    # Don't use extra platforms in search
    try_all_platforms = false
