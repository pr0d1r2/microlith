//! The single command registry (V33): one table of verb, synopsis and
//! blurb, rendered two ways so `--help` and `cavespec docs` can never
//! disagree. `usage()` is the terse help; `markdown()` is the reference
//! block a test freezes into README. Add a verb HERE and both views update.
//!
//! It lives in the BINARY rather than the lib on purpose: everything the
//! lib exports becomes a promise a consumer compiles against (V32), and
//! nobody calling the rules needs the help text.
//!
//! GENERATED is the REFERENCE only. The narrative above README's block --
//! why the line cap exists, what the guarantees are -- stays hand-written,
//! because prose answering "why" is not derivable from a table.

/// One command's docs: its name, argument synopsis, and what it does.
struct Command {
    name: &'static str,
    synopsis: &'static str,
    blurb: &'static str,
}

/// Every verb, in the order `--help` and the reference both present them:
/// the two that gate first, then the one that rewrites, then the reports.
const COMMANDS: &[Command] = &[
    Command {
        name: "fmt",
        synopsis: "fmt [--check] [--verbose] [<path>]",
        blurb: "One line per statement: joins hard wraps and enforces the line cap. Rewrites the file; `--check` reports drift and exits 1 instead. The transform is proven whitespace-only before any write. `--verbose` confirms what was examined, with the longest line against the cap.",
    },
    Command {
        name: "check",
        synopsis: "check [--records <file>] [--format human|json] [--verbose] [<path>]",
        blurb: "The structural rules: sections present and ordered, ids unique, citations resolve, rows sorted, every task in exactly one milestone, every status one of `.` `~` `x`. Each violation carries a line and a ranked fix, marked mechanical (safe to apply unattended) or judgment (needs a human). `--records` adds the rejected-option check, whose baseline the caller owns because survival is a claim about edits rather than about the file. `--verbose` confirms what was examined, and says when the records check did not run.",
    },
    Command {
        name: "migrate",
        synopsis: "migrate [--check] [--verbose] [<path>]",
        blurb: "Section headers to canonical 4.1.0. A case or punctuation difference is rewritten silently; a label carrying real text is rewritten with the original kept beneath it, so nothing is discarded. Every alphanumeric run of the original is proven to survive before any write. A letter used for a DIFFERENT concept is never touched -- annotating one keeps the characters and inverts the meaning -- so those are reported and exit 1. `--check` reports without writing.",
    },
    Command {
        name: "derive",
        synopsis: "derive [--verbose] [<path>]",
        blurb: "Sizes, the citation graph, invariants cited by nothing, and statements said twice. Report-only: exits 0 even with findings, because an orphan is a question for a reader, not a build failure. `--verbose` adds every statement's size, biggest first: what to cut.",
    },
    Command {
        name: "anchors",
        synopsis: "anchors [--verbose] [<path>]",
        blurb: "The section address of every item, with the id it resolves to and whether the two have drifted apart. Report-only. `--verbose` prints each item in full, not a 60-char gist.",
    },
    Command {
        name: "docs",
        synopsis: "docs",
        blurb: "Print this command reference as markdown -- the source for README's generated block, kept in sync by a test. Report-only: it writes to stdout, never to the README.",
    },
];

/// What opens `--help`.
const HEAD: &str = "\
cavespec -- the cavekit SPEC format, enforced

usage: cavespec <command> [args]

commands:
";

/// What closes `--help`, up to the list of verbs this binary carries.
const FOOT: &str = "\
<path> defaults to SPEC.md, the one file FORMAT.md says every
cavekit command reads. Run from a project root and omit it.

built in this binary: ";

/// The exit line, in the terse rendering.
const EXIT_LINE: &str = "exit: 0 ok | 1 drift or violation | 2 usage";

/// The exit codes, as the table the reference block carries.
const EXIT: &str = "\
| code | meaning |
|------|---------|
| `0` | clean |
| `1` | drift, or a violation the command gates on |
| `2` | usage error |";

/// What opens the reference block.
const INTRO: &str = "\
## Commands

Every verb, its synopsis and what it does. Regenerate with `cavespec docs`.

";

/// The column the terse rendering wraps at, and the indent its prose sits
/// under. 76 leaves room in an 80-column terminal for the two characters a
/// shell prompt or a pager gutter takes.
const WIDTH: usize = 76;
const INDENT: &str = "      ";

/// The terse `--help` text, rendered from the registry (V33).
pub(crate) fn usage() -> String {
    let body: String = COMMANDS.iter().map(terse).collect();
    format!("{HEAD}{body}{FOOT}{}\n\n{EXIT_LINE}\n", names().join(", "))
}

/// Every verb the registry carries, in order.
pub(crate) fn names() -> Vec<&'static str> {
    COMMANDS.iter().map(|c| c.name).collect()
}

/// One command in the terse rendering: synopsis, wrapped prose, blank line.
fn terse(c: &Command) -> String {
    format!("  {}\n{}\n", c.synopsis, wrap(c.blurb, INDENT, WIDTH))
}

/// The markdown command reference (`cavespec docs`), frozen into README.
/// This IS the block between README's `cavespec docs` markers.
pub(crate) fn markdown() -> String {
    markdown_from(COMMANDS)
}

/// Rendered from a GIVEN registry, so the freeze can be shown to fail on a
/// registry that differs from the committed block (V18).
fn markdown_from(commands: &[Command]) -> String {
    let body: String = commands.iter().map(section).collect();
    format!("{INTRO}{body}## Exit codes\n\n{EXIT}\n")
}

/// One command in the reference: a heading, the synopsis fenced as text so
/// no bracket is read as markup, then the prose unwrapped.
fn section(c: &Command) -> String {
    format!(
        "### `{}`\n\n```text\n{}\n```\n\n{}\n\n",
        c.name, c.synopsis, c.blurb
    )
}

/// Wrap `text` to `width` columns, each line prefixed by `indent`.
///
/// The help text is the first output a user sees, so it is laid out for a
/// narrow terminal rather than run to whatever width the source happened to
/// have. Wrapping HERE rather than in the literal is what lets one blurb
/// serve both renderings: the reference wants it whole, help wants it
/// folded, and a hand-folded literal can only give the second.
fn wrap(text: &str, indent: &str, width: usize) -> String {
    let room = width.saturating_sub(indent.chars().count());
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        push_word(&mut lines, word, room);
    }
    lines.iter().map(|l| format!("{indent}{l}\n")).collect()
}

/// Add `word` to the last line if it still fits, else open a new one.
fn push_word(lines: &mut Vec<String>, word: &str, room: usize) {
    match lines.last_mut().filter(|l| fits(l.as_str(), word, room)) {
        Some(line) => {
            line.push(' ');
            line.push_str(word);
        }
        None => lines.push(word.to_owned()),
    }
}

/// Whether `word`, plus the space before it, still fits on `line`.
fn fits(line: &str, word: &str, room: usize) -> bool {
    line.chars()
        .count()
        .saturating_add(word.chars().count())
        .saturating_add(1)
        <= room
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The text between README's markers, exclusive of the marker lines.
    fn readme_block() -> String {
        let p =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
        let full = std::fs::read_to_string(p).unwrap_or_default();
        let after = full.split_once("<!-- BEGIN cavespec docs -->\n");
        let inner =
            after.and_then(|(_, r)| r.split_once("<!-- END cavespec docs -->"));
        inner.map(|(b, _)| b.to_owned()).unwrap_or_default()
    }

    /// The freeze (V33): README's generated block IS `cavespec docs` output.
    /// Drift -- a hand-edit, or a verb added without regenerating -- fails
    /// here rather than being merely regenerable by someone who noticed.
    #[test]
    fn the_readme_block_matches_the_registry() {
        assert_eq!(
            readme_block().trim(),
            markdown().trim(),
            "README block is stale -- regenerate: `cavespec docs`"
        );
    }

    /// V18: the guard is proven by PLANTING the drift it exists to catch.
    /// A flag added to the registry and not to README must go RED, or the
    /// freeze above is indistinguishable from one that cannot fail.
    #[test]
    fn a_flag_added_to_the_registry_alone_makes_the_freeze_red() {
        let planted = &[Command {
            name: "fmt",
            synopsis: "fmt [--check] [--verbose] [--undocumented] [<path>]",
            blurb: "A flag the README has never heard of.",
        }];
        assert_ne!(
            markdown_from(planted).trim(),
            readme_block().trim(),
            "the freeze accepted a registry the README does not carry"
        );
    }

    /// ...and the companion: the same comparison ACCEPTS the real registry,
    /// so it cannot pass by rejecting everything (V18).
    #[test]
    fn the_freeze_accepts_the_registry_it_was_generated_from() {
        assert_eq!(markdown_from(COMMANDS), markdown());
    }

    #[test]
    fn the_reference_carries_every_verb_and_the_exit_codes() {
        let m = markdown();
        assert!(m.starts_with("## Commands"), "{m}");
        assert!(m.contains("## Exit codes"), "{m}");
        for c in COMMANDS {
            let heading = format!("### `{}`", c.name);
            assert!(m.contains(&heading), "no section for {}", c.name);
        }
    }

    #[test]
    fn the_help_carries_every_verb() {
        let u = usage();
        assert!(u.contains("usage: cavespec"), "{u}");
        for c in COMMANDS {
            assert!(u.contains(c.synopsis), "help missing {}", c.name);
        }
    }

    /// The wrap is what lets ONE blurb serve both renderings, so it has to
    /// hold the column the help promises -- and lose no word doing it.
    #[test]
    fn wrapping_keeps_the_column_and_every_word() {
        let text = "one two three four five six seven eight nine ten";
        let out = wrap(text, "  ", 20);
        for line in out.lines() {
            assert!(line.len() <= 20, "{line:?} is {} wide", line.len());
        }
        let back: Vec<&str> = out.split_whitespace().collect();
        assert_eq!(back.join(" "), text, "a word was lost in the wrap");
    }

    /// A word longer than the column gets its own line rather than an empty
    /// one before it -- the loop that would otherwise never place it.
    #[test]
    fn a_word_wider_than_the_column_still_lands() {
        let out = wrap("supercalifragilistic ok", "", 5);
        assert_eq!(out, "supercalifragilistic\nok\n");
    }
}
