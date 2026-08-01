//! The structural rules (V11-V16), as pure functions over `&str`.
//!
//! Each returns the violations it found, empty meaning clean, so a consumer
//! CALLS the rule instead of re-porting it (V7) and the binary is a thin shell
//! over the same code the tests exercise.
//!
//! These are STRUCTURAL. They cannot prove a rewrite preserved meaning, and
//! they are not meant to: what they catch is the dangerous class a byte count
//! cannot see -- a vanished invariant, a citation pointing at nothing, a
//! section that lost its header. That class is what let 88 tasks belong to no
//! milestone while two gates stayed green.
//!
//! Restated here rather than referenced. The first consumer carries a ported
//! copy of these rules today, and T7 deletes it; a rule whose evidence lived
//! only in the copy would lose that evidence with it (V19).

use crate::id::{at_line_start, Id};

/// FORMAT.md fixes the sections and their order.
///
/// `\u{a7}` is the section sign, written as an escape so this source stays
/// ASCII -- the runtime string is identical either way.
pub const SECTIONS: [&str; 6] = [
    "## \u{a7}G GOAL",
    "## \u{a7}C CONSTRAINTS",
    "## \u{a7}I INTERFACE",
    "## \u{a7}V INVARIANTS",
    "## \u{a7}T TASKS",
    "## \u{a7}B BUGS",
];

/// Every id DECLARED in the text, of one kind, in the order they appear.
#[must_use]
pub fn declared(text: &str, kind: char) -> Vec<Id> {
    text.lines()
        .filter_map(at_line_start)
        .filter(|id| id.kind == kind)
        .collect()
}

/// Every `V<n>` mentioned anywhere -- a citation from any section.
///
/// Splitting on non-alphanumerics rather than whitespace, so `(V21,V22)` and
/// `V13.` are found: a citation is rarely followed by a space.
#[must_use]
pub fn cited(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| is_invariant_ref(t))
        .map(str::to_owned)
        .collect()
}

fn is_invariant_ref(token: &str) -> bool {
    match token.strip_prefix('V') {
        Some(n) => !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

/// V11: sections are PRESENT and ORDERED. A lost header silently unnames
/// every item under it, so the items look like prose and no rule reaches them.
#[must_use]
pub fn sections_ordered(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut last = 0usize;
    for header in SECTIONS {
        match text.find(header) {
            None => out.push(format!("V11: missing section `{header}`")),
            Some(at) if at < last => {
                out.push(format!("V11: `{header}` is out of order"));
            }
            Some(at) => last = at,
        }
    }
    out
}

/// V12: ids are UNIQUE and never reused; a GAP is fine.
///
/// A skipped number costs nothing. A REUSED one silently redirects every
/// citation that pointed at the old meaning, which is why this checks for
/// repeats and says nothing about gaps.
#[must_use]
pub fn ids_unique(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for kind in ['V', 'T', 'B'] {
        let mut seen: Vec<String> = Vec::new();
        for label in declared(text, kind).iter().map(Id::label) {
            if seen.contains(&label) {
                out.push(format!("V12: `{label}` is declared twice"));
            }
            seen.push(label);
        }
    }
    out
}

/// V13: every citation RESOLVES.
///
/// A dangling `V99` is a pointer into nothing and it reads as authoritative --
/// the most expensive kind of wrong, because nobody follows a reference that
/// looks deliberate.
#[must_use]
pub fn citations_resolve(text: &str) -> Vec<String> {
    let declared: Vec<String> =
        declared(text, 'V').iter().map(Id::label).collect();
    let mut out: Vec<String> = cited(text)
        .into_iter()
        .filter(|c| !declared.contains(c))
        .map(|c| format!("V13: `{c}` is cited but never declared"))
        .collect();
    out.dedup();
    out
}

/// V14: rows appear in SORTED id order, and a suffixed id RIDES its base.
///
/// An out-of-order block RENDERS identically to a sorted one, so nothing but a
/// check sees it -- measured in the first consumer, where four rows had sat
/// out of order for weeks because rows get appended wherever is convenient.
#[must_use]
pub fn rows_sorted(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for kind in ['T', 'B'] {
        let keys: Vec<(u32, String)> =
            declared(text, kind).iter().map(Id::sort_key).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        if keys != sorted {
            out.push(format!("V14: a `{kind}` row is out of id order"));
        }
    }
    out
}

/// V15: every task belongs to EXACTLY ONE milestone.
///
/// Named by the original checker as the rule most often broken and invisible
/// without a check; it was right -- 88 tasks belonged to none while two gates
/// stayed green, after the mapping was deleted on the reasoning that no runner
/// read it.
#[must_use]
pub fn tasks_in_one_milestone(text: &str) -> Vec<String> {
    let claimed = claims(text);
    let mut out = duplicate_claims(&claimed);
    for id in declared(text, 'T') {
        if !id.suffix.is_empty() {
            continue;
        }
        if !claimed.contains(&id.num) {
            out.push(format!("V15: {} is in no milestone", id.label()));
        }
    }
    out.extend(claims_without_rows(text, &claimed));
    out
}

fn duplicate_claims(claimed: &[u32]) -> Vec<String> {
    let mut seen: Vec<u32> = Vec::new();
    let mut out = Vec::new();
    for n in claimed {
        if seen.contains(n) {
            out.push(format!("V15: T{n} is claimed by two milestones"));
        }
        seen.push(*n);
    }
    out
}

/// The other direction: a milestone naming a task that has no row. Without it
/// the rule passes by claiming everything, including work that does not exist.
fn claims_without_rows(text: &str, claimed: &[u32]) -> Vec<String> {
    let rows: Vec<u32> = declared(text, 'T').iter().map(|i| i.num).collect();
    claimed
        .iter()
        .filter(|n| !rows.contains(n))
        .map(|n| format!("V15: a milestone claims T{n}, which has no row"))
        .collect()
}

/// Every task number claimed by a `| M<n> |` row's THIRD field -- the same
/// cell the original checker reads, so the two cannot disagree about where to
/// look.
#[must_use]
pub fn claims(text: &str) -> Vec<u32> {
    text.lines()
        .filter(|l| l.starts_with("| M"))
        .flat_map(|l| {
            let fields: Vec<&str> = l.split('|').collect();
            expand_cell(fields.get(3).copied().unwrap_or(""))
        })
        .collect()
}

/// `T1-T4, T12` -> the task numbers it names.
///
/// Ranges are the format's own affordance and the reason the mapping is cheap
/// to maintain: not knowing about them is why the column once got judged a
/// burden and deleted.
#[must_use]
pub fn expand_cell(cell: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for token in cell.split(',') {
        match token.trim().split_once('-') {
            Some((a, b)) => match (task_num(a), task_num(b)) {
                (Some(lo), Some(hi)) => out.extend(lo..=hi),
                _ => continue,
            },
            None => out.extend(task_num(token)),
        }
    }
    out
}

fn task_num(s: &str) -> Option<u32> {
    s.trim().strip_prefix('T')?.parse().ok()
}

/// V16: considered-and-REJECTED records SURVIVE.
///
/// An option recorded with its rejection & the trigger that would reopen it
/// is what makes a decision AUDITABLE rather than merely obeyable, and
/// compaction must never trade one away for bytes.
///
/// `expected` is the CALLER's, and that is the whole design: survival is a
/// claim about edits over time, so no single text can say what used to be in
/// it. The checker owns the rule; the repo being checked owns the list.
///
/// Checked by NAMED records rather than a word count, because an arbitrary
/// threshold on how often a word appears either fires on nothing or fires on
/// prose edits that changed no decision.
#[must_use]
pub fn records_survive(
    text: &str,
    expected: &[(String, String)],
) -> Vec<String> {
    expected
        .iter()
        .filter(|(id, marker)| !body(text, id).contains(marker.as_str()))
        .map(|(id, marker)| format!("V16: {id} lost its `{marker}` record"))
        .collect()
}

/// The body of one declaration, up to the next one.
///
/// Scoped rather than searching the whole file: a marker that moved to some
/// other rule would otherwise still count as present, and "the record is
/// somewhere" is not the claim V16 makes.
#[must_use]
pub fn body(text: &str, id: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in text.lines() {
        match at_line_start(line) {
            Some(found) if found.label() == id => inside = true,
            Some(_) => inside = false,
            None => {}
        }
        if inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Parse a `.spec-records` baseline: `<id>` then whitespace then the marker,
/// which runs to end of line so it may contain spaces. `#` comments and blank
/// lines are skipped.
#[must_use]
pub fn parse_records(text: &str) -> Vec<(String, String)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once(char::is_whitespace))
        .map(|(id, marker)| (id.to_owned(), marker.trim().to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal spec with every section, in order, that all five rules
    /// accept. The companion to every planted violation below: a guard that
    /// rejected everything would pass an all-planted suite (V18).
    const REAL: [&str; 27] = [
        "# spec",
        "",
        "## \u{a7}G GOAL",
        "one line.",
        "",
        "## \u{a7}C CONSTRAINTS",
        "- a bullet",
        "",
        "## \u{a7}I INTERFACE",
        "- `cmd foo` -- does a thing",
        "",
        "## \u{a7}V INVARIANTS",
        "V1: **a rule.** cited by T1.",
        "V3: **a gap above is fine.** V1 is cited here too.",
        "",
        "## \u{a7}T TASKS",
        "| id | scope | tasks | done-when |",
        "|----|-------|-------|-----------|",
        "| M1 | first | T1-T2, T4 | done |",
        "T1|x|a task|V1",
        "T2|.|another|V3",
        "T2a|.|rides its base|V1",
        "T4|.|after a gap|V1",
        "",
        "## \u{a7}B BUGS",
        "id|date|cause|fix",
        "B1|2026-08-01|a cause|a fix",
    ];

    fn real() -> String {
        REAL.join("\n")
    }

    /// Swap two whole lines. A `replace` pair cannot do this: the second call
    /// finds the header the first one just wrote and undoes it -- which is
    /// exactly how the first version of this test passed while planting
    /// nothing at all.
    fn swap_lines(text: &str, a: &str, b: &str) -> String {
        text.lines()
            .map(|l| match l {
                _ if l == a => b,
                _ if l == b => a,
                _ => l,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn all(text: &str) -> Vec<String> {
        let mut out = sections_ordered(text);
        out.extend(ids_unique(text));
        out.extend(citations_resolve(text));
        out.extend(rows_sorted(text));
        out.extend(tasks_in_one_milestone(text));
        out
    }

    /// The companion for all five rules at once: every real shape passes.
    #[test]
    fn a_well_formed_spec_has_no_violations() {
        assert_eq!(all(&real()), Vec::<String>::new());
    }

    fn v11_says(text: &str, want: &str) {
        let got = sections_ordered(text);
        assert!(got.iter().any(|v| v.contains(want)), "want {want}: {got:?}");
    }

    /// A header lost to an edit. Everything under it is still there and still
    /// reads fine, which is the problem: it is no longer in any section.
    #[test]
    fn v11_rejects_a_missing_section() {
        v11_says(&real().replace("## \u{a7}B BUGS", "## BUGS"), "missing");
    }

    /// Order is part of the format, not a convention, because every `§S.n`
    /// address is read against it.
    #[test]
    fn v11_rejects_a_misordered_section() {
        let swapped =
            swap_lines(&real(), "## \u{a7}V INVARIANTS", "## \u{a7}T TASKS");
        v11_says(&swapped, "out of order");
    }

    /// V12, planted: the same id declared twice. And the companion that
    /// matters most here -- a GAP is not a violation, or every spec that ever
    /// retired a number would be red.
    #[test]
    fn v12_rejects_a_repeat_but_allows_a_gap() {
        let dup = real().replace("V3: **a gap", "V1: **a gap");
        assert!(
            ids_unique(&dup).iter().any(|v| v.contains("`V1`")),
            "{:?}",
            ids_unique(&dup)
        );
        // real() declares V1 and V3 -- V2 is a gap, and it is clean.
        assert!(ids_unique(&real()).is_empty());
    }

    /// V13, planted: a citation to an invariant that was never declared.
    #[test]
    fn v13_rejects_a_dangling_citation() {
        let dangling = real().replace("T1|x|a task|V1", "T1|x|a task|V99");
        assert!(
            citations_resolve(&dangling)
                .iter()
                .any(|v| v.contains("V99")),
            "{:?}",
            citations_resolve(&dangling)
        );
    }

    /// V14, planted: two rows swapped. The companion is in `real()`, where
    /// `T2a` follows `T2` and precedes `T4` -- the case lexical ordering gets
    /// wrong, since "T2a" sorts before "T4" only by luck of the alphabet.
    #[test]
    fn v14_rejects_an_out_of_order_row() {
        let swapped = real().replace(
            "T1|x|a task|V1\nT2|.|another|V3",
            "T2|.|another|V3\nT1|x|a task|V1",
        );
        assert!(!rows_sorted(&swapped).is_empty());
        assert!(rows_sorted(&real()).is_empty());
    }

    /// V15 fails in three directions, so it is planted three times below. A
    /// guard that caught only the first would look just as green as one that
    /// caught all three -- and the first is the one that already happened.
    fn v15_says(text: &str, want: &str) {
        let got = tasks_in_one_milestone(text);
        assert!(got.iter().any(|v| v.contains(want)), "want {want}: {got:?}");
    }

    /// The measured failure: 88 tasks belonged to no milestone.
    #[test]
    fn v15_rejects_a_task_in_no_milestone() {
        let orphan = real().replace("| T1-T2, T4 |", "| T1-T2 |");
        v15_says(&orphan, "T4 is in no milestone");
    }

    /// EXACTLY one, so two claims on one task is a violation too -- otherwise
    /// the rule is satisfied by claiming everything everywhere.
    #[test]
    fn v15_rejects_a_task_claimed_twice() {
        let twice = real().replace(
            "| M1 | first | T1-T2, T4 | done |",
            "| M1 | first | T1-T2, T4 | done |\n| M2 | second | T1 | done |",
        );
        v15_says(&twice, "claimed by two");
    }

    /// And the mirror: a milestone naming work that has no row, which is what
    /// a renamed or deleted task leaves behind.
    #[test]
    fn v15_rejects_a_milestone_claiming_a_missing_row() {
        let phantom = real().replace("| T1-T2, T4 |", "| T1-T2, T4, T9 |");
        v15_says(&phantom, "has no row");
    }

    /// Ranges are the affordance that makes the milestone column cheap, so
    /// they get their own test: `T1-T2, T4` is three tasks, not two tokens.
    #[test]
    fn a_range_expands_and_a_suffixed_row_rides_its_base() {
        assert_eq!(expand_cell("T1-T4, T12"), vec![1, 2, 3, 4, 12]);
        assert_eq!(expand_cell(" T7 "), vec![7]);
        assert_eq!(expand_cell(""), Vec::<u32>::new());
        // `T2a` has a row but is never claimed on its own -- it rides `T2`.
        assert!(tasks_in_one_milestone(&real()).is_empty());
    }

    fn records(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(i, m)| ((*i).to_owned(), (*m).to_owned()))
            .collect()
    }

    /// V16, planted: the record is stripped out of the body that carried it.
    #[test]
    fn v16_rejects_a_record_that_was_edited_away() {
        let want = records(&[("V1", "a rule")]);
        assert!(records_survive(&real(), &want).is_empty(), "companion");
        let stripped = real().replace("V1: **a rule.**", "V1: **a thing.**");
        assert!(
            records_survive(&stripped, &want)
                .iter()
                .any(|v| v.contains("V1 lost")),
            "{:?}",
            records_survive(&stripped, &want)
        );
    }

    /// The body is SCOPED to its own declaration. A marker that survived only
    /// by moving to a different rule has not survived: the decision it
    /// documented now hangs off something that never made it.
    #[test]
    fn a_record_that_moved_to_another_rule_does_not_count() {
        let moved = real()
            .replace("V1: **a rule.** cited by T1.", "V1: **moved.**")
            .replace("V3: **a gap", "V3: **a rule.** **a gap");
        let want = records(&[("V1", "a rule")]);
        assert!(!records_survive(&moved, &want).is_empty());
    }

    #[test]
    fn a_records_file_parses_with_comments_and_spaces() {
        let got = parse_records("# a note\n\nT6   DROPPED\nV24  a built pkg\n");
        assert_eq!(got, records(&[("T6", "DROPPED"), ("V24", "a built pkg")]));
    }

    #[test]
    fn a_citation_is_found_next_to_punctuation() {
        let found = cited("cites (V21,V22) and V13. not Vx or V");
        assert_eq!(found, vec!["V21", "V22", "V13"]);
    }
}
