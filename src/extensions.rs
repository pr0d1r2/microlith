//! V40: the letters this copy ADDS to the vendored format, as a document.
//!
//! `FORMAT.md` is upstream's and ships byte-identical (V8); `§V` is the
//! superset this tool actually enforces. Until now the difference between
//! the two lived in `§V`'s prose and three `const`s, so a reader who opened
//! the vendored file met seven sections and nothing said otherwise -- and a
//! cavekit user with no microlith could not adopt the extensions at all.
//! An extension nobody outside can READ is a fork wearing a superset's
//! clothes.
//!
//! So this renders the delta as a document that travels WITHOUT the tool:
//! the full section order marking which letters are upstream's and which are
//! ours, each extension's canonical word, what it holds, and an example.
//!
//! RENDERED, NOT WRITTEN -- V33's shape on a third generated claim, after
//! the command reference and the README badges. The letters and their rank
//! come from `check::SECTIONS`, which is the list the checker itself reads
//! (V7), and the registry below carries only what a `const` cannot derive:
//! what a section HOLDS, and an example of it. A freeze test makes staleness
//! red rather than merely regenerable.

use crate::check::{CANONICAL_WORDS, KINDS, SECTIONS, UPSTREAM_KINDS};

/// What one extension section holds, and what it looks like.
///
/// The letter is the key, and it is checked against `SECTIONS` in both
/// directions: an entry naming no known letter fails, and a letter with no
/// entry fails. That is what makes "the next letter cannot land
/// undocumented" true rather than aspirational -- the same bidirectional
/// guard T22's construct corpus carries, for the same reason.
struct Extension {
    letter: char,
    holds: &'static str,
    example: &'static str,
}

const EXTENSIONS: &[Extension] = &[
    Extension {
        letter: 'F',
        holds: "The edges this directory DECLARES. A spec federated over a \
                directory tree is one file per directory, so each one names \
                the tree it belongs to rather than restating it. Bullets, \
                no ids -- an edge is a fact about the file, not an \
                addressable item. A citation that crosses an edge names the \
                file it crosses to, because a bare `\u{a7}V.2` addresses \
                THIS spec and nothing else.",
        example: "- up: `../SPEC.md` -- the parent this spec refines.\n\
                  - down: `worker/SPEC.md`, `store/SPEC.md`.\n\
                  - `worker/SPEC.md \u{a7}V.2` -- how a citation crosses.",
    },
    Extension {
        letter: 'N',
        holds: "The DERIVED half: what a reader follows to reach a \
                neighbour. Written down rather than computed at read time, \
                so a spec read on its own still says where it sits. \
                Independent of the section above -- a leaf declares no edges \
                and still has neighbours -- so neither implies the other.",
        example: "- parent: [../SPEC.md](../SPEC.md)\n\
                  - siblings: [../api/SPEC.md](../api/SPEC.md)",
    },
];

/// The letters `SECTIONS` carries that the vendored format does not.
///
/// COMPUTED rather than listed (V7). Adding a letter to `KINDS` makes it an
/// extension here the same moment, which is what forces the document to
/// describe it.
pub(crate) fn extension_letters() -> Vec<char> {
    KINDS
        .into_iter()
        .filter(|k| !UPSTREAM_KINDS.contains(k))
        .collect()
}

/// The canonical header for a letter, as `SECTIONS` spells it.
fn header_for(letter: char) -> Option<&'static str> {
    KINDS
        .into_iter()
        .zip(SECTIONS)
        .find(|(k, _)| *k == letter)
        .map(|(_, h)| h)
}

/// The word V27 holds this letter's header to.
fn word_for(letter: char) -> Option<&'static str> {
    CANONICAL_WORDS
        .into_iter()
        .find(|(k, _)| *k == letter)
        .map(|(_, w)| w)
}

/// The full order, marking whose each letter is. This is the half a reader
/// needs first: an extension means nothing until you know WHERE it sits.
fn order() -> String {
    let ours = extension_letters();
    let mut out = String::from(
        "| # | section | defined by |\n|---|---------|------------|\n",
    );
    for (i, (letter, header)) in KINDS.into_iter().zip(SECTIONS).enumerate() {
        let n = i.saturating_add(1);
        let source = if ours.contains(&letter) {
            "**this document**"
        } else {
            "cavekit `FORMAT.md`"
        };
        out.push_str(&format!("| {n} | `{header}` | {source} |\n"));
    }
    out
}

/// `federation` -> `Federation`, for the spelling examples.
///
/// Chars rather than a byte slice: the canonical words are ASCII today and
/// nothing says the next one is.
fn title(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// One extension's own section: its header, its word, what it holds, an
/// example.
fn section(ext: &Extension) -> String {
    let header = header_for(ext.letter).unwrap_or_default();
    let word = word_for(ext.letter).unwrap_or_default();
    let cased = title(word);
    format!(
        "### `{header}`\n\nThe header must carry **`{word}`** -- matched as \
         a stem, case-insensitively, with qualifiers free to follow. \
         `{header}`, `## \u{a7}{letter} {cased}` and `## \u{a7}{letter} \
         \u{2014} {cased}` all name it.\n\n{}\n\n```\n{header}\n{}\n```\n\n",
        ext.holds,
        ext.example,
        letter = ext.letter,
    )
}

/// The document (`mth extensions`), frozen into `FORMAT-EXTENSIONS.md`.
pub(crate) fn markdown() -> String {
    markdown_from(EXTENSIONS)
}

/// Rendered from a GIVEN registry, so the freeze can be shown to fail on a
/// registry that differs from the committed file (V18).
fn markdown_from(exts: &[Extension]) -> String {
    let body: String = exts.iter().map(section).collect();
    format!("{HEAD}{}\n{MIDDLE}{body}{FOOT}", order())
}

const HEAD: &str = "\
# SPEC.md FORMAT -- EXTENSIONS

Cavekit's `FORMAT.md` fixes seven sections. This document is the DELTA a
project may add on top of it, and nothing here needs a particular tool to be
true: a spec that follows it is still a cavekit spec, read by hand or by
whatever you already run.

The vendored `FORMAT.md` beside this file is upstream's, unmodified. See
`.format-upstream` for the exact revision it matches. This file never edits
it -- it says what is ADDED, so the difference between the two is a thing a
reader can find rather than a thing they have to notice.

**Every extension section is OPTIONAL.** A spec carrying none of them is
unchanged, and must stay that way: an extension that makes existing specs
fail is a fork, whatever it is called.

## SECTION ORDER

Fixed order, fixed headers, addressable -- the same rule upstream states. A
section may be absent, but is never reordered.

";

const MIDDLE: &str = "\
## THE EXTENSIONS

";

const FOOT: &str = "\
## ADDRESSING

`\u{a7}<S>.<n>` addresses item `n` of section `S`, exactly as upstream
defines it -- `\u{a7}F.2` is the second edge declared.

This is the whole reason a letter must mean ONE thing everywhere. A citation
resolves against the READER's section set, not the writer's, so two projects
spelling one letter for two concepts break every citation that travels
between them -- silently, because both files still parse.

## ADOPTING THESE

Copy this file next to your `FORMAT.md` and write the sections. Nothing else
is required: the letters are ordinary cavekit sections that upstream has no
opinion about yet, and a spec that uses them is legal to every reader that
tolerates an unknown letter -- which upstream's own rule requires.

If upstream adopts a letter, it stops being an extension and leaves this
document. That is the intended end state, not a defeat: this file exists to
record what one copy leads with, and a shorter one means the lead was taken.

## THIS FILE IS GENERATED

Rendered from the same constants the checker reads, and frozen by a test --
so it cannot describe a format the tool does not enforce. Regenerate with
`mth extensions`; never edit it by hand.
";

#[cfg(test)]
mod tests {
    use super::*;

    /// The committed document IS the render (V33's freeze, third use).
    #[test]
    fn the_committed_document_matches_the_registry() {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("FORMAT-EXTENSIONS.md");
        let committed = std::fs::read_to_string(p).unwrap_or_default();
        assert_eq!(
            committed.trim(),
            markdown().trim(),
            "FORMAT-EXTENSIONS.md is stale -- regenerate: `mth extensions`"
        );
    }

    /// V18: the freeze is proven by PLANTING the drift it exists to catch.
    /// A registry that differs from the committed file must go RED, or the
    /// test above is indistinguishable from one that cannot fail.
    #[test]
    fn a_changed_registry_makes_the_freeze_red() {
        let planted = &[Extension {
            letter: 'F',
            holds: "something else entirely",
            example: "- nothing like the committed one",
        }];
        assert_ne!(markdown_from(planted), markdown());
    }

    /// BIDIRECTIONAL, and this is the half that matters: a letter added to
    /// `SECTIONS` with no entry here fails, so the NEXT extension cannot
    /// land undocumented the way `\u{a7}F` and `\u{a7}N` nearly did.
    #[test]
    fn every_extension_letter_has_an_entry() {
        let missing: Vec<char> = extension_letters()
            .into_iter()
            .filter(|l| !EXTENSIONS.iter().any(|e| e.letter == *l))
            .collect();
        assert_eq!(missing, Vec::<char>::new(), "letters with no entry");
    }

    /// ...and the other half: an entry naming a letter the format does not
    /// carry describes a section nobody can write.
    #[test]
    fn every_entry_names_an_extension_letter() {
        let letters = extension_letters();
        let orphans: Vec<char> = EXTENSIONS
            .iter()
            .map(|e| e.letter)
            .filter(|l| !letters.contains(l))
            .collect();
        assert_eq!(orphans, Vec::<char>::new(), "entries with no letter");
    }

    /// The extension set is COMPUTED, so this pins what it computes to
    /// today -- and a letter promoted upstream must change it here.
    #[test]
    fn the_extensions_are_the_letters_upstream_lacks() {
        assert_eq!(extension_letters(), vec!['F', 'N']);
    }

    /// Every rendered section names its canonical word, because that is the
    /// one thing an adopter has to get right for a citation to travel.
    #[test]
    fn each_section_names_the_word_v27_holds_it_to() {
        let out = markdown();
        assert!(out.contains("`federation`"), "{out}");
        assert!(out.contains("`nav`"), "{out}");
    }

    /// The order table says whose each letter is, both ways round. A table
    /// that marked everything ours, or nothing, would render and say
    /// nothing.
    #[test]
    fn the_order_marks_upstream_and_ours_apart() {
        let table = order();
        assert!(table.contains("| `## \u{a7}G GOAL` | cavekit"), "{table}");
        assert!(
            table.contains("| `## \u{a7}F FEDERATION` | **this document**"),
            "{table}"
        );
    }
}
