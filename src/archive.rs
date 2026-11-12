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
use crate::format::continuation;
use crate::id::{Id, at_line_start, cells};

/// The file rows are moved to, beside the spec they came from.
///
/// A sibling rather than a path, for `DEFAULT_PATH`'s reason: FORMAT.md puts
/// the spec at a project root, so the archive that belongs to it lives there
/// too, and the stub can name it without knowing a directory layout.
pub const ARCHIVE: &str = "SPEC-ARCHIVE.md";

/// What a newly written stub's text cell says.
///
/// A CONSTANT because it is read as well as written: a row already carrying
/// it has been archived, which is what makes a second run a no-op instead of
/// a second move.
pub const MOVED: &str = "A";

/// The marker written by releases before the compact form.
///
/// It remains readable so archiving an older spec is still idempotent.
const LEGACY_MOVED: &str = "ARCHIVED to SPEC-ARCHIVE.md";

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

Task and fixed bug rows moved out of `SPEC.md` by `mth archive`. An id is never reused
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
///
/// The newest `keep` finished rows of each kind stay whole. "Newest" is by
/// numeric id, and counts finished rows already stubbed as well as those still
/// whole, so the kept set does not slide forward as rows are archived and a
/// second run is a no-op. `keep == 0` offers every row that may leave.
#[must_use]
pub fn moves(spec: &str, records: &[Record], keep: usize) -> Vec<Move> {
    let kept = newest_finished(spec, keep);
    let lines: Vec<&str> = spec.lines().collect();
    movable_lines(&lines, records, &kept)
}

fn movable_lines<'a>(
    lines: &'a [&'a str],
    records: &[Record],
    kept: &[String],
) -> Vec<Move> {
    let mut moving = Vec::new();
    let mut rest = lines;
    let mut fence = None;
    while let Some((line, tail)) = rest.split_first() {
        rest = tail;
        if advance_fence(line, &mut fence) || fence.is_some() {
            continue;
        }
        let (m, after) = move_with_continuations(line, rest, records);
        rest = after;
        if let Some(m) = m.filter(|m| !kept.contains(&m.id)) {
            moving.push(m);
        }
    }
    moving
}

fn move_with_continuations<'a>(
    line: &'a str,
    rest: &'a [&'a str],
    records: &[Record],
) -> (Option<Move>, &'a [&'a str]) {
    let Some(mut m) = movable(line, records) else {
        return (None, rest);
    };
    if m.id.starts_with('V') {
        let (lines, after) = continuations(line, rest);
        m.row = std::iter::once(m.row.as_str())
            .chain(lines)
            .collect::<Vec<_>>()
            .join("\n")
            .trim_end()
            .to_owned();
        return (Some(m), after);
    }
    (Some(m), rest)
}

fn continuations<'a>(
    mut previous: &'a str,
    mut rest: &'a [&'a str],
) -> (Vec<&'a str>, &'a [&'a str]) {
    let mut lines = Vec::new();
    while let Some((next, after)) = rest.split_first()
        && fence_marker(next).is_none()
        && continuation(previous, next)
    {
        lines.push(*next);
        previous = next;
        rest = after;
    }
    (lines, rest)
}

/// The lines of the spec that are not inside a fenced block.
fn outside_fences(spec: &str) -> impl Iterator<Item = &str> {
    let mut fence = None;
    spec.lines().filter(move |line| {
        if advance_fence(line, &mut fence) {
            return false;
        }
        fence.is_none()
    })
}

/// The labels of the newest `keep` finished `T` rows and `B` rows.
fn newest_finished(spec: &str, keep: usize) -> Vec<String> {
    let finished: Vec<Id> = outside_fences(spec)
        .filter(|line| is_finished(line))
        .filter_map(at_line_start)
        .collect();
    ['T', 'B']
        .into_iter()
        .flat_map(|kind| {
            let mut rows: Vec<&Id> =
                finished.iter().filter(|i| i.kind == kind).collect();
            rows.sort_by_key(|i| i.sort_key());
            let from = rows.len().saturating_sub(keep);
            rows.split_off(from)
        })
        .map(Id::label)
        .collect()
}

/// Whether this row is a finished `T` or a fixed `B`, stubbed or not.
fn is_finished(line: &str) -> bool {
    let Some(id) = at_line_start(line) else {
        return false;
    };
    let f = cells(line.trim_end());
    let cell = |n: usize| f.get(n).map_or("", |c| c.trim());
    f.len() == 4
        && match id.kind {
            'T' => cell(1) == "x",
            'B' => !cell(3).is_empty() && cell(3) != "-",
            _ => false,
        }
}

/// This line's move, if it is a done task row that may leave.
///
/// FOUR conditions, and each one is a way the fold could lose something:
/// it is a `T` row at all · its status is `x`, so the text is history rather
/// than live work · it has not already been archived, which is what makes a
/// second run a no-op · and no V16 record is named on it.
fn movable(line: &str, records: &[Record]) -> Option<Move> {
    // The record check comes first, so it binds every kind, `V` included.
    let id = at_line_start(line)
        .filter(|i| matches!(i.kind, 'T' | 'B' | 'V'))
        .filter(|i| !holds_a_record(i, records))?;
    if id.kind == 'V' {
        return movable_invariant(line, &id);
    }
    let f = cells(line.trim_end());
    if f.len() != 4 {
        return None;
    }
    match id.kind {
        'T' => movable_task(line, &id, &f),
        'B' => movable_bug(line, &id, &f),
        _ => None,
    }
}

fn movable_task(line: &str, id: &Id, f: &[&str]) -> Option<Move> {
    if f.get(1)?.trim() != "x" || is_archived_marker(f.get(2)?.trim()) {
        return None;
    }
    Some(Move {
        id: id.label(),
        row: line.trim_end().to_owned(),
        stub: format!("{}|x|{MOVED}|{}", id.label(), f.get(3)?),
    })
}

fn movable_bug(line: &str, id: &Id, f: &[&str]) -> Option<Move> {
    let cause = f.get(2)?.trim();
    let fix = f.get(3)?.trim();
    // `cause` and `fix` are prose cells, so either one may legitimately
    // contain `A`. The marker identifies a stub only as the pair this
    // module writes; checking either cell alone would silently skip a
    // real fixed bug whose fix happens to be the one-letter word.
    if fix.is_empty()
        || fix == "-"
        || (is_archived_marker(cause) && is_archived_marker(fix))
    {
        return None;
    }
    Some(Move {
        id: id.label(),
        row: line.trim_end().to_owned(),
        stub: format!("{}|{}|{MOVED}|{MOVED}", id.label(), f.get(1)?),
    })
}

/// A `§V` statement that is retired and not yet a stub.
///
/// Retired is `check::marked`'s reading, the one V41 gates on (V7): a mark
/// inside backticks or after prose is no mark, so that statement is live and
/// stays. The stub keeps the id and the mark -- the id so citations resolve,
/// the mark so V41 still checks the chain.
fn movable_invariant(line: &str, id: &Id) -> Option<Move> {
    let row = line.trim_end();
    let stub = invariant_stub(row, id)?;
    (stub != row).then(|| Move {
        id: id.label(),
        row: row.to_owned(),
        stub,
    })
}

/// The id as written, the marker word, and the mark -- for a marked line.
fn invariant_stub(row: &str, id: &Id) -> Option<String> {
    let winners = crate::check::marked(row);
    if winners.is_empty() {
        return None;
    }
    let from = row.find(id.kind)?;
    let to = from.saturating_add(row.get(from..)?.find(id.terminator)?);
    Some(format!(
        "{} {MOVED} [{} {}]",
        row.get(..=to)?,
        crate::check::SUPERSEDED_BY,
        winners.join(", ")
    ))
}

fn is_archived_marker(text: &str) -> bool {
    text == MOVED || text == LEGACY_MOVED
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
    keep: usize,
) -> Result<(String, String), String> {
    let moving = moves(spec, records, keep);
    let folded = stubbed(spec, &moving);
    let stored = merged(archive, &moving)?;
    proven(spec, &moving, (&folded, &stored))?;
    Ok((folded, stored))
}

/// The spec with each moved row replaced by its stub.
fn stubbed(spec: &str, moving: &[Move]) -> String {
    let mut out = String::new();
    let mut skip: Option<usize> = None;
    for line in spec.split_inclusive('\n') {
        if let Some(remaining) = skip.filter(|remaining| *remaining != 0) {
            skip = remaining.checked_sub(1);
            continue;
        }
        skip = None;
        stub_line(&mut out, line, moving, &mut skip);
    }
    out
}

fn stub_line(o: &mut String, l: &str, ms: &[Move], s: &mut Option<usize>) {
    let body = l.trim_end_matches(['\r', '\n']);
    let Some(m) = ms
        .iter()
        .find(|m| m.row.lines().next() == Some(body.trim_end()))
    else {
        o.push_str(l);
        return;
    };
    o.push_str(&m.stub);
    o.push_str(&l[body.len()..]);
    *s = continuation_count(m);
}

fn continuation_count(m: &Move) -> Option<usize> {
    m.row
        .lines()
        .count()
        .checked_sub(1)
        .filter(|remaining| *remaining != 0)
}

/// The archive with the new rows folded in, in id order (V14).
///
/// MERGED rather than appended, because the rows arriving are not always
/// later than the rows already there: archiving twice, with a row finished
/// in between, puts a lower id after a higher one and the file the tool
/// produced would fail the tool's own sort rule.
fn merged(archive: &str, moving: &[Move]) -> Result<String, String> {
    let kinds = sorted_rows(archive, moving)?;
    if archive.trim().is_empty() {
        return Ok(fresh_archive(&kinds));
    }
    if !has_section_heading(archive.lines(), 'T') {
        return Err(unreadable_archive());
    }
    Ok(kinds
        .into_iter()
        .filter(|(kind, _)| moving.iter().any(|m| m.id.starts_with(*kind)))
        .fold(archive.to_owned(), |out, (kind, rows)| {
            splice(&out, kind, &rows)
        }))
}

/// The rows of each kind, the moving ones included, in id order.
type Kinds = [(char, Vec<String>); 3];

fn sorted_rows(archive: &str, moving: &[Move]) -> Result<Kinds, String> {
    let mut kinds = ['T', 'B', 'V'].map(|k| (k, archived_rows(archive, k)));
    add_moves(&mut kinds, moving)?;
    for (_, rows) in &mut kinds {
        rows.sort_by_key(|r| at_line_start(r).map(|i| i.sort_key()));
    }
    Ok(kinds)
}

/// An archive written from nothing, for an empty file.
fn fresh_archive(kinds: &Kinds) -> String {
    let [(_, tasks), (_, bugs), (_, rules)] = kinds;
    let bug_section = if bugs.is_empty() {
        String::new()
    } else {
        format!("\n\n## §B BUGS\nid|date|cause|fix\n{}", bugs.join("\n"))
    };
    let rule_section = if rules.is_empty() {
        String::new()
    } else {
        format!("\n\n{INVARIANTS_HEADING}\n{}", rules.join("\n"))
    };
    format!(
        "{HEADER}\n{}{bug_section}{rule_section}\n",
        tasks.join("\n")
    )
}

/// The heading of the archive's section for retired `§V` statements.
const INVARIANTS_HEADING: &str = "## §V INVARIANTS";

fn unreadable_archive() -> String {
    format!(
        "{ARCHIVE} has no `## §T` section, so it cannot be read as an \
         archive; refusing to rewrite it"
    )
}

/// The archive with `rows` (one kind, sorted) written over the rows of that
/// kind already there, and any surplus placed after the last of them.
///
/// Every other line is carried through untouched: the header paragraph, a
/// `§B` table nobody asked to change. With no row of the kind to anchor on,
/// the rows go at the end of that kind's section, or in a new one.
fn splice(text: &str, kind: char, rows: &[String]) -> String {
    let mut lines: Vec<String> =
        text.split_inclusive('\n').map(str::to_owned).collect();
    let slots = slots_of(&lines, kind);
    rewrite_slots(&mut lines, &slots, rows);
    match rows.get(slots.len()..) {
        Some(surplus) if !surplus.is_empty() => {
            insert_surplus(lines, kind, &slots, surplus).concat()
        }
        _ => lines.concat(),
    }
}

/// The line indexes already holding a row of `kind`, in file order.
fn slots_of(lines: &[String], kind: char) -> Vec<usize> {
    table_slots(lines.iter().map(String::as_str), kind)
}

/// The rows in the requested table, ignoring prose and fenced examples.
fn table_slots<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    kind: char,
) -> Vec<usize> {
    let mut section = None;
    let mut fence = None;
    lines
        .into_iter()
        .enumerate()
        .filter_map(|(index, line)| {
            table_slot(line, index, kind, (&mut section, &mut fence))
        })
        .collect()
}

#[derive(Clone, Copy)]
struct Fence {
    marker: char,
    length: usize,
}

fn fence_marker(line: &str) -> Option<(char, usize)> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let length = trimmed.chars().take_while(|&c| c == marker).count();
    (length >= 3).then_some((marker, length))
}

fn advance_fence(line: &str, fence: &mut Option<Fence>) -> bool {
    let Some((marker, length)) = fence_marker(line) else {
        return false;
    };
    match *fence {
        Some(open) if marker == open.marker && length >= open.length => {
            *fence = None;
        }
        None => {
            *fence = Some(Fence { marker, length });
        }
        Some(_) => {}
    }
    true
}

fn table_slot(
    line: &str,
    index: usize,
    kind: char,
    state: (&mut Option<char>, &mut Option<Fence>),
) -> Option<usize> {
    let (section, fence) = state;
    if advance_fence(line, fence) {
    } else if fence.is_none() {
        if line.starts_with("## ") {
            *section = ['T', 'B', 'V']
                .into_iter()
                .find(|&candidate| is_section_heading(line, candidate));
        }
        return (*section == Some(kind)
            && at_line_start(line).is_some_and(|id| id.kind == kind))
        .then_some(index);
    }
    None
}

/// Writes each row over its slot, keeping the line ending the old one had.
fn rewrite_slots(lines: &mut [String], slots: &[usize], rows: &[String]) {
    for (&slot, row) in slots.iter().zip(rows) {
        if let Some(line) = lines.get_mut(slot) {
            let body = line.trim_end_matches(['\r', '\n']).len();
            let eol = line.split_off(body);
            *line = format!("{row}{eol}");
        }
    }
}

/// Places the rows that found no slot after the last slot, or at the end of
/// the `kind` section, creating that section when it is missing.
fn insert_surplus(
    mut lines: Vec<String>,
    kind: char,
    slots: &[usize],
    surplus: &[String],
) -> Vec<String> {
    let at = slots.last().map_or_else(
        || section_end(&lines, kind),
        |&last| last.saturating_add(1),
    );
    if let Some(prev) = at.checked_sub(1).and_then(|p| lines.get_mut(p))
        && !prev.ends_with('\n')
    {
        prev.push('\n');
    }
    let mut fresh = new_section(&lines, kind, slots);
    fresh.extend(surplus.iter().map(|r| format!("{r}\n")));
    lines.splice(at..at, fresh);
    lines
}

/// The heading a `kind` section needs when it has no rows and no heading yet.
fn new_section(lines: &[String], kind: char, slots: &[usize]) -> Vec<String> {
    if !slots.is_empty()
        || has_section_heading(lines.iter().map(String::as_str), kind)
    {
        return Vec::new();
    }
    if kind == 'V' {
        return vec![format!("\n{INVARIANTS_HEADING}\n")];
    }
    vec![
        "\n## §B BUGS\n".to_owned(),
        "id|date|cause|fix\n".to_owned(),
    ]
}

/// The index just past the last non-blank line of the `## §<kind>` section,
/// or of the file when there is no such section.
fn section_end(lines: &[String], kind: char) -> usize {
    let Some(start) = section_heading(lines, kind) else {
        return lines.len();
    };
    let first = start.saturating_add(1);
    let stop = next_heading(lines, first);
    lines
        .iter()
        .enumerate()
        .take(stop)
        .skip(first)
        .rev()
        .find(|(_, l)| !l.trim().is_empty())
        .map_or(first, |(i, _)| i.saturating_add(1))
}

/// A section marker ends at whitespace or the end of the heading line.
fn is_section_heading(line: &str, kind: char) -> bool {
    let prefix = format!("## §{kind}");
    line.strip_prefix(&prefix).is_some_and(|rest| {
        rest.chars().next().is_none_or(|c| c.is_whitespace())
    })
}

/// The index of the first `## ` heading at or after `from`, or the file's end.
fn section_heading(lines: &[String], kind: char) -> Option<usize> {
    let mut fence = None;
    lines.iter().enumerate().find_map(|(index, line)| {
        advance_fence(line, &mut fence);
        (fence.is_none() && is_section_heading(line, kind)).then_some(index)
    })
}

fn has_section_heading<'a>(
    lines: impl IntoIterator<Item = &'a str>,
    kind: char,
) -> bool {
    let mut fence = None;
    lines.into_iter().any(|line| {
        advance_fence(line, &mut fence);
        fence.is_none() && is_section_heading(line, kind)
    })
}

fn next_heading(lines: &[String], from: usize) -> usize {
    let mut fence = None;
    lines
        .iter()
        .enumerate()
        .find(|(index, line)| {
            advance_fence(line, &mut fence);
            *index >= from && fence.is_none() && line.starts_with("## ")
        })
        .map_or(lines.len(), |(i, _)| i)
}

fn add_moves(kinds: &mut Kinds, moving: &[Move]) -> Result<(), String> {
    for m in moving {
        let Some((_, rows)) =
            kinds.iter_mut().find(|(k, _)| m.id.starts_with(*k))
        else {
            continue;
        };
        if rows.iter().any(|r| label_of(r) == m.id) {
            return Err(format!("{} is already archived", m.id));
        }
        rows.push(m.row.clone());
    }
    Ok(())
}

fn archived_rows(archive: &str, kind: char) -> Vec<String> {
    let lines: Vec<&str> = archive.lines().collect();
    table_slots(lines.iter().copied(), kind)
        .into_iter()
        .filter_map(|index| lines.get(index).copied())
        .map(str::to_owned)
        .collect()
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
    if m.id.starts_with('B') {
        return true;
    }
    if m.id.starts_with('V') {
        return crate::check::marked(&m.stub) == crate::check::marked(&m.row);
    }
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
pub fn report(spec: &str, records: &[Record], keep: usize) -> String {
    let moving = moves(spec, records, keep);
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
        let (spec, arch) =
            archive(SPEC, "", &records(&[]), 0).unwrap_or_default();
        assert!(spec.contains(&format!("T1|x|{MOVED}|V1,V2")), "{spec}");
        assert!(spec.contains(&format!("T3|x|{MOVED}|-")), "{spec}");
        assert!(
            arch.contains("T1|x|the first thing, at length|V1,V2"),
            "{arch}"
        );
        assert!(arch.contains("T3|x|the third thing|-"), "{arch}");
    }

    /// The compact marker is the normal output, while the old marker remains
    /// an archived row rather than becoming eligible for a second move.
    #[test]
    fn compact_marker_is_written_and_legacy_marker_is_read() {
        let compact = archive(SPEC, "", &records(&[]), 0).unwrap_or_default().0;
        assert!(compact.contains("T1|x|A|V1,V2"), "{compact}");
        let legacy = SPEC.replace(
            "T1|x|the first thing, at length",
            "T1|x|ARCHIVED to SPEC-ARCHIVE.md",
        );
        assert!(
            moves(&legacy, &records(&[]), 0)
                .iter()
                .all(|m| m.id != "T1")
        );
    }

    /// A row that is not FINISHED is not history, so it is left whole.
    #[test]
    fn an_unfinished_row_is_left_alone() {
        let (spec, arch) =
            archive(SPEC, "", &records(&[]), 0).unwrap_or_default();
        assert!(spec.contains("T2|.|not done yet|V3"), "{spec}");
        assert!(!arch.contains("not done yet"), "{arch}");
    }

    #[test]
    fn a_fixed_bug_moves_to_the_bug_section_and_leaves_a_stub() {
        let spec = "## §B BUGS\nid|date|cause|fix\nB2|2026-01-02|a cause|V40\nB3|2026-01-03|open cause|-\n";
        let (folded, arch) =
            archive(spec, "", &records(&[]), 0).unwrap_or_default();
        assert!(folded.contains("B2|2026-01-02|A|A"), "{folded}");
        assert!(folded.contains("B3|2026-01-03|open cause|-"), "{folded}");
        assert!(arch.contains("## §B BUGS\nid|date|cause|fix"), "{arch}");
        assert!(arch.contains("B2|2026-01-02|a cause|V40"), "{arch}");
        assert!(!arch.contains("B3|2026-01-03"), "{arch}");
    }

    #[test]
    fn a_bug_whose_fix_is_the_marker_word_is_still_moved() {
        let spec = "## §B BUGS\nid|date|cause|fix\nB2|2026-01-02|a cause|A\n";
        let (folded, arch) =
            archive(spec, "", &records(&[]), 0).unwrap_or_default();
        assert!(folded.contains("B2|2026-01-02|A|A"), "{folded}");
        assert!(arch.contains("B2|2026-01-02|a cause|A"), "{arch}");
    }

    /// PLANTED (V16): a row carrying a closed-option record never moves.
    ///
    /// The one thing compaction is explicitly forbidden to trade for bytes.
    /// Measured on this repo's own spec: five of 33 finished rows carry one.
    #[test]
    fn a_row_carrying_a_record_is_held_back() {
        let recs = records(&[("T1", "at length")]);
        let (spec, arch) = archive(SPEC, "", &recs, 0).unwrap_or_default();
        assert!(spec.contains("T1|x|the first thing, at length|V1,V2"));
        assert!(!arch.contains("the first thing"), "{arch}");
        assert!(
            report(SPEC, &recs, 0).contains("held back"),
            "said out loud"
        );
    }

    /// Archiving twice is archiving once: the stub is READ as well as
    /// written, so the second run finds nothing to move.
    #[test]
    fn a_second_run_moves_nothing() {
        let (once, arch) =
            archive(SPEC, "", &records(&[]), 0).unwrap_or_default();
        assert!(moves(&once, &records(&[]), 0).is_empty(), "already folded");
        let (twice, again) =
            archive(&once, &arch, &records(&[]), 0).unwrap_or_default();
        assert_eq!(twice, once);
        assert_eq!(again, arch);
    }

    #[test]
    fn a_finished_row_with_trailing_spaces_is_stubbed_and_idempotent() {
        let text = "## §T TASKS\n\nT1|x|done|-   \n";
        let (once, arch) =
            archive(text, "", &records(&[]), 0).unwrap_or_default();
        assert!(once.contains("T1|x|A|-\n"), "{once}");
        assert!(!once.contains("T1|x|done|-"), "{once}");

        let (twice, again) =
            archive(&once, &arch, &records(&[]), 0).unwrap_or_default();
        assert_eq!(twice, once);
        assert_eq!(again, arch);
    }

    /// Rows arriving are not always LATER than the rows already there, so
    /// the archive is merged and re-sorted rather than appended to (V14).
    #[test]
    fn a_later_run_sorts_a_lower_id_into_place() {
        let first = "T9|x|ninth|-\n";
        let later = "# SPEC\n\n## \u{a7}T TASKS\n\nT2|x|second|-\n";
        let (_, arch) = archive(later, &format!("{HEADER}\n{first}"), &[], 0)
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
        let (_, arch) = archive(SPEC, "", &records(&[]), 0).unwrap_or_default();
        assert!(arch.starts_with("# SPEC ARCHIVE"), "{arch}");
        assert!(arch.contains("## \u{a7}T TASKS"), "{arch}");
        let listed = crate::tasks::json("a", &arch);
        assert!(listed.contains("\"T1\""), "{listed}");
        assert!(listed.contains("\"T3\""), "{listed}");
    }

    const HAND_ARCHIVE: &str = "\
# SPEC ARCHIVE

My own words about this file.

## \u{a7}T TASKS

T2|x|older|-

## \u{a7}B BUGS

id|date|cause|fix
B1|2026-01-01|x|y
B2|2026-01-02|x|y

## Notes

kept
";

    /// Issue #109: a hand-kept `§B` table, header text and trailing section
    /// survive; only the `§T` table gains its rows.
    #[test]
    fn archiving_leaves_every_other_byte_of_the_archive() {
        let (_, arch) =
            archive(SPEC, HAND_ARCHIVE, &records(&[]), 0).unwrap_or_default();
        let want = HAND_ARCHIVE.replace(
            "T2|x|older|-\n",
            "T1|x|the first thing, at length|V1,V2\nT2|x|older|-\n\
             T3|x|the third thing|-\n",
        );
        assert_eq!(arch, want);
    }

    #[test]
    fn row_like_text_in_notes_and_fences_is_not_an_archive_slot() {
        let held =
            "# A\n\n## §T TASKS\n\n## Notes\n\n```\nT9|x|example|-\n```\n";
        let (_, arch) =
            archive(SPEC, held, &records(&[]), 0).unwrap_or_default();
        assert!(
            arch.contains(
                "## §T TASKS\nT1|x|the first thing, at length|V1,V2\n"
            )
        );
        assert!(arch.contains("```\nT9|x|example|-\n```"));
    }

    /// Companion (V18): an archive that is not one is refused, by name.
    #[test]
    fn an_archive_without_a_task_section_is_refused() {
        let err = archive(SPEC, "# notes\nB1|d|c|f\n", &records(&[]), 0).err();
        assert!(err.is_some_and(|e| e.contains(ARCHIVE)));
    }

    /// A heading that only shares the task prefix is not a task section.
    #[test]
    fn an_archive_with_a_task_prefix_heading_is_refused() {
        let err =
            archive(SPEC, "# notes\n\n## §TODO\nB1|d|c|f\n", &records(&[]), 0)
                .err();
        assert!(err.is_some_and(|e| e.contains(ARCHIVE)));
    }

    /// Rows arriving where the archive has none yet land in its `§T` section.
    #[test]
    fn a_task_section_with_no_rows_receives_them_in_place() {
        let held = "# A\n\n## \u{a7}T TASKS\n\n## \u{a7}B BUGS\nB1|d|c|f\n";
        let (_, arch) =
            archive(SPEC, held, &records(&[]), 0).unwrap_or_default();
        assert_eq!(
            arch,
            "# A\n\n## \u{a7}T TASKS\nT1|x|the first thing, at length|V1,V2\n\
             T3|x|the third thing|-\n\n## \u{a7}B BUGS\nB1|d|c|f\n"
        );
    }

    #[test]
    fn a_heading_inside_a_fenced_empty_task_section_does_not_bound_insertion() {
        let held = "# A\n\n## §T TASKS\n\n```\n## Notes\n```\n";
        let (_, arch) =
            archive(SPEC, held, &records(&[]), 0).unwrap_or_default();
        assert!(arch.contains(
            "```\n## Notes\n```\nT1|x|the first thing, at length|V1,V2\n"
        ));
        assert!(arch.contains("T3|x|the third thing|-\n"));
    }

    /// PLANTED (V18): the proof refuses a move that would lose a row.
    ///
    /// The archive already holds this id, so folding it in would leave two
    /// rows with one id -- V12's defect, written by the tool that checks it.
    #[test]
    fn re_archiving_one_id_is_refused_rather_than_duplicated() {
        let held = format!("{HEADER}\nT1|x|the first thing, at length|V1,V2\n");
        let err = archive(SPEC, &held, &records(&[]), 0).err();
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
        assert_eq!(moves(text, &records(&[]), 0), vec![]);
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
        let (spec, _) = archive(text, "", &records(&[]), 0).unwrap_or_default();
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
        assert_eq!(moves(text, &records(&[]), 0), vec![]);
        let (spec, arch) =
            archive(text, "", &records(&[]), 0).unwrap_or_default();
        assert_eq!(spec, text);
        assert!(report(text, &records(&[]), 0).contains("nothing to archive"));
        assert!(!arch.contains("T1"), "{arch}");
    }

    const RULES: &str = "\
## \u{a7}V INVARIANTS

V1: live rule.
V3: old rule with words. [superseded by V9]
V4: quoted `[superseded by V9]` only.
V5: was superseded by V9 [B1].
V9: the replacement.
";

    /// The text moves; the id and the mark stay, so V41 still checks the chain.
    #[test]
    fn a_superseded_statement_leaves_a_stub_with_id_and_mark() {
        let (spec, arch) =
            archive(RULES, "", &records(&[]), 0).unwrap_or_default();
        assert!(spec.contains("\nV3: A [superseded by V9]\n"), "{spec}");
        assert!(!spec.contains("old rule with words"), "{spec}");
        assert!(arch.contains("## \u{a7}V INVARIANTS\n"), "{arch}");
        assert!(
            arch.contains("V3: old rule with words. [superseded by V9]"),
            "{arch}"
        );
        assert!(
            crate::check::supersessions_resolve(&spec).is_empty(),
            "{spec}"
        );
        assert!(crate::check::citations_resolve(&spec).is_empty(), "{spec}");
    }

    #[test]
    fn a_wrapped_superseded_statement_moves_all_continuation_lines() {
        let text = "## §V INVARIANTS\n\nV3: old rule with words. [superseded by V9]\n\
continued evidence for the old rule.\nV9: the replacement.\n";
        let (spec, arch) =
            archive(text, "", &records(&[]), 0).unwrap_or_default();
        assert!(spec.contains("V3: A [superseded by V9]\n"), "{spec}");
        assert!(!spec.contains("continued evidence"), "{spec}");
        assert!(
            arch.contains("V3: old rule with words. [superseded by V9]\ncontinued evidence for the old rule."),
            "{arch}"
        );
    }

    #[test]
    fn a_fence_between_a_rule_and_prose_is_not_consumed_as_a_continuation() {
        let text = "## §V INVARIANTS\n\nV3: old rule. [superseded by V9]\n```\ncontinued example\n```\ncontinued evidence\nV9: the replacement.\n";
        let (spec, arch) =
            archive(text, "", &records(&[]), 0).unwrap_or_default();
        assert!(spec.contains("V3: A [superseded by V9]\n```\ncontinued example\n```\ncontinued evidence\n"), "{spec}");
        assert!(
            arch.contains("V3: old rule. [superseded by V9]\n"),
            "{arch}"
        );
        assert!(!arch.contains("continued evidence"), "{arch}");
    }

    /// Companion (V18): live rules, and marks only in backticks or prose, stay.
    #[test]
    fn live_and_prose_marked_statements_are_never_moved() {
        let (spec, arch) =
            archive(RULES, "", &records(&[]), 0).unwrap_or_default();
        for keep in ["V1: live rule.", "V4: quoted `[superseded by V9]` only."]
        {
            assert!(spec.contains(keep), "{spec}");
        }
        assert!(spec.contains("V5: was superseded by V9 [B1]."), "{spec}");
        assert!(spec.contains("V9: the replacement."), "{spec}");
        assert!(!arch.contains("V1:") && !arch.contains("V4:"), "{arch}");
        assert!(!arch.contains("V5:") && !arch.contains("V9:"), "{arch}");
    }

    /// PLANTED (V16): a superseded statement carrying a record stays whole,
    /// and is named, exactly as a row carrying one is.
    #[test]
    fn a_superseded_statement_carrying_a_record_is_held_back() {
        let recs = records(&[("V3", "with words")]);
        let (spec, arch) = archive(RULES, "", &recs, 0).unwrap_or_default();
        assert!(spec.contains("V3: old rule with words."), "{spec}");
        assert!(!arch.contains("V3:"), "{arch}");
        assert!(
            held_back(RULES, &recs).contains("V3 carry"),
            "said out loud"
        );
    }

    #[test]
    fn a_second_run_over_stubbed_rules_moves_nothing() {
        let (once, arch) =
            archive(RULES, "", &records(&[]), 0).unwrap_or_default();
        assert!(moves(&once, &records(&[]), 0).is_empty());
        let (twice, again) =
            archive(&once, &arch, &records(&[]), 0).unwrap_or_default();
        assert_eq!((twice, again), (once, arch));
    }

    /// Existing archive content survives and the `§V` section is added.
    #[test]
    fn rules_join_an_existing_archive_without_disturbing_it() {
        let (_, arch) =
            archive(RULES, HAND_ARCHIVE, &records(&[]), 0).unwrap_or_default();
        assert!(arch.starts_with(HAND_ARCHIVE.trim_end_matches("kept\n")));
        assert!(arch.contains("\n## \u{a7}V INVARIANTS\n"), "{arch}");
        assert!(arch.contains("T2|x|older|-\n"), "{arch}");
        assert!(arch.ends_with("[superseded by V9]\n"), "{arch}");
    }

    /// Archived rules keep V14 order across runs.
    #[test]
    fn a_later_rule_sorts_into_an_existing_invariants_section() {
        let held = format!(
            "{HEADER}\n## \u{a7}V INVARIANTS\nV7: seven. [superseded by V9]\n"
        );
        let (_, arch) =
            archive(RULES, &held, &records(&[]), 0).unwrap_or_default();
        let ids: Vec<String> = arch
            .lines()
            .filter_map(at_line_start)
            .map(|i| i.label())
            .collect();
        assert_eq!(ids, vec!["V3", "V7"], "{arch}");
    }
}
