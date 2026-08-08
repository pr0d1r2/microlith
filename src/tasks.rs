//! §T ENUMERATED for a machine: which rows exist, and what status each one
//! carries.
//!
//! ENUMERATE, never SELECT. Which rows are `.`, `~` or `x` is mechanical --
//! a field of a row, read by the grammar `check` already owns. WHICH pending
//! task should be worked next is judgement, and it stays with the caller
//! (V6): a fleet driver has its own ordering rules, and a tool that answered
//! for it would be inference wearing a report's clothes. So this names the
//! SET and its ORDER, and says nothing about which member matters.
//!
//! WHY IT IS A VERB and not a section of `derive`: `0.5.0` is published and
//! crates.io is immutable (V30), so an older `mth` is a real thing a consumer
//! runs. `derive --format json` there is an unrecognised flag -- prose on
//! stdout, exit 0 -- which the caller parses as JSON and fails far from the
//! cause. An unknown VERB exits 2, so the fallback fires on ABSENCE rather
//! than on a parse difference, which is the distinction the consumer asked
//! for.
//!
//! Order is V14's: a suffixed id RIDES its base, so `T7a` follows `T7` and
//! never sorts lexically between `T1` and `T2`. Sorted HERE rather than
//! reported in file order, because a spec whose rows are out of order is a
//! `check` finding and not a reason to hand a consumer a different sequence
//! than the format promises.

use crate::id::{Id, at_line_start, cells, unescape};

/// One `§T` row, as the format defines it: `T<n>|status|task|cites`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Task {
    /// The row's id, e.g. `T7a`.
    pub id: String,
    /// The SECOND field, verbatim but trimmed. Whether it is one of `.` `~`
    /// `x` is V25's question, not this one's: a report that silently dropped
    /// a row with a status it disliked would hide exactly what V25 exists to
    /// surface.
    pub status: String,
    /// The task text, with `\|` read back as `|`.
    pub text: String,
    /// The cites cell, split on commas. `-` means none, per FORMAT.md.
    pub cites: Vec<String>,
}

/// Every `§T` row, in V14 order.
pub(crate) fn tasks(text: &str) -> Vec<Task> {
    let mut rows: Vec<(Id, Task)> = text
        .lines()
        .filter_map(|line| {
            let id = at_line_start(line).filter(|i| i.kind == 'T')?;
            Some((id, one(line)?))
        })
        .collect();
    rows.sort_by_key(|(id, _)| id.sort_key());
    rows.into_iter().map(|(_, t)| t).collect()
}

/// One row's cells. A row with no `|` at all -- V26's bulleted `- T1 text`
/// dialect, which has no fields -- carries no status to report, so it is not
/// a task row this verb can speak about.
fn one(line: &str) -> Option<Task> {
    let cells = cells(line);
    let cell = |n: usize| unescape(cells.get(n).copied().unwrap_or("").trim());
    Some(Task {
        id: at_line_start(line)?.label(),
        status: cells.get(1).map(|c| c.trim().to_owned())?,
        text: cell(2),
        cites: cites(&cell(3)),
    })
}

/// `V1,V2` -> the two. `-` is FORMAT.md's empty cell, not a citation.
fn cites(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty() && *c != "-")
        .map(str::to_owned)
        .collect()
}

/// How many rows carry each status, in FORMAT.md's own order.
pub(crate) fn tally(rows: &[Task]) -> Vec<(&'static str, usize)> {
    crate::check::STATUSES
        .iter()
        .map(|s| (*s, rows.iter().filter(|t| t.status == *s).count()))
        .collect()
}

/// The human rendering: a count line, then one line per row.
///
/// A spec with NO `§T` says so in words rather than printing nothing --
/// silence is a GATE's success shape (V10), and here it would be
/// indistinguishable from a run that failed to reach the file.
pub(crate) fn report(text: &str, full: bool) -> String {
    let rows = tasks(text);
    let mut out = head(&rows);
    for t in &rows {
        let shown = if full { t.text.clone() } else { gist(&t.text) };
        out.push_str(&format!("task {}: {} -- {shown}\n", t.id, t.status));
    }
    out
}

/// The count line: how many rows, and how many of each status.
fn head(rows: &[Task]) -> String {
    if rows.is_empty() {
        return "tasks: none -- no \u{a7}T rows here\n".to_owned();
    }
    let counts: Vec<String> = tally(rows)
        .iter()
        .map(|(s, n)| format!("{n} {s}"))
        .collect();
    format!("tasks: {} rows -- {}\n", rows.len(), counts.join(", "))
}

/// Enough of the task to recognise it; `--verbose` prints all of it. The same
/// width `anchors` uses, so the two reports read alike.
fn gist(text: &str) -> String {
    let short: String = text.chars().take(60).collect();
    if short.chars().count() < text.chars().count() {
        return format!("{short}...");
    }
    short
}

/// The machine rendering: one object, `tasks` in V14 order.
///
/// ALWAYS emitted, including `"tasks":[]` for a spec with no `§T`. That is
/// the one place this departs from `render::json`, and deliberately: silence
/// is success for a GATE, where an empty stream means nothing fired. Here an
/// empty stream is indistinguishable from a crash, and the consumer's whole
/// question is whether the backlog is empty or merely unreadable.
pub(crate) fn json(file: &str, text: &str) -> String {
    let items: Vec<String> = tasks(text).iter().map(one_json).collect();
    format!(
        "{{\"file\":{},\"tasks\":[{}]}}\n",
        crate::render::quote(file),
        items.join(",")
    )
}

fn one_json(t: &Task) -> String {
    let cites: Vec<String> =
        t.cites.iter().map(|c| crate::render::quote(c)).collect();
    format!(
        "{{\"id\":{},\"status\":{},\"text\":{},\"cites\":[{}]}}",
        crate::render::quote(&t.id),
        crate::render::quote(&t.status),
        crate::render::quote(&t.text),
        cites.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of every status, a suffixed id, an escaped pipe and an empty
    /// cites cell -- the shapes FORMAT.md permits in one fixture.
    const SPEC: &str = "\
## \u{a7}V INVARIANTS
V1: **a rule.** cited below.

## \u{a7}T TASKS
| id | scope | tasks | done-when |
|----|-------|-------|-----------|
| M1 | core | T1-T2 | all done |
T1|x|a done task|V1
T2|~|a wip task, `Mechanical`\\|`Judgment`|V1
T2a|.|a todo task riding T2|-
";

    fn ids(text: &str) -> Vec<String> {
        tasks(text).into_iter().map(|t| t.id).collect()
    }

    /// The acceptance case: a `.` row is reported with its id and status.
    #[test]
    fn a_todo_row_is_reported_with_its_id_and_status() {
        let rows = tasks(SPEC);
        let todo: Vec<&Task> =
            rows.iter().filter(|t| t.status == ".").collect();
        assert_eq!(todo.len(), 1, "{rows:?}");
        assert_eq!(todo.first().map(|t| t.id.as_str()), Some("T2a"));
    }

    /// The companion (V18): the enumeration cannot pass by calling
    /// everything todo. All three statuses come back as they were written.
    #[test]
    fn every_status_the_format_allows_round_trips() {
        assert_eq!(
            tally(&tasks(SPEC)),
            vec![(".", 1), ("~", 1), ("x", 1)],
            "one of each, from the fixture"
        );
    }

    /// V14: `T2a` rides `T2`. Lexically `"T2a" < "T3"` is right by accident
    /// and `"T10" < "T2"` is wrong, which is the trap the id grammar exists
    /// to avoid -- so the order is asserted against a spec written OUT of it.
    #[test]
    fn a_suffixed_id_sorts_with_its_base_not_lexically() {
        let out_of_order = SPEC.replace(
            "T1|x|a done task|V1\n",
            "T10|x|a tenth task|V1\nT1|x|a done task|V1\n",
        );
        assert_eq!(ids(&out_of_order), vec!["T1", "T2", "T2a", "T10"]);
    }

    /// V26's other dialect: a declaration behind a markdown bullet is the
    /// SAME row and must enumerate identically. 1,097 of 1,750 fleet
    /// declarations are written this way (B8).
    #[test]
    fn a_bulleted_row_enumerates_like_a_bare_one() {
        let bulleted = SPEC.replace("\nT1|", "\n- T1|");
        assert_eq!(ids(&bulleted), ids(SPEC));
        assert_eq!(tally(&tasks(&bulleted)), tally(&tasks(SPEC)));
    }

    /// FORMAT.md's escape, and the reason `cells` exists: a naive split cuts
    /// this text in half and reads `` `Judgment` `` as the status of nothing.
    #[test]
    fn an_escaped_pipe_stays_inside_the_task_text() {
        let t2 = tasks(SPEC).into_iter().find(|t| t.id == "T2");
        assert_eq!(
            t2.as_ref().map(|t| t.text.as_str()),
            Some("a wip task, `Mechanical`|`Judgment`")
        );
        assert_eq!(t2.map(|t| t.cites), Some(vec!["V1".to_owned()]));
    }

    /// `-` is FORMAT.md's EMPTY cell, not a citation of a rule called `-`.
    #[test]
    fn an_empty_cites_cell_is_no_citations() {
        let t2a = tasks(SPEC).into_iter().find(|t| t.id == "T2a");
        assert_eq!(t2a.map(|t| t.cites), Some(Vec::new()));
    }

    /// A milestone row is furniture, not a task: it has no status field, and
    /// counting it would put a row in the report that no `check` rule reads.
    #[test]
    fn a_milestone_row_is_not_a_task() {
        assert!(!ids(SPEC).iter().any(|id| id.starts_with('M')));
        assert_eq!(ids(SPEC).len(), 3);
    }

    /// THE distinguishing case: no `§T` at all reports differently from a
    /// `§T` whose rows are all done. Both exit 0, so the payload is what a
    /// consumer reads -- and an empty array is an answer where silence is
    /// indistinguishable from a crash.
    #[test]
    fn no_tasks_reads_differently_from_all_done() {
        let none = "## \u{a7}V INVARIANTS\nV1: **a rule.** alone here.\n";
        assert!(report(none, false).starts_with("tasks: none"), "empty spec");
        assert_eq!(json("f", none), "{\"file\":\"f\",\"tasks\":[]}\n");
        let done = SPEC.replace("|~|", "|x|").replace("|.|", "|x|");
        assert!(report(&done, false).contains("3 rows -- 0 ., 0 ~, 3 x"));
        assert!(json("f", &done).contains("\"status\":\"x\""));
    }

    /// The JSON is the contract: every field the consumer indexes on, in one
    /// object on one line, with the escape a hand-written encoder must own.
    #[test]
    fn the_json_carries_id_status_text_and_cites() {
        let out = json("SPEC.md", SPEC);
        assert!(
            out.starts_with("{\"file\":\"SPEC.md\",\"tasks\":["),
            "{out}"
        );
        assert!(out.contains("{\"id\":\"T1\",\"status\":\"x\""), "{out}");
        assert!(out.contains("\"cites\":[\"V1\"]"), "{out}");
        assert!(out.contains("\"cites\":[]"), "T2a cites nothing: {out}");
        assert_eq!(out.lines().count(), 1, "one line per run: {out}");
    }

    /// A quote in a task text would otherwise emit JSON no parser accepts.
    /// The encoder is shared with `render` (V7), so this asserts the reuse
    /// rather than a second escaping.
    #[test]
    fn a_quote_in_a_task_is_escaped() {
        let text = "## \u{a7}T TASKS\nT1|.|a \"quoted\" task|-\n";
        assert!(json("f", text).contains(r#"a \"quoted\" task"#));
    }

    /// The human rendering leads with the counts, so a reader gets the
    /// answer before the list -- and `--verbose` stops truncating.
    #[test]
    fn the_report_counts_first_then_lists_each_row() {
        let out = report(SPEC, false);
        assert!(out.starts_with("tasks: 3 rows -- 1 ., 1 ~, 1 x\n"), "{out}");
        assert!(
            out.contains("task T2a: . -- a todo task riding T2\n"),
            "{out}"
        );
    }

    #[test]
    fn verbose_prints_the_whole_task_text() {
        let long = "z".repeat(80);
        let text = format!("## \u{a7}T TASKS\nT1|.|{long}|-\n");
        assert!(report(&text, false).contains("..."), "truncates");
        assert!(!report(&text, true).contains("..."), "verbose does not");
    }

    /// It names the set and its order, and NOTHING about which row matters.
    /// SELECT is the caller's (V6), so the report must not contain a word
    /// that reads as a recommendation.
    #[test]
    fn the_report_recommends_no_next_task() {
        let out = report(SPEC, false).to_lowercase();
        for word in ["next", "should", "recommend", "start with"] {
            assert!(
                !out.contains(word),
                "{word} leaked into the report: {out}"
            );
        }
    }

    /// A status outside the set is REPORTED, not dropped. V25 is what calls
    /// it a violation; hiding the row here would leave a consumer reading a
    /// backlog with a hole in it and no way to know.
    #[test]
    fn a_status_outside_the_set_is_still_enumerated() {
        let text = "## \u{a7}T TASKS\nT1|q|a task with a bad status|-\n";
        let rows = tasks(text);
        assert_eq!(rows.first().map(|t| t.status.as_str()), Some("q"));
    }
}
