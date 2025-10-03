# Section: \[search\]

This config section is used to configure the page search in the cache.
The settings apply to `tldr <page>` and `tldr --list`.

## `languages`

The list of languages that should be considered when searching.
If unspecified, the list of languages will be inferred from the `LANG` and `LANGUAGE` environment variables.
Either way, the language used can be overwritten using the `--language` command line flag.

```toml
[search]
# Show pages in German if available, otherwise show in English
languages = ["de", "en"]
```

## `platforms`

The list of platforms that should be considered when searching.
In addition to the platforms listed in the help text of the `--platform` flag, there are two special platforms available:
- `"current"`: equals the platform that tealdeer was compiled for
- `"all"`: adds all remaining platforms to the list

Tealdeer searches the platforms in order of appearance in this list.
The default list of platforms is `["current", "common", "all"]`.
The list of platforms can be overwritten using the `--platform` command line flag.

```toml
[search]
# Search for linux and common, and then search windows before trying the remaining platforms
platforms = ["linux", "common", "windows", "all"]
```
