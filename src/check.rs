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
use crate::violation::{Fix, Violation};

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
    declared_at(text, kind)
        .into_iter()
        .map(|(_, id)| id)
        .collect()
}

/// The same, each with its 1-based line, so a violation can say WHERE.
#[must_use]
pub fn declared_at(text: &str, kind: char) -> Vec<(usize, Id)> {
    text.lines()
        .enumerate()
        .filter_map(|(i, l)| Some((i.saturating_add(1), at_line_start(l)?)))
        .filter(|(_, id)| id.kind == kind)
        .collect()
}

/// Every `V<n>` CITED -- that is, mentioned OUTSIDE backticks (V13).
///
/// Inside backticks it is a literal: `grep V47` is an example command, and
/// `itok's V82` is another repo's namespace, which V19 requires be written
/// exactly that way. Both were reported as dangling citations on this
/// checker's first real run (B3), and the rule's own illustration of a
/// dangling reference was among them.
///
/// The boundary is READ OFF the format rather than invented here: FORMAT.md
/// already reserves backticks for verbatim text. A boundary invented in the
/// checker is one the next consumer re-derives differently.
///
/// Splitting on non-alphanumerics rather than whitespace, so `(V21,V22)` and
/// `V13.` are found: a citation is rarely followed by a space.
#[must_use]
pub fn cited(text: &str) -> Vec<String> {
    outside_backticks(text)
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| is_invariant_ref(t))
        .map(str::to_owned)
        .collect()
}

/// Every citation with the 1-based line it appears on.
///
/// Per-line rather than whole-file, so a dangling reference can be pointed
/// at. The backtick boundary (V13) is applied line by line, which is a
/// deliberate narrowing: a code span never legitimately spans a line here,
/// and scoping it per line stops one stray backtick suppressing citations
/// for the rest of the file.
#[must_use]
pub fn cited_at(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .flat_map(|(i, line)| {
            cited(line)
                .into_iter()
                .map(move |c| (i.saturating_add(1), c))
        })
        .collect()
}

/// The text with every backticked span removed.
///
/// An unclosed backtick swallows the rest of the text, which is the safe
/// direction: it can only SUPPRESS citations, never invent one, so the
/// failure is a check that misses rather than a check that lies.
fn outside_backticks(text: &str) -> String {
    text.split('`').step_by(2).collect::<Vec<_>>().join(" ")
}

fn is_invariant_ref(token: &str) -> bool {
    match token.strip_prefix('V') {
        Some(n) => !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

/// V11: sections are PRESENT and ORDERED.
///
/// Document-scoped: a missing header has no line to point at, and pointing
/// at where it OUGHT to be would be a guess dressed as a fact.
#[must_use]
pub fn sections_ordered(text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut last = 0usize;
    for header in SECTIONS {
        match text.find(header) {
            None => out.push(missing_section(header)),
            Some(at) if at < last => out.push(misordered_section(header)),
            Some(at) => last = at,
        }
    }
    out
}

fn missing_section(header: &str) -> Violation {
    Violation::new("V11", format!("missing section `{header}`"))
        .why("a lost header silently unnames every item under it")
        .try_(
            Fix::Mechanical,
            format!("add `{header}` in FORMAT.md order"),
        )
}

fn misordered_section(header: &str) -> Violation {
    Violation::new("V11", format!("`{header}` is out of order"))
        .why("every §S.n address is read against the section order")
        .try_(Fix::Mechanical, "move the section into FORMAT.md order")
}

/// V12: ids are UNIQUE and never reused; a GAP is fine.
#[must_use]
pub fn ids_unique(text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    for kind in ['V', 'T', 'B'] {
        let mut seen: Vec<String> = Vec::new();
        for (line, id) in declared_at(text, kind) {
            let label = id.label();
            if seen.contains(&label) {
                out.push(duplicate_id(&label).at(line));
            }
            seen.push(label);
        }
    }
    out
}

fn duplicate_id(label: &str) -> Violation {
    Violation::new("V12", format!("`{label}` is declared twice"))
        .why("a reused id silently redirects every citation to the old meaning")
        .try_(
            Fix::Mechanical,
            "give this one the next FREE id; a gap costs nothing",
        )
}

/// V13: every citation RESOLVES; a citation is a `V<n>` outside backticks.
#[must_use]
pub fn citations_resolve(text: &str) -> Vec<Violation> {
    let known: Vec<String> =
        declared(text, 'V').iter().map(Id::label).collect();
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for (line, cite) in cited_at(text) {
        if !known.contains(&cite) && !seen.contains(&cite) {
            seen.push(cite.clone());
            out.push(dangling(&cite).at(line));
        }
    }
    out
}

fn dangling(cite: &str) -> Violation {
    Violation::new("V13", format!("`{cite}` is cited but never declared"))
        .why(
            "a dangling reference reads as authoritative, so nobody follows it",
        )
        .try_(Fix::Mechanical, "point it at the rule that was meant")
        .try_(
            Fix::Judgment,
            format!("declare {cite}, if the rule is real but missing"),
        )
}

/// V14: rows appear in SORTED id order, and a suffixed id RIDES its base.
#[must_use]
pub fn rows_sorted(text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    for kind in ['T', 'B'] {
        let rows = declared_at(text, kind);
        for pair in rows.windows(2) {
            match (pair.first(), pair.get(1)) {
                (Some(a), Some(b)) if b.1.sort_key() < a.1.sort_key() => {
                    out.push(unsorted(&b.1.label(), &a.1.label()).at(b.0));
                }
                _ => continue,
            }
        }
    }
    out
}

fn unsorted(row: &str, after: &str) -> Violation {
    Violation::new("V14", format!("`{row}` sorts before `{after}` above it"))
        .why("an out-of-order block renders identically to a sorted one")
        .try_(
            Fix::Mechanical,
            "move the row into id order; a suffixed id rides its base",
        )
}

/// V15: every task belongs to EXACTLY ONE milestone.
#[must_use]
pub fn tasks_in_one_milestone(text: &str) -> Vec<Violation> {
    let claimed = claims(text);
    let mut out = duplicate_claims(&claimed);
    for (line, id) in declared_at(text, 'T') {
        if id.suffix.is_empty() && !claimed.contains(&id.num) {
            out.push(unclaimed(&id.label()).at(line));
        }
    }
    out.extend(claims_without_rows(text, &claimed));
    out
}

fn unclaimed(label: &str) -> Violation {
    Violation::new("V15", format!("{label} is in no milestone"))
        .why("88 tasks once belonged to none while two gates stayed green")
        .try_(
            Fix::Mechanical,
            "add it to a milestone's tasks cell; ranges expand",
        )
        .try_(Fix::Judgment, "delete the row, if the work is not real")
}

fn duplicate_claims(claimed: &[u32]) -> Vec<Violation> {
    let mut seen: Vec<u32> = Vec::new();
    let mut out = Vec::new();
    for n in claimed {
        if seen.contains(n) {
            out.push(claimed_twice(*n));
        }
        seen.push(*n);
    }
    out
}

fn claimed_twice(n: u32) -> Violation {
    Violation::new("V15", format!("T{n} is claimed by two milestones"))
        .why("EXACTLY one, or the rule is satisfied by claiming everything")
        .try_(
            Fix::Mechanical,
            "remove the claim from all but one milestone",
        )
}

/// The other direction: a milestone naming a task that has no row. Without it
/// the rule passes by claiming everything, including work that does not exist.
fn claims_without_rows(text: &str, claimed: &[u32]) -> Vec<Violation> {
    let rows: Vec<u32> = declared(text, 'T').iter().map(|i| i.num).collect();
    claimed
        .iter()
        .filter(|n| !rows.contains(n))
        .map(|n| phantom_claim(*n))
        .collect()
}

fn phantom_claim(n: u32) -> Violation {
    Violation::new("V15", format!("a milestone claims T{n}, which has no row"))
        .why("a renamed or deleted task leaves the claim behind")
        .try_(Fix::Mechanical, "drop the claim")
        .try_(
            Fix::Judgment,
            format!("add the T{n} row, if the work is real"),
        )
}

/// The three statuses FORMAT.md allows, and nothing else.
pub const STATUSES: [&str; 3] = [".", "~", "x"];

/// V25: a task STATUS is `.` todo, `~` wip or `x` done.
///
/// Read from the SECOND pipe field of a `T<n>|` row. A status outside the
/// set renders as ordinary text, so a table full of them looks fine while
/// every runner that reads the column disagrees about the state of the work.
#[must_use]
pub fn statuses_valid(text: &str) -> Vec<Violation> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let id = at_line_start(line).filter(|id| id.kind == 'T')?;
            let status = line.split('|').nth(1)?;
            (!STATUSES.contains(&status)).then(|| {
                bad_status(&id.label(), status).at(i.saturating_add(1))
            })
        })
        .collect()
}

fn bad_status(label: &str, status: &str) -> Violation {
    Violation::new("V25", format!("{label} has status `{status}`"))
        .why("a status outside . ~ x renders fine and every runner reads it differently")
        .try_(Fix::Mechanical, "set it to `.` todo, `~` wip or `x` done")
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

/// One named record: the id that owns it, and a marker substring of it.
pub type Record = (String, String);

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
pub fn records_survive(text: &str, expected: &[Record]) -> Vec<Violation> {
    expected
        .iter()
        .filter(|(id, marker)| !body(text, id).contains(marker.as_str()))
        .map(|(id, marker)| lost_record(id, marker, line_of(text, id)))
        .collect()
}

/// V16 offers NO mechanical direction, and that is the point rather than an
/// omission. Restoring a deleted record needs the record; deciding the option
/// no longer needs one changes intent. Both are judgement, so an agent that
/// hits this must stop, which is exactly what compaction trading a record
/// away for bytes should feel like.
fn lost_record(id: &str, marker: &str, line: usize) -> Violation {
    Violation::new("V16", format!("{id} lost its `{marker}` record"))
        .why("a rejected option without its record is obeyable but not auditable")
        .try_(Fix::Judgment, "restore the record, or record why the option no longer needs one")
        .at(line)
}

/// The 1-based line a declaration sits on, or 0 if it is gone entirely.
fn line_of(text: &str, id: &str) -> usize {
    text.lines()
        .enumerate()
        .find(|(_, l)| at_line_start(l).is_some_and(|i| i.label() == id))
        .map_or(0, |(i, _)| i.saturating_add(1))
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
pub fn parse_records(text: &str) -> Vec<Record> {
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

    fn all(text: &str) -> Vec<Violation> {
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
        assert_eq!(all(&real()), Vec::<Violation>::new());
    }

    fn v11_says(text: &str, want: &str) {
        let got = sections_ordered(text);
        assert!(
            got.iter().any(|v| v.msg.contains(want)),
            "want {want}: {got:?}"
        );
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
            ids_unique(&dup).iter().any(|v| v.msg.contains("`V1`")),
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
                .any(|v| v.msg.contains("V99")),
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
        assert!(
            got.iter().any(|v| v.msg.contains(want)),
            "want {want}: {got:?}"
        );
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

    fn records(pairs: &[(&str, &str)]) -> Vec<Record> {
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
                .any(|v| v.msg.contains("V1 lost")),
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

    /// V25, planted: a status outside the set. This is the rule the host
    /// checker had and nanokit did not (B4), so it gets the same treatment
    /// as the rest -- a plant and a companion.
    #[test]
    fn v25_rejects_a_status_outside_the_set() {
        let bad = real().replace("T1|x|a task|V1", "T1|q|a task|V1");
        let got = statuses_valid(&bad);
        assert!(
            got.iter().any(|v| v.msg.contains("T1 has status `q`")),
            "{got:?}"
        );
        assert!(got.iter().any(|v| v.line > 0), "names the line: {got:?}");
    }

    /// The companion: all three real statuses are accepted, so the guard
    /// cannot pass by rejecting every row.
    #[test]
    fn v25_accepts_every_status_the_format_allows() {
        assert!(statuses_valid(&real()).is_empty(), "the fixture is clean");
        for s in STATUSES {
            let text = format!("## \u{a7}T TASKS\nT1|{s}|a task|V1\n");
            assert!(statuses_valid(&text).is_empty(), "rejected `{s}`");
        }
    }

    /// A milestone row has no status field and must not be read as though it
    /// did -- it opens with `|`, so it is not a task row at all.
    #[test]
    fn a_milestone_row_is_not_checked_for_a_status() {
        let text = "## \u{a7}T TASKS\n| M1 | scope | T1 | done |\n";
        assert!(statuses_valid(text).is_empty());
    }

    #[test]
    fn a_citation_is_found_next_to_punctuation() {
        let found = cited("cites (V21,V22) and V13. not Vx or V");
        assert_eq!(found, vec!["V21", "V22", "V13"]);
    }

    /// V13's boundary, planted from the three shapes that actually fired on
    /// this checker's first real run (B3): an example command, the rule's own
    /// illustration, and a qualified cross-project reference.
    #[test]
    fn a_backticked_mention_is_a_literal_not_a_citation() {
        assert!(cited("`grep V47` returned a fragment").is_empty());
        assert!(cited("a dangling `V99` points at nothing").is_empty());
        assert!(cited("name the repo (`itok's V82`)").is_empty());
    }

    /// ...and the boundary must not swallow real citations sharing a line
    /// with backticked text, which is the common case in this very spec.
    #[test]
    fn a_citation_beside_backticks_still_counts() {
        let line = "V5: the cap in `format.rs` is measured (V21), not taste";
        assert_eq!(cited(line), vec!["V5", "V21"]);
    }
}
