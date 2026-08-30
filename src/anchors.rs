//! The `§S.n` addressing FORMAT.md defines.
//!
//! `§V.2` is "invariants section, item 2" -- an ORDINAL, counted from the
//! section header. That is FORMAT.md's wording and FORMAT.md is normative, so
//! it stands here whatever its drawbacks.
//!
//! It has one, and this module makes it visible rather than designing around
//! it. Ids may have GAPS (V12), and an ordinal SHIFTS when an earlier item is
//! deleted -- so `§V.13` can silently come to mean a different rule, which is
//! B2's failure one level out. Every address therefore prints the id it
//! currently resolves to, and a divergence line says when the two have parted
//! company. A reader quoting `§V.13` in a commit can see whether that address
//! is stable in this file or not.
//!
//! The prose sections -- `§G`, and the bulleted ones `§F`, `§N`, `§C` and
//! `§I` -- carry no ids at all, so they print `-`: there the ordinal is the
//! only address there is. That makes `§F.2` the ONLY way to cite an edge,
//! which is why B25 mattered more than its size suggested.

use crate::check::{ITEM_KINDS, KINDS, is_header_for};
use crate::id::at_line_start;

/// One addressable item.
#[derive(Debug, PartialEq, Eq)]
pub struct Anchor {
    /// `§V.2`, as FORMAT.md writes it.
    pub address: String,
    /// The id it resolves to, or `-` where the section has none.
    pub id: String,
    /// Enough of the item to recognise it.
    pub gist: String,
    /// The whole line, for `--verbose`.
    pub full: String,
}

impl Anchor {
    /// Whether the ordinal and the id number have parted company -- which
    /// happens exactly when an earlier id was skipped or retired.
    #[must_use]
    pub fn diverges(&self) -> bool {
        let Some((_, ordinal)) = self.address.split_once('.') else {
            return false;
        };
        match self.id.get(1..) {
            None => false,
            Some(num) => !num.is_empty() && num != ordinal,
        }
    }
}

/// The section letter a header names: `## §V INVARIANTS` -> `V`.
fn section_letter(header: &str) -> Option<char> {
    header.strip_prefix("## \u{a7}")?.chars().next()
}

/// Whether a line is an addressable ITEM rather than prose continuing one.
///
/// A bullet, or a line opening with an id. Deliberately NOT every non-blank
/// line, and deliberately not every table row:
///
/// * the table header and separator in `§T` are furniture;
/// * a `| M<n> |` MILESTONE row is furniture too, and that is the one worth
///   arguing about. It carries real content, so addressing it is defensible
///   -- but counting it would push every task's ordinal out of step with its
///   id, so `§T.4` would stop meaning T4 in the ordinary no-gap case. The
///   whole point of printing the id beside the address is to make drift
///   VISIBLE; manufacturing drift on every well-formed spec would defeat it.
///   Milestones already have names, `M1` through `M4`, and nothing cites
///   them by address.
///
/// The letters were LISTED here until B25, and that list was a fourth copy
/// of the section set -- so `\u{a7}F` and `\u{a7}N` landed in `KINDS`,
/// `SECTIONS` and `CANONICAL_WORDS` and were addressable by none of them.
/// DERIVED now: a section either declares ids (`ITEM_KINDS`, caught above) or
/// it is prose, and prose outside `\u{a7}G` is bulleted. The next letter is
/// addressable the day it is known, without anyone remembering this file.
fn is_item(line: &str, letter: char) -> bool {
    if at_line_start(line).is_some() {
        return true;
    }
    match letter {
        'G' => !line.trim().is_empty() && !line.starts_with("##"),
        l if !ITEM_KINDS.contains(&l) => line.starts_with("- "),
        _ => false,
    }
}

/// Every addressable item in the spec, in reading order.
#[must_use]
pub fn anchors(text: &str) -> Vec<Anchor> {
    let mut out = Vec::new();
    let mut section = None;
    let mut n = 0usize;
    for line in text.lines() {
        if let Some(next) = opens_section(line) {
            section = next;
            n = 0;
        } else if let Some(l) = section.filter(|l| is_item(line, *l)) {
            n = n.saturating_add(1);
            out.push(anchor(line, l, n));
        }
    }
    out
}

/// `Some(section)` when this line is a header: the section it opens, or
/// `None` inside it for a header that is not one of V11's letters, so stray
/// `##` prose cannot be addressed as if it were spec content.
///
/// Uses `check`'s header test rather than its own (V7). This matched the
/// SECTIONS strings whole until B9: T15 made `check` label-agnostic and left
/// this behind, so the two disagreed about what a section header even is --
/// and `anchors` returned NOTHING for a real fleet spec whose headers read
/// `## \u{a7}V — Invariants`. Two readings of one thing is the defect this
/// crate exists to end, and it grew back inside it in a single task.
fn opens_section(line: &str) -> Option<Option<char>> {
    let letter = section_letter(line)?;
    Some(
        KINDS
            .into_iter()
            .any(|k| is_header_for(line, k))
            .then_some(letter),
    )
}

fn anchor(line: &str, letter: char, n: usize) -> Anchor {
    Anchor {
        address: format!("\u{a7}{letter}.{n}"),
        id: at_line_start(line).map_or_else(|| "-".to_owned(), |i| i.label()),
        gist: gist(line),
        full: line.trim_end().to_owned(),
    }
}

/// The first 60 chars, so a listing stays scannable. Chars, not bytes, or a
/// cut could land inside a multi-byte symbol.
fn gist(line: &str) -> String {
    line.chars()
        .take(60)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

/// The listing, one item per line, plus what the addresses are worth.
#[must_use]
pub fn report(text: &str) -> String {
    listing(text, false)
}

/// The same, with each item's FULL text instead of a 60-char gist.
///
/// `anchors` already speaks, so verbose DEEPENS rather than confirms (§I).
/// The gist exists to keep a 30-line listing scannable; it is the wrong
/// trade the moment you are looking for the item that says a particular
/// thing, because a statement's distinguishing clause is rarely in its
/// first 60 characters -- V21 and V22 are identical for the first 40.
#[must_use]
pub fn report_verbose(text: &str) -> String {
    listing(text, true)
}

fn listing(text: &str, full: bool) -> String {
    let found = anchors(text);
    let mut out: String = found
        .iter()
        .map(|a| {
            let body = if full { &a.full } else { &a.gist };
            format!("{}\t{}\t{}\n", a.address, a.id, body)
        })
        .collect();
    out.push_str(&divergence(&found));
    out
}

fn divergence(found: &[Anchor]) -> String {
    let drifted: Vec<&Anchor> = found.iter().filter(|a| a.diverges()).collect();
    if drifted.is_empty() {
        return "stable: every ordinal matches its id -- no gaps\n".to_owned();
    }
    drifted
        .iter()
        .map(|a| {
            format!(
                "shifted {}: resolves to {} -- an earlier id was skipped, so this address moves when the section is edited\n",
                a.address, a.id
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "\
# spec

## \u{a7}G GOAL
one line of goal.

## \u{a7}C CONSTRAINTS
- first constraint
- second constraint

## \u{a7}I INTERFACES
- `cmd foo` -- does a thing

## \u{a7}V INVARIANTS
V1: **first.**
V2: **second.**

## \u{a7}T TASKS
| id | scope | tasks | done-when |
|----|-------|-------|-----------|
| M1 | first | T1 | done |
T1|x|a task|V1

## \u{a7}B BUGS
id|date|cause|fix
B1|2026-08-01|a cause|a fix
";

    /// B9: `anchors` and `check` must agree on what a section header IS.
    ///
    /// This returned NOTHING for a real fleet spec whose headers read
    /// `## §V — Invariants`, because it matched the SECTIONS strings whole
    /// while `check` had moved to the letter. A silent empty listing, from a
    /// verb whose whole job is to list.
    #[test]
    fn a_section_is_found_under_any_label() {
        let relabelled = SPEC
            .replace("## \u{a7}V INVARIANTS", "## \u{a7}V \u{2014} Invariants")
            .replace("## \u{a7}T TASKS", "## \u{a7}T Tasks");
        let got = addresses(&relabelled);
        assert!(got.contains(&"\u{a7}V.2".to_owned()), "{got:?}");
        assert!(got.contains(&"\u{a7}T.1".to_owned()), "{got:?}");
        assert_eq!(got.len(), addresses(SPEC).len(), "same items, new labels");
    }

    /// B25, and the guard that keeps it fixed: EVERY known letter is
    /// addressable, asked of `KINDS` rather than of a list written here.
    ///
    /// `§F` and `§N` landed in `KINDS`, `SECTIONS` and `CANONICAL_WORDS` and
    /// in none of `is_item`'s hand-listed letters, so their items had no
    /// address at all -- while V39's entire argument is that `§F.2` must
    /// resolve to one KIND of thing in every repo. The claim had no runner
    /// (V17) in the one verb that produces the addresses.
    ///
    /// Asked over `KINDS` on purpose: a test naming the letters would be the
    /// fifth copy of the set, and would pass tomorrow while the next letter
    /// went unaddressable exactly as these two did.
    #[test]
    fn every_known_letter_is_addressable() {
        for letter in KINDS {
            let body = match letter {
                l if ITEM_KINDS.contains(&l) => format!("{l}1|a|b|c"),
                'G' => "prose line".to_owned(),
                _ => "- a bullet".to_owned(),
            };
            let text =
                format!("## \u{a7}{letter} X\n{body}\n", letter = letter);
            let got = addresses(&text);
            assert_eq!(
                got,
                vec![format!("\u{a7}{letter}.1")],
                "\u{a7}{letter} has no address"
            );
        }
    }

    /// §R is addressable too, once 4.1.0 put it in the section list.
    #[test]
    fn a_research_row_is_addressable() {
        let with_r = SPEC.replace(
            "## \u{a7}V INVARIANTS",
            "## \u{a7}R RESEARCH\nR1|jwt|`jose` wins|url\n\n## \u{a7}V INVARIANTS",
        );
        let got = anchors(&with_r);
        let r = got.iter().find(|a| a.address == "\u{a7}R.1");
        assert_eq!(r.map(|a| a.id.as_str()), Some("R1"), "{got:?}");
    }

    fn addresses(text: &str) -> Vec<String> {
        anchors(text).into_iter().map(|a| a.address).collect()
    }

    /// Every section is addressed, including the ones with no ids at all.
    #[test]
    fn every_section_yields_addresses() {
        let got = addresses(SPEC);
        for want in ["\u{a7}G.1", "\u{a7}C.2", "\u{a7}I.1", "\u{a7}V.2"] {
            assert!(got.contains(&want.to_owned()), "{want} missing: {got:?}");
        }
    }

    /// Prose and bullets have no id, and say so rather than inventing one.
    #[test]
    fn an_item_without_an_id_prints_a_dash() {
        let found = anchors(SPEC);
        let goal = found.iter().find(|a| a.address == "\u{a7}G.1");
        assert_eq!(goal.map(|a| a.id.as_str()), Some("-"));
        let v1 = found.iter().find(|a| a.address == "\u{a7}V.1");
        assert_eq!(v1.map(|a| a.id.as_str()), Some("V1"));
    }

    /// Table furniture is not an item: the header row, the separator, and the
    /// `| M<n> |` milestone row. Counting any of them would push every task's
    /// ordinal out of step with its id, so `§T.4` would stop meaning T4 on a
    /// perfectly well-formed spec -- manufacturing exactly the drift the id
    /// column exists to expose.
    #[test]
    fn table_furniture_and_milestone_rows_are_not_addressable() {
        let found = anchors(SPEC);
        let t1 = found.iter().find(|a| a.id == "T1");
        assert_eq!(t1.map(|a| a.address.as_str()), Some("\u{a7}T.1"));
        assert!(
            !found.iter().any(|a| a.gist.contains("| M1 |")),
            "milestone"
        );
        assert!(
            !found.iter().any(|a| a.gist.starts_with("|---")),
            "separator"
        );
        let b1 = found.iter().find(|a| a.id == "B1");
        assert_eq!(b1.map(|a| a.address.as_str()), Some("\u{a7}B.1"));
    }

    /// The companion: with no gaps, every ordinal equals its id number, so
    /// the report says the addresses are stable.
    #[test]
    fn a_spec_without_gaps_reports_stable() {
        assert!(anchors(SPEC).iter().all(|a| !a.diverges()));
        assert!(report(SPEC).contains("stable:"), "{}", report(SPEC));
    }

    /// PLANTED: retire V1, and §V.1 now resolves to V2. The address did not
    /// change, the rule it names did -- which is why the divergence is worth
    /// a line of its own rather than being left for a reader to notice.
    #[test]
    fn a_gap_shifts_later_ordinals_and_is_reported() {
        let gapped = SPEC.replace("V1: **first.**\n", "");
        let found = anchors(&gapped);
        let first = found.iter().find(|a| a.address == "\u{a7}V.1");
        assert_eq!(first.map(|a| a.id.as_str()), Some("V2"));
        assert!(first.is_some_and(Anchor::diverges));
        assert!(report(&gapped).contains("shifted \u{a7}V.1"));
    }

    /// A suffixed row rides its base and so has no number of its own to
    /// compare -- `T30a` is not "item 30a". It must not be called shifted.
    #[test]
    fn a_suffixed_id_is_not_reported_as_shifted() {
        let a = Anchor {
            address: "\u{a7}T.3".to_owned(),
            id: "T3a".to_owned(),
            gist: String::new(),
            full: String::new(),
        };
        assert!(a.diverges(), "3a differs from 3, and that is real");
        let plain = Anchor {
            address: "\u{a7}T.3".to_owned(),
            id: "T3".to_owned(),
            gist: String::new(),
            full: String::new(),
        };
        assert!(!plain.diverges());
    }

    /// Verbose keeps the whole statement, so a listing can be searched for
    /// the clause that distinguishes two items rather than only their
    /// openings.
    #[test]
    fn verbose_prints_the_whole_line() {
        let long = format!("V1: **a rule.** {}", "distinguishing ".repeat(8));
        let text = format!("## \u{a7}V INVARIANTS\n{long}\n");
        let whole = long.trim_end();
        assert!(!report(&text).contains(whole), "gist must truncate");
        assert!(report_verbose(&text).contains(whole), "verbose must not");
        // Both still end with the same verdict about the addresses.
        assert!(report_verbose(&text).contains("stable:"));
    }

    #[test]
    fn the_gist_is_cut_by_chars_not_bytes() {
        let long = format!("V1: {}", "\u{22a5}".repeat(80));
        let text = format!("## \u{a7}V INVARIANTS\n{long}\n");
        let found = anchors(&text);
        assert_eq!(found.first().map(|a| a.gist.chars().count()), Some(60));
    }
}
