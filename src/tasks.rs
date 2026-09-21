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

use crate::rows::Row;

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
    /// The milestone claiming this row, if one does (V52).
    ///
    /// `None` is TWO facts and the row alone cannot tell them apart: a spec
    /// that declares no milestones at all, and a row that every declared
    /// milestone left out. The first is V15 opting out; the second is V15
    /// firing. Which one it is, the DOCUMENT says.
    pub milestone: Option<String>,
}

/// Every `§T` row, in V14 order.
///
/// The ROW is read by `rows`, shared with every other pipe section (V7).
/// What §T calls those three cells is the part that lives here.
pub(crate) fn tasks(text: &str) -> Vec<Task> {
    let owners = crate::check::milestones(text);
    crate::rows::of(text, 'T')
        .into_iter()
        .map(|row| one(&owners, row))
        .collect()
}

/// The milestone claiming this task NUMBER, if one does.
///
/// By number rather than by label, because a suffixed row RIDES its base
/// (V14): `T7a` belongs to whichever milestone claims `7`, and a lookup on
/// the written id would find nothing and call it unclaimed.
///
/// The partition comes from `check::milestones`, so a consumer reading this
/// field and a consumer calling that function get the same answer -- and
/// `check` reads the same cell when V15 fires (V7).
fn claimed_by(owners: &[(String, Vec<u32>)], num: u32) -> Option<String> {
    owners
        .iter()
        .find(|(_, claimed)| claimed.contains(&num))
        .map(|(id, _)| id.clone())
}

/// One row, under §T's names for its three cells.
fn one(owners: &[(String, Vec<u32>)], row: Row) -> Task {
    let [status, text, cites] = row.cells;
    Task {
        milestone: claimed_by(owners, row.num),
        id: row.id,
        status,
        text,
        cites: crate::rows::cites(&cites),
    }
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
    let mut out = head(&rows, unread(text));
    for t in &rows {
        let shown = crate::rows::shown(&t.text, full);
        out.push_str(&format!("task {}: {} -- {shown}\n", t.id, t.status));
    }
    out
}

/// How many §T rows this build could not read (V49).
fn unread(text: &str) -> usize {
    crate::rows::unread(text, 'T')
}

/// The count line: how many rows, how many of each status, and how many
/// rows were THERE but unreadable.
///
/// The last part is why `none` is not the whole answer. A spec whose §T is
/// written in a dialect reported `none` and exit 0, which a caller cannot
/// tell from a spec with an empty backlog -- and the rows were sitting right
/// there. Report-only still (V10): this is an ANSWER, not a verdict, and
/// `check` is where it becomes one.
fn head(rows: &[Task], unread: usize) -> String {
    if rows.is_empty() {
        return crate::rows::none_read("tasks", 'T', unread);
    }
    let counts: Vec<String> = tally(rows)
        .iter()
        .map(|(s, n)| format!("{n} {s}"))
        .collect();
    format!(
        "tasks: {} rows -- {}{}\n",
        rows.len(),
        counts.join(", "),
        crate::rows::also(unread)
    )
}

/// The machine rendering: one object, `tasks` in V14 order.
///
/// ALWAYS emitted, including `"tasks":[]` for a spec with no `§T`. That is
/// the one place this departs from `render::json`, and deliberately: silence
/// is success for a GATE, where an empty stream means nothing fired. Here an
/// empty stream is indistinguishable from a crash, and the consumer's whole
/// question is whether the backlog is empty or merely unreadable.
pub(crate) fn json(file: &str, text: &str) -> String {
    let rows = tasks(text);
    let items: Vec<String> = rows.iter().map(one_json).collect();
    format!(
        "{{\"file\":{},\"tasks\":[{}],\"unread\":{},\
         \"declares_milestones\":{},\"milestones\":[{}]}}\n",
        crate::render::quote(file),
        items.join(","),
        unread(text),
        crate::check::uses_milestones(text),
        milestones_json(text, &rows)
    )
}

/// Each declared milestone, in FILE order: its id, the version it ships as
/// (V53), the rows it claims in V14 order, and how many are not yet `x`.
///
/// `ships` is `null` where the scope names no version, so "unversioned" is a
/// value rather than a missing key. `pending` counts every row whose status
/// is not `x`, including one outside the set: a row nobody can call done is
/// not done, and V25 is where its status becomes a finding. The array is
/// EMPTY, never absent, when the spec declares none -- a key that comes and
/// goes makes every reader test before indexing (V52).
fn milestones_json(text: &str, rows: &[Task]) -> String {
    let items: Vec<String> = crate::check::ships(text)
        .into_iter()
        .map(|(id, ships)| milestone_json(&id, ships.as_deref(), rows))
        .collect();
    items.join(",")
}

/// One milestone. Its rows are the ones whose `milestone` field names it, so
/// the array and the per-row field cannot disagree about who owns what (V7).
fn milestone_json(id: &str, ships: Option<&str>, rows: &[Task]) -> String {
    let owned: Vec<&Task> = rows
        .iter()
        .filter(|t| t.milestone.as_deref() == Some(id))
        .collect();
    let ids: Vec<String> =
        owned.iter().map(|t| crate::render::quote(&t.id)).collect();
    format!(
        "{{\"id\":{},\"ships\":{},\"tasks\":[{}],\"pending\":{}}}",
        crate::render::quote(id),
        claimed_json(ships),
        ids.join(","),
        owned.iter().filter(|t| t.status != "x").count()
    )
}

fn one_json(t: &Task) -> String {
    let cites: Vec<String> =
        t.cites.iter().map(|c| crate::render::quote(c)).collect();
    format!(
        "{{\"id\":{},\"status\":{},\"text\":{},\"cites\":[{}],\
         \"milestone\":{}}}",
        crate::render::quote(&t.id),
        crate::render::quote(&t.status),
        crate::render::quote(&t.text),
        cites.join(","),
        claimed_json(t.milestone.as_deref())
    )
}

/// The milestone, or JSON's own word for "no answer here".
///
/// `null` rather than `""` or a missing key: an empty string is a value
/// somebody has to know is special, and a key that comes and goes makes
/// every reader test for it before indexing. `declares_milestones` beside
/// the array says which of `null`'s two meanings applies.
fn claimed_json(milestone: Option<&str>) -> String {
    milestone.map_or_else(|| "null".to_owned(), crate::render::quote)
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
        assert_eq!(
            json("f", none),
            "{\"file\":\"f\",\"tasks\":[],\"unread\":0,\
             \"declares_milestones\":false,\"milestones\":[]}\n"
        );
        let done = SPEC.replace("|~|", "|x|").replace("|.|", "|x|");
        assert!(report(&done, false).contains("3 rows -- 0 ., 0 ~, 3 x"));
        assert!(json("f", &done).contains("\"status\":\"x\""));
    }

    /// V52: each row carries the milestone that claims it.
    ///
    /// The point of the field: a consumer filtering §T by milestone had the
    /// partition only through the library, so a CLI caller re-read the
    /// `| M<n> |` grammar itself -- the duplication V7 exists to prevent.
    #[test]
    fn each_row_carries_the_milestone_claiming_it() {
        let text = "## \u{a7}T TASKS\n\n\
            | M1 | first | T1-T2 | done |\n\
            | M2 | second | T4 | done |\n\
            T1|x|one|-\nT2|.|two|-\nT4|.|four|-\n";
        let out = json("f", text);
        assert!(out.contains("\"id\":\"T1\",\"status\":\"x\",\"text\":\"one\",\"cites\":[],\"milestone\":\"M1\""), "{out}");
        assert!(out.contains("\"id\":\"T2\",\"status\":\".\",\"text\":\"two\",\"cites\":[],\"milestone\":\"M1\""), "{out}");
        assert!(out.contains("\"id\":\"T4\",\"status\":\".\",\"text\":\"four\",\"cites\":[],\"milestone\":\"M2\""), "{out}");
    }

    /// A suffixed row RIDES its base (V14), so `T7a` is in whichever
    /// milestone claims `7`. Looking it up by the WRITTEN id would find
    /// nothing and report it unclaimed -- a wrong answer that reads exactly
    /// like a right one.
    #[test]
    fn a_suffixed_row_rides_its_base_into_a_milestone() {
        let text = "## \u{a7}T TASKS\n\n\
            | M1 | first | T7 | done |\n\
            T7|x|the base|-\nT7a|.|rides it|-\n";
        let out = json("f", text);
        assert_eq!(out.matches("\"milestone\":\"M1\"").count(), 2, "{out}");
    }

    /// `null`'s TWO meanings, kept apart by the document rather than by the
    /// row: a spec that declares no milestones has opted out of V15, and a
    /// spec that declares some and left a row out is V15 FIRING. Both put
    /// `null` on the row; only `declares_milestones` says which.
    #[test]
    fn null_is_told_apart_by_the_document_not_the_row() {
        let opted_out = "## \u{a7}T TASKS\n\nT1|x|a task|-\n";
        let out = json("f", opted_out);
        assert!(out.contains("\"milestone\":null"), "{out}");
        assert!(out.contains("\"declares_milestones\":false"), "{out}");

        let unclaimed = "## \u{a7}T TASKS\n\n\
            | M1 | first | T1 | done |\n\
            T1|x|claimed|-\nT2|.|left out|-\n";
        let out = json("f", unclaimed);
        assert!(out.contains("\"declares_milestones\":true"), "{out}");
        assert!(out.contains("\"id\":\"T2\""), "{out}");
        assert_eq!(out.matches("\"milestone\":null").count(), 1, "{out}");
    }

    /// The COMPANION (V18): a MILESTONE row is not a task, so it never
    /// appears as one -- and the field did not quietly turn it into one.
    #[test]
    fn a_milestone_row_still_never_becomes_a_task() {
        let text = "## \u{a7}T TASKS\n\n\
            | M1 | first | T1 | done |\nT1|x|one|-\n";
        let out = json("f", text);
        // Only the `tasks` array: since V53 the milestone itself is listed,
        // under `milestones`, where an `M1` id is the right answer.
        let tasks = out.split("\"unread\"").next().unwrap_or_default();
        assert!(!tasks.contains("\"id\":\"M1\""), "{out}");
        assert_eq!(tasks.matches("\"id\":").count(), 1, "{out}");
    }

    /// V53: each milestone carries the version it ships as, the rows it
    /// claims -- a suffix riding its base -- and how many are not yet done.
    /// Milestone numbers are NOT release order here, so the array keeps the
    /// FILE's and leaves ordering releases to the caller.
    #[test]
    fn each_milestone_carries_its_version_rows_and_pending_count() {
        let text = "## \u{a7}T TASKS\n\n\
            | M9 | later; ships as `0.8.0` | T7 | done |\n\
            | M2 | never shipped alone | T1-T2 | done |\n\
            T1|x|one|-\nT2|~|two|-\nT7|x|seven|-\nT7a|.|rides it|-\n";
        let out = json("f", text);
        assert!(
            out.contains(
                "\"milestones\":[\
                 {\"id\":\"M9\",\"ships\":\"0.8.0\",\"tasks\":[\"T7\",\"T7a\"],\"pending\":1},\
                 {\"id\":\"M2\",\"ships\":null,\"tasks\":[\"T1\",\"T2\"],\"pending\":1}]"
            ),
            "{out}"
        );
    }

    /// The companion (V18): a declared milestone claiming nothing is still
    /// listed, with an empty array and zero pending -- dropping it would
    /// read exactly like a spec that never declared it.
    #[test]
    fn an_empty_milestone_is_listed_not_dropped() {
        let text = "## \u{a7}T TASKS\n\n\
            | M1 | first; ships as `1.0.0` | T1 | done |\n\
            | M2 | later |  | tbd |\nT1|.|one|-\n";
        let out = json("f", text);
        assert!(
            out.contains(
                "{\"id\":\"M2\",\"ships\":null,\"tasks\":[],\"pending\":0}"
            ),
            "{out}"
        );
        assert!(out.contains("\"ships\":\"1.0.0\""), "{out}");
    }

    /// THE issue, in one test: three states a caller must tell apart.
    ///
    /// An empty backlog, a readable one, and a §T whose rows are there but
    /// in a dialect. The third used to be spelled exactly like the first --
    /// `"tasks":[]`, exit 0 -- with two id-shaped rows sitting right there,
    /// and the only verb that knew was one a reader has no reason to run.
    #[test]
    fn an_unread_section_is_not_spelled_like_an_empty_one() {
        let dialect = "## \u{a7}T TASKS\n\n\
            | T1 | x | first | - |\n| T2 | . | second | - |\n";
        let empty = "## \u{a7}T TASKS\n\nnothing here yet.\n";
        assert!(json("f", dialect).contains("\"unread\":2"), "counted");
        assert!(json("f", empty).contains("\"unread\":0"), "really empty");
        assert!(
            json("f", dialect).contains("\"tasks\":[]"),
            "still enumerates"
        );
        assert!(report(dialect, false).starts_with("tasks: none READ"));
        assert!(report(empty, false).starts_with("tasks: none --"));
    }

    /// A section that is PARTLY readable says so beside the rows it read,
    /// rather than reporting a count the caller would believe is the whole.
    #[test]
    fn a_partly_read_section_says_how_many_it_missed() {
        let mixed =
            "## \u{a7}T TASKS\n\nT1|x|read fine|-\n| T2 | . | not | - |\n";
        let said = report(mixed, false);
        assert!(said.contains("1 more unread"), "{said}");
        assert!(json("f", mixed).contains("\"unread\":1"), "{mixed}");
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
