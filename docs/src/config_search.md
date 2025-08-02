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
