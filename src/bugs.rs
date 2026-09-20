//! §B ENUMERATED for a machine: which bug records exist, and what each one
//! carries.
//!
//! WHY IT EXISTS AT ALL, and it is not a new parse: `check` already
//! validates these ids and their citations, and `derive` already counts the
//! rows and resolves what they cite. The reading was there, exercised and
//! trusted -- with no way for a caller to ASK for it. So a consumer acting
//! on bug records had to keep its own parser of a format this crate exists
//! to own, and the two would disagree eventually, silently, on a spelling
//! neither author thought about. That is the defect `tasks` removed for §T
//! (V7), left standing one section further down.
//!
//! WHY A VERB rather than `tasks --section B`: `0.7.2` is published and
//! crates.io is immutable (V30), and every verb here IGNORES a flag it does
//! not know. So `mth tasks --all-sections` or `--section=B` on an older
//! build answers with §T rows, exit 0 -- bug records requested, task rows
//! delivered, and nothing in the exchange says so. An unknown VERB exits 2,
//! so a caller's fallback fires on ABSENCE. T29 made this argument against
//! `derive --format json`; a flag on an ENUMERATING verb is worse, because
//! there the wrong answer is shaped exactly like the right one.
//!
//! Measured against ourselves before it was believed (§G): the SPACED form
//! `--section B <path>` is refused today, and by accident -- `B` is not a
//! known flag's value, so V46's arity guard reads it as a second PATH and
//! answers "one path per run". Two of the three spellings are silent, and
//! the third is right for the wrong reason.
//!
//! §B IS NOT §T WITH DIFFERENT WORDS, which is the other reason the shape is
//! its own. FORMAT.md gives §B `id|date|cause|fix` -- no status column at
//! all -- so a caller reaching for `.` `~` `x` here finds a DATE, and one
//! reaching for a cites cell finds prose that may or may not name a rule.
//! The row grammar is shared (`rows`); the field names are not.

use crate::rows::Row;

/// One `§B` row, as the format defines it: `B<n>|date|cause|fix`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Bug {
    /// The row's id, e.g. `B7`.
    pub id: String,
    /// The date cell, verbatim. NOT parsed into a date: FORMAT.md gives no
    /// format for it, and a reader that rejected what it could not parse
    /// would drop a record -- which is the one thing V16 forbids trading
    /// away.
    pub date: String,
    /// What went wrong, with `\|` read back as `|`.
    pub cause: String,
    /// What now catches it. Often an invariant id, per FORMAT.md's own
    /// example, but prose is what the corpus actually holds -- so it is
    /// carried as written rather than split into citations it may not be.
    pub fix: String,
}

/// Every `§B` row, in V14 order.
pub(crate) fn bugs(text: &str) -> Vec<Bug> {
    crate::rows::of(text, 'B').into_iter().map(one).collect()
}

/// One row, under §B's names for its three cells.
fn one(row: Row) -> Bug {
    let [date, cause, fix] = row.cells;
    Bug {
        id: row.id,
        date,
        cause,
        fix,
    }
}

/// The human rendering: a count line, then one line per row.
///
/// A spec with NO `§B` says so in words rather than printing nothing --
/// silence is a GATE's success shape (V10), and here it would be
/// indistinguishable from a run that failed to reach the file.
pub(crate) fn report(text: &str, full: bool) -> String {
    let rows = bugs(text);
    let mut out = head(&rows, unread(text));
    for b in &rows {
        let shown = crate::rows::shown(&b.cause, full);
        out.push_str(&format!("bug {}: {} -- {shown}\n", b.id, b.date));
    }
    out
}

/// How many §B rows this build could not read (V49).
fn unread(text: &str) -> usize {
    crate::rows::unread(text, 'B')
}

/// The count line: how many records, and how many rows were THERE but
/// unreadable.
///
/// NO tally beside the count, deliberately. §T has one because a status is a
/// closed set the format names; §B's second cell is a DATE, and bucketing
/// dates would be this verb deciding what "recent" means -- judgement, and
/// the caller's (V6).
fn head(rows: &[Bug], unread: usize) -> String {
    if rows.is_empty() {
        return crate::rows::none_read("bugs", 'B', unread);
    }
    format!("bugs: {} rows{}\n", rows.len(), crate::rows::also(unread))
}

/// The machine rendering: one object, `bugs` in V14 order.
///
/// ALWAYS emitted, including `"bugs":[]` for a spec with no `§B`. An empty
/// §B asserts nothing -- a project that has recorded no bug and one whose
/// records this build cannot read are different facts, and `unread` is what
/// tells them apart.
pub(crate) fn json(file: &str, text: &str) -> String {
    let items: Vec<String> = bugs(text).iter().map(one_json).collect();
    format!(
        "{{\"file\":{},\"bugs\":[{}],\"unread\":{}}}\n",
        crate::render::quote(file),
        items.join(","),
        unread(text)
    )
}

fn one_json(b: &Bug) -> String {
    format!(
        "{{\"id\":{},\"date\":{},\"cause\":{},\"fix\":{}}}",
        crate::render::quote(&b.id),
        crate::render::quote(&b.date),
        crate::render::quote(&b.cause),
        crate::render::quote(&b.fix)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §B as FORMAT.md writes it, plus the shapes it permits inside: a
    /// header row, an escaped pipe, a suffixed id, and a fix cell that is
    /// prose rather than a citation.
    const SPEC: &str = "\
## \u{a7}V INVARIANTS
V1: **a rule.** cited below.

## \u{a7}B BUGS
id|date|cause|fix
B1|2026-08-01|a first cause|V1
B2|2026-08-02|a cause naming `a\\|b`|the rule that now catches it
B2a|2026-08-03|a cause riding B2|-
";

    fn ids(text: &str) -> Vec<String> {
        bugs(text).into_iter().map(|b| b.id).collect()
    }

    /// The acceptance case: a record is reported with its id and its date.
    #[test]
    fn a_bug_row_is_reported_with_its_id_and_date() {
        let b1 = bugs(SPEC).into_iter().find(|b| b.id == "B1");
        assert_eq!(b1.as_ref().map(|b| b.date.as_str()), Some("2026-08-01"));
        assert_eq!(b1.map(|b| b.cause), Some("a first cause".to_owned()));
    }

    /// The companion (V18): it cannot pass by reporting every line. The
    /// header row is furniture, the §V statement above is not a §B row, and
    /// only the three records come back.
    #[test]
    fn only_the_rows_come_back_not_the_header_or_another_section() {
        assert_eq!(ids(SPEC), vec!["B1", "B2", "B2a"]);
    }

    /// V14: `B2a` RIDES `B2`, asserted against a spec written OUT of order.
    #[test]
    fn a_suffixed_id_sorts_with_its_base_not_lexically() {
        let out_of_order = SPEC.replace(
            "B1|2026-08-01|a first cause|V1\n",
            "B10|2026-08-10|a tenth cause|-\nB1|2026-08-01|a first cause|V1\n",
        );
        assert_eq!(ids(&out_of_order), vec!["B1", "B2", "B2a", "B10"]);
    }

    /// FORMAT.md's escape, and the reason the cells are split rather than
    /// cut on every pipe: a naive split reads `` `b` `` as the fix cell.
    #[test]
    fn an_escaped_pipe_stays_inside_the_cause() {
        let b2 = bugs(SPEC).into_iter().find(|b| b.id == "B2");
        assert_eq!(
            b2.as_ref().map(|b| b.cause.as_str()),
            Some("a cause naming `a|b`")
        );
        assert_eq!(
            b2.map(|b| b.fix),
            Some("the rule that now catches it".to_owned())
        );
    }

    /// §B IS NOT §T. The second cell is a DATE, and the fourth is prose that
    /// may name a rule or may not -- so neither is reported as a status or
    /// as a citation list, which is what a caller reusing `tasks`'s shape
    /// would have assumed.
    #[test]
    fn the_second_cell_is_a_date_and_the_fourth_is_carried_as_written() {
        let out = json("f", SPEC);
        assert!(!out.contains("\"status\""), "{out}");
        assert!(!out.contains("\"cites\""), "{out}");
        assert!(out.contains("\"date\":\"2026-08-03\""), "{out}");
        assert!(out.contains("\"fix\":\"-\""), "a `-` fix is carried: {out}");
    }

    /// THE distinguishing case: no `§B` at all reports differently from a
    /// `§B` with records in it. Both exit 0, so the payload is what a
    /// consumer reads -- and an empty array is an answer where silence is
    /// indistinguishable from a crash.
    #[test]
    fn no_bugs_reads_differently_from_some() {
        let none = "## \u{a7}V INVARIANTS\nV1: **a rule.** alone here.\n";
        assert!(report(none, false).starts_with("bugs: none"), "empty spec");
        assert_eq!(
            json("f", none),
            "{\"file\":\"f\",\"bugs\":[],\"unread\":0}\n"
        );
        assert!(report(SPEC, false).starts_with("bugs: 3 rows\n"));
    }

    /// THE issue, in one test: three states a caller must tell apart -- an
    /// empty §B, a readable one, and one whose rows are THERE in a dialect.
    /// Without the third, "no pending bugs" and "bug rows I could not read"
    /// are spelled the same way, which is the state this verb was asked for.
    #[test]
    fn an_unread_section_is_not_spelled_like_an_empty_one() {
        let dialect = "## \u{a7}B BUGS\n\n\
            | B1 | 2026-08-01 | a cause | a fix |\n\
            | B2 | 2026-08-02 | another | another |\n";
        let empty = "## \u{a7}B BUGS\n\nnothing recorded yet.\n";
        assert!(json("f", dialect).contains("\"unread\":2"), "counted");
        assert!(json("f", empty).contains("\"unread\":0"), "really empty");
        assert!(json("f", dialect).contains("\"bugs\":[]"), "enumerates");
        assert!(report(dialect, false).starts_with("bugs: none READ"));
        assert!(report(empty, false).starts_with("bugs: none --"));
    }

    /// A section that is PARTLY readable says so beside the rows it read,
    /// rather than reporting a count the caller would believe is the whole.
    #[test]
    fn a_partly_read_section_says_how_many_it_missed() {
        let mixed = "## \u{a7}B BUGS\n\nB1|2026-08-01|read fine|-\n\
            | B2 | 2026-08-02 | not | - |\n";
        let said = report(mixed, false);
        assert!(said.contains("1 more unread"), "{said}");
        assert!(json("f", mixed).contains("\"unread\":1"), "{mixed}");
    }

    /// The JSON is the contract: every field the consumer indexes on, in one
    /// object on one line, with the escape a hand-written encoder must own.
    #[test]
    fn the_json_carries_id_date_cause_and_fix_on_one_line() {
        let out = json("SPEC.md", SPEC);
        assert!(out.starts_with("{\"file\":\"SPEC.md\",\"bugs\":["), "{out}");
        assert!(
            out.contains(
                "{\"id\":\"B1\",\"date\":\"2026-08-01\",\
                 \"cause\":\"a first cause\",\"fix\":\"V1\"}"
            ),
            "{out}"
        );
        assert_eq!(out.lines().count(), 1, "one line per run: {out}");
    }

    /// A quote in a cause would otherwise emit JSON no parser accepts. The
    /// encoder is shared with `render` (V7), so this asserts the reuse
    /// rather than a second escaping.
    #[test]
    fn a_quote_in_a_cause_is_escaped() {
        let text = "## \u{a7}B BUGS\nB1|2026-08-01|a \"quoted\" cause|-\n";
        assert!(json("f", text).contains(r#"a \"quoted\" cause"#));
    }

    /// The human rendering leads with the count, and `--verbose` stops
    /// truncating -- the same deepen-not-repeat rule every verb here keeps.
    #[test]
    fn verbose_prints_the_whole_cause() {
        let long = "z".repeat(80);
        let text = format!("## \u{a7}B BUGS\nB1|2026-08-01|{long}|-\n");
        assert!(report(&text, false).contains("..."), "truncates");
        assert!(!report(&text, true).contains("..."), "verbose does not");
    }

    /// It names the SET and its order, and nothing about which record
    /// matters -- the same line `tasks` holds (V6). A verb that ranked bug
    /// records by anything would be inference wearing a report's clothes.
    #[test]
    fn the_report_recommends_no_next_bug() {
        let out = report(SPEC, false).to_lowercase();
        for word in ["next", "should", "recommend", "start with"] {
            assert!(!out.contains(word), "{word} leaked in: {out}");
        }
    }
}
