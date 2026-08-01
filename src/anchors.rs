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
//! `§G`, `§C` and `§I` are prose and bullets with no ids at all, so they print
//! `-`: for those sections the ordinal is the only address there is.

use crate::check::SECTIONS;
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
fn is_item(line: &str, letter: char) -> bool {
    if at_line_start(line).is_some() {
        return true;
    }
    match letter {
        'G' => !line.trim().is_empty() && !line.starts_with("##"),
        'C' | 'I' => line.starts_with("- "),
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
/// `None` inside it for a header that is not one of the six (V11's list), so
/// stray `##` prose cannot be addressed as if it were spec content.
fn opens_section(line: &str) -> Option<Option<char>> {
    let letter = section_letter(line)?;
    Some(
        SECTIONS
            .iter()
            .any(|s| line.starts_with(*s))
            .then_some(letter),
    )
}

fn anchor(line: &str, letter: char, n: usize) -> Anchor {
    Anchor {
        address: format!("\u{a7}{letter}.{n}"),
        id: at_line_start(line).map_or_else(|| "-".to_owned(), |i| i.label()),
        gist: gist(line),
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
    let found = anchors(text);
    let mut out: String = found
        .iter()
        .map(|a| format!("{}\t{}\t{}\n", a.address, a.id, a.gist))
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

## \u{a7}I INTERFACE
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
        };
        assert!(a.diverges(), "3a differs from 3, and that is real");
        let plain = Anchor {
            address: "\u{a7}T.3".to_owned(),
            id: "T3".to_owned(),
            gist: String::new(),
        };
        assert!(!plain.diverges());
    }

    #[test]
    fn the_gist_is_cut_by_chars_not_bytes() {
        let long = format!("V1: {}", "\u{22a5}".repeat(80));
        let text = format!("## \u{a7}V INVARIANTS\n{long}\n");
        let found = anchors(&text);
        assert_eq!(found.first().map(|a| a.gist.chars().count()), Some(60));
    }
}
