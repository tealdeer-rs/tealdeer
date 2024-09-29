# Tips and Tricks

This page features some example use cases of Tealdeer.

## Showing a random page on shell start

To display a randomly selected page, you can invoke `tldr` twice: One time to
select a page and a second time to display this page. To randomly select a page,
we use `shuf` from the GNU coreutils:

```bash
tldr --quiet $(tldr --quiet --list | shuf -n1)
```

You can also add the above command to your `.bashrc` (or similar shell
configuration file) to display a random page every time you start a new shell
session.

## Displaying all pages with their summary

If you want to extend the output of `tldr --list` with the first line summary of
each page, you can run the following Python script:

```python
#!/usr/bin/env python3

import subprocess

commands = subprocess.run(
    ["tldr", "--quiet", "--list"],
    capture_output=True,
    encoding="utf-8",
).stdout.splitlines()

for command in commands:
    output = subprocess.run(
        ["tldr", "--quiet", command],
        capture_output=True,
        encoding="utf-8",
    ).stdout
    description = output.lstrip().split("\n\n")[0]
    description = " ".join(description.split())
    print(f"{command} => {description}")
```

Note that there are a lot of pages and the script will run Tealdeer once for
every page, so the script may take a couple of seconds to finish.

## Extending this chapter

If you have an interesting setup with Tealdeer, feel free to share your
configuration on [our Github repository](https://github.com/tealdeer-rs/tealdeer).
