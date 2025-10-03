//! Functions related to formatting and printing lines from a `Tokenizer`.

use log::debug;

use crate::{extensions::FindFrom, types::LineType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Represents a snippet from a page of a specific highlighting class.
pub enum PageSnippet<'a> {
    CommandName(&'a str),
    Variable(&'a str),
    NormalCode(&'a str),
    Description(&'a str),
    Text(&'a str),
    Title(&'a str),
    Linebreak,
}

impl PageSnippet<'_> {
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
    F: for<'snip> FnMut(PageSnippet<'snip>) -> Result<(), E>,
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
                highlight_code(&command, &text, process_snippet)?;
                process_snippet(PageSnippet::Linebreak)?;
            }

            LineType::Other(text) => debug!("Unknown line type: {text:?}"),
        }
    }
    process_snippet(PageSnippet::Linebreak)?;
    Ok(())
}

/// Highlight code examples including user variables in {{ curly braces }}.
fn highlight_code<'a, E>(
    command: &'a str,
    text: &'a str,
    process_snippet: &mut impl FnMut(PageSnippet<'a>) -> Result<(), E>,
) -> Result<(), E> {
    let variable_splits = text
        .split("}}")
        .map(|s| s.split_once("{{").unwrap_or((s, "")));
    for (code_segment, variable) in variable_splits {
        highlight_code_segment(command, code_segment, process_snippet)?;
        process_snippet(PageSnippet::Variable(variable))?;
    }
    Ok(())
}

/// Yields `NormalCode` and `CommandName` in alternating order according to the occurrences of
/// `command_name` in `segment`. Variables are not detected here, see `highlight_code`
/// instead.
fn highlight_code_segment<'a, E>(
    command_name: &'a str,
    mut segment: &'a str,
    process_snippet: &mut impl FnMut(PageSnippet<'a>) -> Result<(), E>,
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
    use PageSnippet::*;

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

    fn run<'a>(cmd: &'a str, segment: &'a str) -> Vec<PageSnippet<'a>> {
        let mut yielded = Vec::new();
        let mut process_snippet = |snip: PageSnippet<'a>| {
            if !snip.is_empty() {
                yielded.push(snip);
            }
            Ok::<(), ()>(())
        };

        highlight_code_segment(cmd, segment, &mut process_snippet)
            .expect("highlight code segment failed");
        yielded
    }

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
