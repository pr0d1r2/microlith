//! `archive`: move a finished task's TEXT out, and leave its ROW behind.
//!
//! The third mutation in this crate, and the one with the narrowest promise.
//! `fmt` reshapes without changing content (V1); `migrate` changes content on
//! purpose and proves every word survives; this one MOVES content between two
//! files and proves the move rather than the edit.
//!
//! WHY A STUB RATHER THAN A DELETION. The obvious compaction -- lift the `x`
//! rows out -- makes every milestone that claimed them name a task with no
//! row, which is V15 going red on the whole file. Milestone-at-a-time
//! archiving was measured against this spec and qualified three milestones of
//! twelve, because one held row blocks the whole claim. So the id, the status
//! and the citations stay exactly where they were and only the TEXT moves:
//! V12 still sees the id, V14 still sees it in order, V15 still finds the row
//! its milestone claims, and the citation graph `derive` reads is unchanged.
//! T39 asked for "not a deletion -- a citation to an archived row must still
//! resolve", and a stub makes that true by construction.
//!
//! WHAT IS HELD BACK. A row carrying a V16 record is never archived. Those
//! records are the one thing in a spec that compaction is explicitly
//! forbidden to trade for bytes, and five of this repo's own 33 done rows
//! carry one.

use crate::check::Record;
use crate::id::{Id, at_line_start, cells};

/// The file rows are moved to, beside the spec they came from.
///
/// A sibling rather than a path, for `DEFAULT_PATH`'s reason: FORMAT.md puts
/// the spec at a project root, so the archive that belongs to it lives there
/// too, and the stub can name it without knowing a directory layout.
pub const ARCHIVE: &str = "SPEC-ARCHIVE.md";

/// What a stub's text cell says, in full.
///
/// A CONSTANT because it is read as well as written: a row already carrying
/// it has been archived, which is what makes a second run a no-op instead of
/// a second move.
pub const MOVED: &str = "ARCHIVED to SPEC-ARCHIVE.md";

/// The header the archive file opens with.
///
/// It says where the rows came from and, in the sentences that matter, what
/// this file is NOT. A `V6` inside an archived row points into `SPEC.md`, so
/// `check` run here reports it dangling -- true of this file and the wrong
/// thing to conclude, which is why the header says so before anyone tries.
///
/// Its OWN references are backticked, because V13 reads a backticked id as a
/// literal. A header that explained the dangling citations by adding two
/// more would be a poor way to make the point.
pub const HEADER: &str = "\
# SPEC ARCHIVE

Task rows moved out of `SPEC.md` by `mth archive`. An id is never reused
(`V12`), so a citation to an archived row still resolves -- here.

This is a SINK, not a spec. The citations inside these rows point into
`SPEC.md`, so `mth check` on this file reports every one of them as dangling,
correctly and uselessly. The verb that reads it is `mth tasks`.

## \u{a7}T TASKS
";

/// One row's move: the id, the text leaving, and the stub replacing it.
#[derive(Debug, PartialEq, Eq)]
pub struct Move {
    /// The row's id, as it is written.
    pub id: String,
    /// The row exactly as it stood in the spec.
    pub row: String,
    /// What is written in its place.
    pub stub: String,
}

/// The moves this spec offers, in file order.
///
/// Pure, and the same answer `archive` acts on -- so `--check` reports what a
/// run would do rather than a second opinion about it (V7).
#[must_use]
pub fn moves(spec: &str, records: &[Record]) -> Vec<Move> {
    let mut out = Vec::new();
    let mut fenced = false;
    for line in spec.lines() {
        if crate::format::is_fence(line) {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if let Some(m) = movable(line, records) {
            out.push(m);
        }
    }
    out
}

/// This line's move, if it is a done task row that may leave.
///
/// FOUR conditions, and each one is a way the fold could lose something:
/// it is a `T` row at all · its status is `x`, so the text is history rather
/// than live work · it has not already been archived, which is what makes a
/// second run a no-op · and no V16 record is named on it.
fn movable(line: &str, records: &[Record]) -> Option<Move> {
    let id = at_line_start(line).filter(|i| i.kind == 'T')?;
    let f = cells(line.trim_end());
    if f.len() != 4 || f.get(1)?.trim() != "x" {
        return None;
    }
    if f.get(2)?.trim() == MOVED || holds_a_record(&id, records) {
        return None;
    }
    Some(Move {
        id: id.label(),
        row: line.trim_end().to_owned(),
        stub: format!("{}|x|{MOVED}|{}", id.label(), f.get(3)?),
    })
}

/// Whether a V16 record is named on this id.
///
/// Keyed on the ID rather than on the marker text, which holds back a row
/// whose marker happens to live elsewhere. That is the SAFE direction: the
/// cost is one row that could have been archived, and the cost of being
/// wrong the other way is a closed decision nobody can audit (V16).
fn holds_a_record(id: &Id, records: &[Record]) -> bool {
    let label = id.label();
    records.iter().any(|(owner, _)| *owner == label)
}

/// The spec and the archive, after moving every row that may leave.
///
/// # Errors
///
/// If the move cannot be proven -- a row that did not arrive, an id that did
/// not stay, or a citation cell the stub failed to carry. Every one of those
/// is a bug in this module rather than a property of the input, which is why
/// the answer is a refusal to write rather than a warning.
pub fn archive(
    spec: &str,
    archive: &str,
    records: &[Record],
) -> Result<(String, String), String> {
    let moving = moves(spec, records);
    let folded = stubbed(spec, &moving);
    let stored = merged(archive, &moving)?;
    proven(spec, &moving, (&folded, &stored))?;
    Ok((folded, stored))
}

/// The spec with each moved row replaced by its stub.
fn stubbed(spec: &str, moving: &[Move]) -> String {
    let mut out = String::new();
    for line in spec.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\r', '\n']);
        match moving.iter().find(|m| m.row == trimmed) {
            Some(m) => {
                out.push_str(&m.stub);
                out.push_str(line.strip_prefix(trimmed).unwrap_or("\n"));
            }
            None => out.push_str(line),
        }
    }
    out
}

/// The archive with the new rows folded in, in id order (V14).
///
/// MERGED rather than appended, because the rows arriving are not always
/// later than the rows already there: archiving twice, with a row finished
/// in between, puts a lower id after a higher one and the file the tool
/// produced would fail the tool's own sort rule.
fn merged(archive: &str, moving: &[Move]) -> Result<String, String> {
    let mut rows: Vec<String> = archive
        .lines()
        .filter(|l| at_line_start(l).is_some_and(|i| i.kind == 'T'))
        .map(str::to_owned)
        .collect();
    for m in moving {
        if rows.iter().any(|r| label_of(r) == m.id) {
            return Err(format!("{} is already archived", m.id));
        }
        rows.push(m.row.clone());
    }
    rows.sort_by_key(|r| at_line_start(r).map(|i| i.sort_key()));
    Ok(format!("{HEADER}\n{}\n", rows.join("\n")))
}

fn label_of(row: &str) -> String {
    at_line_start(row).map(|i| i.label()).unwrap_or_default()
}

/// The move, proven before anything is written.
///
/// THREE claims, each one a way the fold could lose something quietly:
/// every moved row ARRIVED, byte for byte · every id that was in the spec is
/// STILL in it, so no row was dropped rather than stubbed · and each stub
/// carries the citations its row carried, so the graph `derive` reads and
/// V13 checks is the same graph afterwards.
fn proven(
    spec: &str,
    moving: &[Move],
    out: (&str, &str),
) -> Result<(), String> {
    let (folded, stored) = out;
    if let Some(m) = moving.iter().find(|m| !stored.contains(&m.row)) {
        return Err(format!("{} did not arrive in the archive", m.id));
    }
    let before = ids(spec);
    let after = ids(folded);
    if before != after {
        return Err(format!("ids changed: {before:?} -> {after:?}"));
    }
    match moving.iter().find(|m| !carries_its_cites(m)) {
        Some(m) => Err(format!("{}'s stub dropped its citations", m.id)),
        None => Ok(()),
    }
}

/// Whether the stub's fourth cell is the row's fourth cell.
///
/// Compared as CELLS rather than as text, because that is the reading V42
/// defines: `cells` splits on the pipes a row OWNS and leaves an escaped one
/// inside the field it belongs to. Which is also why the stub carries the
/// cell verbatim rather than re-escaping it -- an encode over an already
/// encoded cell is the doubling B36 recorded, one layer out.
fn carries_its_cites(m: &Move) -> bool {
    cells(&m.stub).get(3) == cells(&m.row).get(3)
}

/// Every declared id in the text, in order.
fn ids(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(at_line_start)
        .map(|i| i.label())
        .collect()
}

/// What a run would move, for `--check`.
///
/// Report-only and exit 0 (V10): a spec with finished work in it is an
/// ORDINARY spec, not a defective one. The gates are named and closed, and
/// whether the file is big enough to fold is the caller's threshold to apply
/// -- this crate cannot count tokens and says so (§C).
#[must_use]
pub fn report(spec: &str, records: &[Record]) -> String {
    let moving = moves(spec, records);
    if moving.is_empty() {
        return format!(
            "mth: nothing to archive -- no finished rows to move to {ARCHIVE}\n"
        );
    }
    let held = held_back(spec, records);
    let ids: Vec<&str> = moving.iter().map(|m| m.id.as_str()).collect();
    let chars: usize = moving.iter().map(|m| m.row.len()).sum();
    format!(
        "mth: {} {} would move to {ARCHIVE} ({chars} chars): {}\n{held}",
        moving.len(),
        if moving.len() == 1 { "row" } else { "rows" },
        ids.join(", ")
    )
}

/// The done rows a run would NOT move, named with the reason.
///
/// Said out loud because the alternative is a count that does not add up: a
/// reader who knows the spec holds 33 finished rows and sees 28 move has to
/// work out which five stayed and why. V16 is exactly the rule that must not
/// be applied silently.
///
/// Only the rows the run would OTHERWISE have taken -- asked by running the
/// same eligibility test with an empty record list. A pending row carrying a
/// record was never going to move, so naming it here would inflate the count
/// this line exists to reconcile.
fn held_back(spec: &str, records: &[Record]) -> String {
    let held: Vec<String> = spec
        .lines()
        .filter(|l| movable(l, &[]).is_some())
        .filter_map(at_line_start)
        .filter(|i| holds_a_record(i, records))
        .map(|i| i.label())
        .collect();
    if held.is_empty() {
        return String::new();
    }
    format!(
        "mth: {} held back -- {} carry a closed-option record (V16)\n",
        held.len(),
        held.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(pairs: &[(&str, &str)]) -> Vec<Record> {
        pairs
            .iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect()
    }

    const SPEC: &str = "\
# SPEC

## \u{a7}T TASKS

T1|x|the first thing, at length|V1,V2
T2|.|not done yet|V3
T3|x|the third thing|-
";

    /// The whole promise in one assertion: the TEXT moves, the ROW stays.
    #[test]
    fn a_done_row_keeps_its_id_status_and_cites() {
        let (spec, arch) = archive(SPEC, "", &records(&[])).unwrap_or_default();
        assert!(spec.contains(&format!("T1|x|{MOVED}|V1,V2")), "{spec}");
        assert!(spec.contains(&format!("T3|x|{MOVED}|-")), "{spec}");
        assert!(
            arch.contains("T1|x|the first thing, at length|V1,V2"),
            "{arch}"
        );
        assert!(arch.contains("T3|x|the third thing|-"), "{arch}");
    }

    /// A row that is not FINISHED is not history, so it is left whole.
    #[test]
    fn an_unfinished_row_is_left_alone() {
        let (spec, arch) = archive(SPEC, "", &records(&[])).unwrap_or_default();
        assert!(spec.contains("T2|.|not done yet|V3"), "{spec}");
        assert!(!arch.contains("not done yet"), "{arch}");
    }

    /// PLANTED (V16): a row carrying a closed-option record never moves.
    ///
    /// The one thing compaction is explicitly forbidden to trade for bytes.
    /// Measured on this repo's own spec: five of 33 finished rows carry one.
    #[test]
    fn a_row_carrying_a_record_is_held_back() {
        let recs = records(&[("T1", "at length")]);
        let (spec, arch) = archive(SPEC, "", &recs).unwrap_or_default();
        assert!(spec.contains("T1|x|the first thing, at length|V1,V2"));
        assert!(!arch.contains("the first thing"), "{arch}");
        assert!(report(SPEC, &recs).contains("held back"), "said out loud");
    }

    /// Archiving twice is archiving once: the stub is READ as well as
    /// written, so the second run finds nothing to move.
    #[test]
    fn a_second_run_moves_nothing() {
        let (once, arch) = archive(SPEC, "", &records(&[])).unwrap_or_default();
        assert!(moves(&once, &records(&[])).is_empty(), "already folded");
        let (twice, again) =
            archive(&once, &arch, &records(&[])).unwrap_or_default();
        assert_eq!(twice, once);
        assert_eq!(again, arch);
    }

    /// Rows arriving are not always LATER than the rows already there, so
    /// the archive is merged and re-sorted rather than appended to (V14).
    #[test]
    fn a_later_run_sorts_a_lower_id_into_place() {
        let first = "T9|x|ninth|-\n";
        let later = "# SPEC\n\n## \u{a7}T TASKS\n\nT2|x|second|-\n";
        let (_, arch) = archive(later, &format!("{HEADER}\n{first}"), &[])
            .unwrap_or_default();
        let rows: Vec<&str> = arch
            .lines()
            .filter(|l| at_line_start(l).is_some())
            .collect();
        assert_eq!(rows, vec!["T2|x|second|-", "T9|x|ninth|-"], "{arch}");
    }

    /// The archive is a legal \u{a7}T section, so the rows in it stay
    /// readable by the same tool that put them there.
    #[test]
    fn the_archive_is_a_spec_the_row_rules_still_read() {
        let (_, arch) = archive(SPEC, "", &records(&[])).unwrap_or_default();
        assert!(arch.starts_with("# SPEC ARCHIVE"), "{arch}");
        assert!(arch.contains("## \u{a7}T TASKS"), "{arch}");
        let listed = crate::tasks::json("a", &arch);
        assert!(listed.contains("\"T1\""), "{listed}");
        assert!(listed.contains("\"T3\""), "{listed}");
    }

    /// PLANTED (V18): the proof refuses a move that would lose a row.
    ///
    /// The archive already holds this id, so folding it in would leave two
    /// rows with one id -- V12's defect, written by the tool that checks it.
    #[test]
    fn re_archiving_one_id_is_refused_rather_than_duplicated() {
        let held = format!("{HEADER}\nT1|x|the first thing, at length|V1,V2\n");
        let err = archive(SPEC, &held, &records(&[])).err();
        assert_eq!(err.as_deref(), Some("T1 is already archived"));
    }

    /// A move whose row never reached the archive is refused.
    ///
    /// PLANTED (V18), and with it the two below: these are arms a correct
    /// module never reaches, which is exactly why they are worth running. A
    /// proof whose failure path has never fired is indistinguishable from
    /// one that cannot fire, and the day something else here breaks is the
    /// day it has to.
    #[test]
    fn the_proof_refuses_a_row_that_did_not_arrive() {
        let m = a_move();
        let folded = format!("{}\n", m.stub);
        let out =
            proven("T1|x|text|V1\n", std::slice::from_ref(&m), (&folded, ""));
        assert_eq!(
            out.err().as_deref(),
            Some("T1 did not arrive in the archive")
        );
    }

    /// A row DROPPED rather than stubbed is refused: the id has to stay.
    #[test]
    fn the_proof_refuses_an_id_that_stopped_being_there() {
        let m = a_move();
        let stored = format!("{}\n", m.row);
        let out =
            proven("T1|x|text|V1\n", std::slice::from_ref(&m), ("\n", &stored));
        assert!(out.err().is_some_and(|e| e.starts_with("ids changed")));
    }

    /// A stub that did not carry its citations is refused: the graph
    /// `derive` reads and V13 checks must be the same graph afterwards.
    #[test]
    fn the_proof_refuses_a_stub_that_dropped_its_citations() {
        let m = Move {
            stub: "T1|x|s|-".to_owned(),
            ..a_move()
        };
        let stored = format!("{}\n", m.row);
        let out = proven(
            "T1|x|text|V1\n",
            std::slice::from_ref(&m),
            ("T1|x|s|-\n", &stored),
        );
        assert_eq!(
            out.err().as_deref(),
            Some("T1's stub dropped its citations")
        );
    }

    /// One well-formed move, for the three refusals to break in one way each.
    fn a_move() -> Move {
        Move {
            id: "T1".to_owned(),
            row: "T1|x|text|V1".to_owned(),
            stub: format!("T1|x|{MOVED}|V1"),
        }
    }

    /// A fenced example is not a declaration (B14), so a row inside one is
    /// never moved -- it is somebody's illustration of the format.
    #[test]
    fn a_row_inside_a_fence_is_never_moved() {
        let text = "# SPEC\n\n```\nT1|x|an example|-\n```\n";
        assert_eq!(moves(text, &records(&[])), vec![]);
    }

    /// A literal pipe survives the round trip (V42, V43).
    ///
    /// `cells` leaves an escape inside the field it belongs to, so the cell
    /// is carried across VERBATIM. Written the other way first -- escaping
    /// the cell on the way into the stub -- and the proof caught it: one
    /// backslash became three, which is B36's doubling one layer out.
    #[test]
    fn an_escaped_pipe_survives_the_stub() {
        let text = "## \u{a7}T TASKS\n\nT1|x|a task|`a\\|b`\n";
        let (spec, _) = archive(text, "", &records(&[])).unwrap_or_default();
        let stub = spec.lines().find(|l| l.starts_with("T1|"));
        let want = format!("T1|x|{MOVED}|`a\\|b`");
        assert_eq!(stub, Some(want.as_str()));
        assert_eq!(cells(stub.unwrap_or_default()).len(), 4);
    }

    /// The COMPANION (V18): a spec with no finished rows says so and moves
    /// nothing, so a guard that archived everything would fail here.
    #[test]
    fn a_spec_with_no_finished_rows_says_so() {
        let text = "## \u{a7}T TASKS\n\nT1|.|pending|-\nT2|~|started|-\n";
        assert_eq!(moves(text, &records(&[])), vec![]);
        let (spec, arch) = archive(text, "", &records(&[])).unwrap_or_default();
        assert_eq!(spec, text);
        assert!(report(text, &records(&[])).contains("nothing to archive"));
        assert!(!arch.contains("T1"), "{arch}");
    }
}
