//! Functions related to formatting and printing lines from a `Tokenizer`.

use log::debug;

use crate::{extensions::FindFrom, types::LineType};

#[derive(Debug, Clone, Copy, Eq)]
/// Represents a snippet from a page of a specific highlighting class.
pub enum PageSnippet<T> {
    CommandName(T),
    Variable(T),
    NormalCode(T),
    Description(T),
    Text(T),
    Title(T),
    Linebreak,
}

#[cfg_attr(not(test), allow(dead_code))]
impl<T> PageSnippet<T> {
    pub fn map<F, U>(self, f: F) -> PageSnippet<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            PageSnippet::CommandName(s) => PageSnippet::CommandName(f(s)),
            PageSnippet::Variable(s) => PageSnippet::Variable(f(s)),
            PageSnippet::NormalCode(s) => PageSnippet::NormalCode(f(s)),
            PageSnippet::Description(s) => PageSnippet::Description(f(s)),
            PageSnippet::Text(s) => PageSnippet::Text(f(s)),
            PageSnippet::Title(s) => PageSnippet::Title(f(s)),
            PageSnippet::Linebreak => PageSnippet::Linebreak,
        }
    }
}

impl<T: PartialEq<U>, U> PartialEq<PageSnippet<U>> for PageSnippet<T> {
    fn eq(&self, other: &PageSnippet<U>) -> bool {
        match (self, other) {
            (PageSnippet::CommandName(s), PageSnippet::CommandName(t))
            | (PageSnippet::Variable(s), PageSnippet::Variable(t))
            | (PageSnippet::NormalCode(s), PageSnippet::NormalCode(t))
            | (PageSnippet::Description(s), PageSnippet::Description(t))
            | (PageSnippet::Text(s), PageSnippet::Text(t))
            | (PageSnippet::Title(s), PageSnippet::Title(t)) => s == t,
            (PageSnippet::Linebreak, PageSnippet::Linebreak) => true,
            _ => false,
        }
    }
}

impl PageSnippet<&str> {
    pub fn is_empty(&self) -> bool {
        use PageSnippet::*;

        match self {
            CommandName(s) | Variable(s) | NormalCode(s) | Description(s) | Text(s) | Title(s) => {
                s.is_empty()
            }
            Linebreak => false,
        }
    }
}

/// Parse the content of each line yielded by `lines` and yield `HighLightingSnippet`s accordingly.
pub fn highlight_lines<L, F, E>(
    lines: L,
    process_snippet: &mut F,
    keep_empty_lines: bool,
    show_title: bool,
) -> Result<(), E>
where
    L: Iterator<Item = LineType>,
    F: for<'snip> FnMut(PageSnippet<&'snip str>) -> Result<(), E>,
{
    let mut command = String::new();
    for line in lines {
        match line {
            LineType::Empty => {
                if keep_empty_lines {
                    process_snippet(PageSnippet::Linebreak)?;
                }
            }
            LineType::Title(title) => {
                if show_title {
                    process_snippet(PageSnippet::Linebreak)?;
                    process_snippet(PageSnippet::Title(&title))?;
                } else {
                    debug!("Ignoring title");
                }
                // This is safe as long as the parsed title is only the command,
                // and the iterator yields values in order of appearance.
                command = title;
                debug!("Detected command name: {}", &command);
            }
            LineType::Description(text) => process_snippet(PageSnippet::Description(&text))?,
            LineType::ExampleText(text) => process_snippet(PageSnippet::Text(&text))?,
            LineType::ExampleCode(text) => {
                process_snippet(PageSnippet::NormalCode("      "))?;
                highlight_code(&command, text, process_snippet)?;
                process_snippet(PageSnippet::Linebreak)?;
            }

            LineType::Other(text) => debug!("Unknown line type: {text:?}"),
        }
    }
    process_snippet(PageSnippet::Linebreak)?;
    Ok(())
}

/// Highlight code examples including user variables in {{ curly braces }}.
fn highlight_code<E>(
    command: &str,
    mut text: String,
    process_snippet: &mut impl FnMut(PageSnippet<&str>) -> Result<(), E>,
) -> Result<(), E> {
    // NOTE: This is not optimal, as it allocates one String for each `replace`
    let replace_escaped = |s: &str| s.replace(r"\{\{", "{{").replace(r"\}\}", "}}");

    while !text.is_empty() {
        let Some(placeholder_start) = find_marker(&text, "{{", r"\{\{") else {
            return highlight_code_segment(command, &replace_escaped(&text), process_snippet);
        };
        let Some(mut placeholder_end) = find_marker(&text[placeholder_start + 2..], "}}", r"\}\}")
        else {
            return highlight_code_segment(command, &replace_escaped(&text), process_snippet);
        };
        placeholder_end += placeholder_start + 2;

        // Greedily extend matched range
        while placeholder_end + 2 < text.len() && text.as_bytes()[placeholder_end + 2] == b'}' {
            placeholder_end += 1;
        }

        let placeholder_content = &text[placeholder_start + 2..placeholder_end];

        if placeholder_start > 0 {
            highlight_code_segment(
                command,
                &replace_escaped(&text[..placeholder_start]),
                process_snippet,
            )?;
        }
        process_snippet(PageSnippet::Variable(&replace_escaped(placeholder_content)))?;

        text.replace_range(..placeholder_end + 2, "");
    }

    Ok(())
}

fn find_marker(s: &str, marker: &str, forbidden_prefix: &str) -> Option<usize> {
    assert_eq!(
        marker.as_bytes()[0],
        forbidden_prefix.as_bytes()[forbidden_prefix.len() - 1]
    );

    let mut search_start = 0;
    loop {
        let marker_index = s.find_from(marker, search_start)?;

        // Avoid matching the "{{" at the end of "\{\{{" (where "{{" is the marker, and "\{\{{"
        // is the forbidden prefix)
        let overlaps_with_prefix = (forbidden_prefix.len() <= marker_index + 1) && {
            let prefix_start = marker_index + 1 - forbidden_prefix.len();
            &s[prefix_start..=marker_index] == forbidden_prefix
        };
        if !overlaps_with_prefix {
            return Some(marker_index);
        }

        // The next valid marker cannot include the first character of the current match
        search_start = marker_index + 1;
    }
}

/// Yields `NormalCode` and `CommandName` in alternating order according to the occurrences of
/// `command_name` in `segment`. Variables are not detected here, see `highlight_code`
/// instead.
fn highlight_code_segment<'a, E>(
    command_name: &'a str,
    mut segment: &'a str,
    process_snippet: &mut impl FnMut(PageSnippet<&'a str>) -> Result<(), E>,
) -> Result<(), E> {
    if !command_name.is_empty() {
        let mut search_start = 0;
        while let Some(match_start) = segment.find_from(command_name, search_start) {
            let match_end = match_start + command_name.len();
            if is_freestanding_substring(segment, (match_start, match_end)) {
                process_snippet(PageSnippet::NormalCode(&segment[..match_start]))?;
                process_snippet(PageSnippet::CommandName(command_name))?;
                segment = &segment[match_end..];
                search_start = 0;
            } else {
                search_start = segment[match_start..]
                    .char_indices()
                    .nth(1)
                    .map_or(segment.len(), |(i, _)| match_start + i);
            }
        }
    }
    process_snippet(PageSnippet::NormalCode(segment))?;
    Ok(())
}

/// Checks whether the characters right before and after the substring (given by half-open index interval) are whitespace (if they exist).
fn is_freestanding_substring(surrounding: &str, substring: (usize, usize)) -> bool {
    let (start, end) = substring;
    // "okay" meaning <exists and is whitespace> or <doesn't exist>
    let char_before_is_okay = surrounding[..start]
        .chars()
        .last()
        .filter(|prev_char| !prev_char.is_whitespace())
        .is_none();
    let char_after_is_okay = surrounding[end..]
        .chars()
        .next()
        .filter(|next_char| !next_char.is_whitespace())
        .is_none();
    char_before_is_okay && char_after_is_okay
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_freestanding_substring() {
        assert!(is_freestanding_substring("I love tldr", (0, 1)));
        assert!(is_freestanding_substring("I love tldr", (2, 6)));
        assert!(is_freestanding_substring("I love tldr", (7, 11)));

        assert!(is_freestanding_substring("tldr", (0, 4)));
        assert!(is_freestanding_substring("tldr ", (0, 4)));
        assert!(is_freestanding_substring(" tldr", (1, 5)));
        assert!(is_freestanding_substring(" tldr ", (1, 5)));

        assert!(!is_freestanding_substring("tldr", (1, 3)));
        assert!(!is_freestanding_substring("tldr ", (1, 4)));
        assert!(!is_freestanding_substring(" tldr", (1, 4)));

        assert!(is_freestanding_substring(
            " épicé ",
            (1, " épicé".len()) // note the missing trailing space
        ));
        assert!(!is_freestanding_substring(
            " épicé ",
            (1, " épic".len()) // note the missing trailing space and character
        ));
    }

    fn run<'a>(cmd: &'a str, segment: &'a str) -> Vec<PageSnippet<String>> {
        let mut yielded = Vec::new();
        let mut process_snippet = |snip: PageSnippet<&str>| {
            if !snip.is_empty() {
                yielded.push(snip.map(str::to_string));
            }
            Ok::<(), ()>(())
        };

        highlight_code(cmd, segment.to_string(), &mut process_snippet)
            .expect("highlight code segment failed");
        yielded
    }

    mod highlight_code_segment {
        use super::*;
        use PageSnippet::*;

        #[test]
        fn test_highlight_code_segment() {
            assert!(run("make", "").is_empty());
            assert_eq!(
                &run("make", "make all CC=clang -q"),
                &[CommandName("make"), NormalCode(" all CC=clang -q")]
            );
            assert_eq!(
                &run("make", "  make money --always-make"),
                &[
                    NormalCode("  "),
                    CommandName("make"),
                    NormalCode(" money --always-make")
                ]
            );
            assert_eq!(
                &run("git commit", "git commit -m 'git commit'"),
                &[CommandName("git commit"), NormalCode(" -m 'git commit'"),]
            );
        }

        #[test]
        fn test_i18n() {
            assert_eq!(
                &run("mäke", "mäke höhlenrätselbücher"),
                &[CommandName("mäke"), NormalCode(" höhlenrätselbücher")]
            );
            assert_eq!(
                &run(
                    "Müll",
                    "1000 Gründe warum Müll heute größer ist als Müll früher, ärgerlich"
                ),
                &[
                    NormalCode("1000 Gründe warum "),
                    CommandName("Müll"),
                    NormalCode(" heute größer ist als "),
                    CommandName("Müll"),
                    NormalCode(" früher, ärgerlich")
                ]
            );
            assert_eq!(
                &run(
                    "übergang",
                    "die Zustandsübergangsfunktion übergang Änderungen",
                ),
                &[
                    NormalCode("die Zustandsübergangsfunktion "),
                    CommandName("übergang"),
                    NormalCode(" Änderungen")
                ],
            );
        }

        #[test]
        fn test_empty_command() {
            let segment = "some code";
            let snippets = [NormalCode(segment)];

            assert_eq!(run("", segment), snippets);
            assert_eq!(run(" ", segment), snippets);
            assert_eq!(run("  \t ", segment), snippets);
        }
    }

    mod placeholders {
        use super::*;
        use PageSnippet::*;

        #[test]
        fn variable_vs_escaped() {
            assert_eq!(
                run("ping", "ping {{example.com}}"),
                [
                    CommandName("ping"),
                    NormalCode(" "),
                    Variable("example.com"),
                ],
            );
            assert_eq!(
                run(
                    "docker inspect",
                    r"docker inspect --format '\{\{range.NetworkSettings.Networks\}\}\{\{.IPAddress\}\}\{\{end\}\}' {{container}}"
                ),
                [
                    CommandName("docker inspect"),
                    NormalCode(
                        " --format '{{range.NetworkSettings.Networks}}{{.IPAddress}}{{end}}' "
                    ),
                    Variable("container"),
                ],
            );
            assert_eq!(
                run("mount", r"mount \\{{computer_name}}\{{share_name}} Z:"),
                [
                    CommandName("mount"),
                    NormalCode(r" \\"),
                    Variable("computer_name"),
                    NormalCode(r"\"),
                    Variable("share_name"),
                    NormalCode(" Z:"),
                ],
            );

            assert_eq!(run("", r"\{"), [NormalCode(r"\{")]);
            assert_eq!(run("", r"\{{a"), [NormalCode(r"\{{a")]);
            assert_eq!(run("", r"\{{a}}"), [NormalCode(r"\"), Variable("a")]);

            // Placeholder has begin marker, but no end marker
            assert_eq!(run("", r"{{\}\}}"), [NormalCode("{{}}}")]);
        }

        #[test]
        fn outer_precedence() {
            assert_eq!(
                run("git stash", "git stash show --patch {{stash@{0}}}"),
                [
                    CommandName("git stash"),
                    NormalCode(" show --patch "),
                    Variable("stash@{0}"),
                ],
            );

            // The following is not listed in the specification, but this is the highlighting I would expect.
            assert_eq!(
                run("rg", "rg {{}}}"),
                [CommandName("rg"), NormalCode(" "), Variable("}")]
            );

            // And these are just to document the current behavior
            assert_eq!(run("", "{{{}}}"), [Variable("{}")]);
            assert_eq!(run("", "{{{{}}}"), [Variable("{{}")]);
            assert_eq!(run("", "{{{}}}}"), [Variable("{}}")]);
        }

        #[test]
        fn escaped_inside_placeholder() {
            assert_eq!(
                run(
                    "playerctl",
                    r#"playerctl metadata {{[-f|--format]}} "{{Now playing: \{\{artist\}\} - \{\{album\}\} - \{\{title\}\}}}""#
                ),
                [
                    CommandName("playerctl"),
                    NormalCode(" metadata "),
                    Variable("[-f|--format]"),
                    NormalCode(" \""),
                    Variable("Now playing: {{artist}} - {{album}} - {{title}}"),
                    NormalCode("\""),
                ],
            );
        }

        #[test]
        fn placeholder_inside_escaped() {
            assert_eq!(
                run("test", r#"test \{\{{{var}} normal\}\}"#),
                [
                    CommandName("test"),
                    NormalCode(" {{"),
                    Variable("var"),
                    NormalCode(" normal}}"),
                ],
            );
        }
    }
}
