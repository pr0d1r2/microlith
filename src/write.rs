//! The verbs that WRITE a row: `add` appends one, `amend` replaces one's
//! text, `supersede` marks rows retired.
//!
//! Both are pure functions over `&str`, like every other rule here (V7): the
//! caller supplies the spec text and gets the new text back, so the same
//! code serves the CLI and a consumer. And both hold the result to what
//! `check` and `fmt` hold the file to BEFORE returning it -- a row that
//! would introduce a violation is refused with the violation named, so the
//! file is never written into a state `check` would then reject (V1).
//!
//! The id, the date and the escape are the three things a hand edit gets
//! wrong, so they are the three things done here: the next free id, today's
//! date for `§B`, and `\|` for a pipe inside a cell.

use crate::check::{declared, is_header_for};
use crate::id::{at_line_start, cells, escape};

type Written = Result<(String, String), String>;

/// The section letter, from `V` or `§T`; only these three take rows.
fn kind_of(section: &str) -> Result<char, String> {
    let mut chars = section.trim_start_matches('\u{a7}').chars();
    match (chars.next(), chars.next()) {
        (Some(k @ ('V' | 'T' | 'B')), None) => Ok(k),
        _ => Err(format!(
            "'{section}' is not a section that takes rows -- expected V, T or B"
        )),
    }
}

/// How many cells `add` takes, and what they are.
fn shape(kind: char) -> (usize, &'static str) {
    match kind {
        'V' => (1, "<statement>"),
        'T' => (2, "<task> <cites|->"),
        _ => (2, "<cause> <fix>"),
    }
}

/// A cell is one line of visible text: a newline would be a hard wrap, and
/// the formatter would then join it or the checker misread it.
fn cell_ok(cell: &str) -> Result<String, String> {
    let cell = cell.trim();
    if cell.is_empty() {
        return Err("an empty cell -- use `-` for \"none\"".to_owned());
    }
    if cell.chars().any(char::is_control) {
        return Err(format!(
            "'{cell}' holds a newline or control character -- one row is \
             one line"
        ));
    }
    Ok(cell.to_owned())
}

/// The cells, counted against the section's shape and each checked.
fn cells_for(kind: char, given: &[String]) -> Result<Vec<String>, String> {
    let (want, usage) = shape(kind);
    if given.len() != want {
        return Err(format!("\u{a7}{kind} takes {usage}"));
    }
    given.iter().map(|c| cell_ok(c)).collect()
}

/// The row as written. The statement of a `V` is prose and goes verbatim;
/// the cells of a pipe row are escaped.
fn new_row(kind: char, next: u32, today: &str, cells: &[String]) -> String {
    let esc: Vec<String> = cells.iter().map(|c| escape(c)).collect();
    match kind {
        'V' => format!("V{next}: {}", cells.join(" ")),
        'T' => format!("T{next}|.|{}", esc.join("|")),
        _ => format!("B{next}|{today}|{}", esc.join("|")),
    }
}

/// Append a row to section `section`; returns the new text and the row.
///
/// `today` is supplied rather than read, so the function stays pure and a
/// test can pin it.
///
/// # Errors
/// Unknown section, wrong cell count, a section absent from the file, or a
/// result that `check` or `fmt` would reject.
pub fn add_row(
    text: &str,
    section: &str,
    given: &[String],
    today: &str,
) -> Written {
    let kind = kind_of(section)?;
    let cells = cells_for(kind, given)?;
    let ids = declared(text, kind).into_iter().map(|i| i.num);
    let next = ids.max().map_or(1, |n| n.saturating_add(1));
    let row = new_row(kind, next, today, &cells);
    let out = insert(text, kind, &row)?;
    validated(&out)?;
    Ok((out, row))
}

/// The byte offset of the last `|` that is neither escaped nor inside
/// backticks.
fn pipe_at(s: &str) -> Option<usize> {
    let (mut found, mut ticks, mut slashes) = (None, 0_usize, 0_usize);
    for (i, c) in s.char_indices() {
        if c == '|' && slashes % 2 == 0 && ticks % 2 == 0 {
            found = Some(i);
        }
        ticks = ticks.saturating_add(usize::from(c == '`'));
        slashes = if c == '\\' {
            slashes.saturating_add(1)
        } else {
            0
        };
    }
    found
}

/// Whether `s` is a comma-separated list of citation ids, such as `V7,V14`.
fn cites_only(s: &str) -> bool {
    !s.is_empty()
        && s.split(',').all(|item| {
            let item = item.trim();
            at_line_start(&format!("{item}:"))
                .is_some_and(|i| i.label() == item)
        })
}

/// A `V` row with its statement swapped, its `|V7,V14` citation tail kept.
fn swapped_statement(line: &str, new: &str) -> Result<String, String> {
    let (id, rest) = line.split_once(':').ok_or("malformed row")?;
    if pipe_at(new).is_some() {
        return Err("an unescaped `|` would start a citation field".into());
    }
    let tail = pipe_at(rest)
        .map(|at| rest.split_at(at).1)
        .filter(|t| cites_only(t.trim_start_matches('|')))
        .unwrap_or_default();
    Ok(format!("{id}: {new}{tail}"))
}

/// One line with its text swapped. What "text" is follows the section: the
/// statement of a `V`, the task of a `T`, the cause of a `B`.
fn swapped(line: &str, kind: char, new: &str) -> Result<String, String> {
    if kind == 'V' {
        return swapped_statement(line, new);
    }
    let mut parts: Vec<String> =
        cells(line).into_iter().map(str::to_owned).collect();
    if parts.len() < 4 {
        return Err("not a four-field row".to_owned());
    }
    if let Some(cell) = parts.get_mut(2) {
        *cell = escape(new);
    }
    Ok(parts.join("|"))
}

/// Replace the text of row `id`, keeping its id and its citations; returns
/// the new text and the row as written. The status, date and citations stay.
///
/// # Errors
/// No such row, a malformed one, or a result `check` or `fmt` would reject.
pub fn amend_row(text: &str, id: &str, new: &str) -> Written {
    let kind = kind_of(id.get(..1).unwrap_or(""))?;
    let new = cell_ok(new)?;
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let at = lines
        .iter()
        .position(|l| at_line_start(l).is_some_and(|i| i.label() == id))
        .ok_or_else(|| format!("no row {id} here"))?;
    let line = lines.get_mut(at).ok_or("no such line")?;
    let row = swapped(line, kind, &new).map_err(|e| format!("{id}: {e}"))?;
    line.clone_from(&row);
    let out = rejoin(text, &lines);
    validated(&out)?;
    Ok((out, row))
}

/// Whether `s` is exactly one row id, such as `V7` or `T7a`.
pub(crate) fn is_id(s: &str) -> bool {
    at_line_start(&format!("{s}:")).is_some_and(|i| i.label() == s)
}

/// The ids a `[<word> A, B]` mark on `line` already names.
fn named_by(line: &str, word: &str) -> Vec<String> {
    if word == crate::check::SUPERSEDED_BY {
        return crate::check::marked(line);
    }
    line.split_once(format!("[{word} ").as_str())
        .and_then(|(_, rest)| rest.split_once(']'))
        .map(|(inside, _)| inside.split(", ").map(str::to_owned).collect())
        .unwrap_or_default()
}

/// `fresh` joined into the existing `[<word> ...]` mark, before its `]`.
fn merged(line: &str, word: &str, fresh: &[&str]) -> Option<String> {
    let open = format!("[{word} ");
    let at = line.find(open.as_str())?.checked_add(open.len())?;
    let close = line.get(at..)?.find(']')?.checked_add(at)?;
    let (head, tail) = line.split_at(close);
    Some(format!("{head}, {}{tail}", fresh.join(", ")))
}

/// A `V` row with `mark` after its statement, before any `|V7` tail.
fn marked_statement(line: &str, mark: &str) -> Option<String> {
    let (prefix, id, rest) = if let Some((id, rest)) = line.split_once(':') {
        ("".to_owned(), id.to_owned(), format!(":{rest}"))
    } else {
        let (prefix, body) = ["- ", "* ", "+ "].iter().find_map(|prefix| {
            line.strip_prefix(prefix).map(|body| (*prefix, body))
        })?;
        let id = at_line_start(line)?.label();
        let rest = body.get(id.len()..)?;
        (prefix.to_owned(), id, rest.to_owned())
    };
    let tail_at = pipe_at(&rest).filter(|&a| {
        cites_only(rest.get(a..).unwrap_or_default().trim_start_matches('|'))
    });
    let (body, tail) = rest.split_at(tail_at.unwrap_or(rest.len()));
    Some(format!("{prefix}{id}{} {mark}{tail}", body.trim_end()))
}

/// A pipe row with `mark` at the end of its text cell.
fn marked_cell(line: &str, mark: &str) -> Option<String> {
    let mut parts: Vec<String> =
        cells(line).into_iter().map(str::to_owned).collect();
    let cell = parts.get_mut(2)?;
    cell.push(' ');
    cell.push_str(mark);
    Some(parts.join("|"))
}

fn fresh_ids<'a>(named: &[String], ids: &'a [&str]) -> Vec<&'a str> {
    let mut fresh = Vec::new();
    for id in ids.iter().copied() {
        if !named.iter().any(|n| n == id) && !fresh.contains(&id) {
            fresh.push(id);
        }
    }
    fresh
}

/// The `[<word> A, B]` mark put on a row, or merged into the one it has.
///
/// `None` when every id is already named: re-running is a no-op.
fn with_mark(
    line: &str,
    kind: char,
    word: &str,
    ids: &[&str],
) -> Result<Option<String>, String> {
    let named = named_by(line, word);
    let fresh = fresh_ids(&named, ids);
    let mark = format!("[{word} {}]", fresh.join(", "));
    match (fresh.is_empty(), named.is_empty(), kind) {
        (true, _, _) => Ok(None),
        (false, false, _) => merged(line, word, &fresh)
            .map(Some)
            .ok_or_else(|| format!("cannot place mark on row: {line}")),
        (false, true, 'V') => marked_statement(line, &mark)
            .map(Some)
            .ok_or_else(|| format!("cannot place mark on row: {line}")),
        (false, true, _) => marked_cell(line, &mark)
            .map(Some)
            .ok_or_else(|| format!("cannot place mark on row: {line}")),
    }
}

/// The index of row `id`, or the refusal for an id the spec does not hold.
fn row_of(lines: &[String], id: &str) -> Result<usize, String> {
    lines
        .iter()
        .position(|l| at_line_start(l).is_some_and(|i| i.label() == id))
        .ok_or_else(|| format!("unknown id: {id} (not in this spec)"))
}

/// Mark row `id` with `[<word> ids]`; returns the row as it now stands.
fn mark_row(
    lines: &mut [String],
    id: &str,
    word: &str,
    ids: &[&str],
) -> Result<String, String> {
    let at = row_of(lines, id)?;
    let line = lines.get_mut(at).ok_or("no such line")?;
    let kind = kind_of(id.get(..1).unwrap_or(""))?;
    if let Some(row) = with_mark(line, kind, word, ids)? {
        *line = row;
    }
    Ok(line.clone())
}

/// The refusals that need only the ids, before any row is read.
fn supersede_shape(winner: &str, losers: &[String]) -> Result<(), String> {
    let kind = kind_of(winner.get(..1).unwrap_or(""))?;
    if losers.is_empty() {
        return Err("supersede needs at least one loser".to_owned());
    }
    if losers.iter().any(|l| l == winner) {
        return Err(format!("{winner} cannot supersede itself"));
    }
    match losers
        .iter()
        .find(|l| kind_of(l.get(..1).unwrap_or("")) != Ok(kind))
    {
        Some(odd) => Err(format!("{odd} is not the same kind as {winner}")),
        None => Ok(()),
    }
}

/// Retire each of `losers` in favour of `winner`: a `[superseded by W]` mark
/// on every loser, and with `link` a `[supersedes L, ...]` mark on the
/// winner. Returns the new text and the rows as written, one per line.
///
/// # Errors
/// An id with no row; the winner among the losers; ids of different kinds; a
/// winner that is itself superseded (V41 would reject the result); or a
/// result `check` or `fmt` would reject. Marking what is already marked is a
/// no-op, not an error.
pub fn supersede_rows(
    text: &str,
    winner: &str,
    losers: &[String],
    link: bool,
) -> Written {
    supersede_shape(winner, losers)?;
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let at = row_of(&lines, winner)?;
    if lines
        .get(at)
        .is_some_and(|l| !crate::check::marked(l).is_empty())
    {
        return Err(format!("{winner} is itself superseded -- use its winner"));
    }
    let rows = retire(&mut lines, winner, losers, link)?;
    let out = rejoin(text, &lines);
    validated(&out)?;
    Ok((out, rows.join("\n")))
}

/// The marks themselves; returns every row touched, in the order marked.
fn retire(
    lines: &mut [String],
    winner: &str,
    losers: &[String],
    link: bool,
) -> Result<Vec<String>, String> {
    let by = crate::check::SUPERSEDED_BY;
    let mut rows = Vec::new();
    let mut seen = Vec::new();
    for loser in losers {
        if seen.contains(&loser) {
            continue;
        }
        seen.push(loser);
        rows.push(mark_row(lines, loser, by, &[winner])?);
    }
    if link {
        let ids: Vec<&str> = losers.iter().map(String::as_str).collect();
        rows.push(mark_row(lines, winner, "supersedes", &ids)?);
    }
    Ok(rows)
}

/// The lines back to text, with the original's final newline (or lack).
fn rejoin(original: &str, lines: &[String]) -> String {
    let mut out = lines.join("\n");
    if original.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// The line after which a new row of `kind` goes: the last non-blank line
/// of the section, which is the header itself when the section is empty.
fn last_line(lines: &[String], kind: char) -> Option<usize> {
    let head = lines.iter().position(|l| is_header_for(l, kind))?;
    let body = lines.iter().enumerate().skip(head.saturating_add(1));
    let mut last = head;
    for (i, line) in body {
        if line.starts_with("## ") {
            break;
        }
        if !line.trim().is_empty() {
            last = i;
        }
    }
    Some(last)
}

/// Put `row` after the last line of its section.
fn insert(text: &str, kind: char, row: &str) -> Result<String, String> {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let last = last_line(&lines, kind)
        .ok_or_else(|| format!("no \u{a7}{kind} section here to add to"))?;
    let heading = lines.get(last).is_some_and(|l| l.starts_with("## "));
    let at = last.saturating_add(1);
    lines.insert(at, row.to_owned());
    if heading {
        // An empty section: keep the blank line a header is followed by.
        lines.insert(at, String::new());
    }
    Ok(rejoin(text, &lines))
}

/// Hold the result to `check` and `fmt`: no violation at all, as `check`
/// would report it, and already in the form `fmt` would leave it.
fn validated(after: &str) -> Result<(), String> {
    let found: Vec<String> = crate::check_spec(after, &[])
        .into_iter()
        .map(|v| format!("{}: {}", v.rule, v.msg))
        .collect();
    if !found.is_empty() {
        return Err(format!(
            "refusing to write -- `check` would report: {}",
            found.join("; ")
        ));
    }
    if crate::format_spec(after)? != after {
        return Err("refusing to write: `fmt` would change the result".into());
    }
    Ok(())
}

/// Today as `YYYY-MM-DD` (UTC), from the clock. The one impure function
/// here, kept apart so everything else can be pinned in a test.
pub(crate) fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil(secs / 86_400)
}

fn leap(year: u64) -> bool {
    year.is_multiple_of(4)
        && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

fn month_len(year: u64, month: u64) -> u64 {
    match month {
        2 if leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 as `YYYY-MM-DD`. Walked year by year and month by
/// month rather than by a closed formula: it is short, and a day count is
/// small enough that no one will wait for it.
fn civil(days: u64) -> String {
    let (mut left, mut year, mut month) = (days, 1970_u64, 1_u64);
    loop {
        let len = if leap(year) { 366 } else { 365 };
        if left < len {
            break;
        }
        left = left.saturating_sub(len);
        year = year.saturating_add(1);
    }
    while left >= month_len(year, month) {
        left = left.saturating_sub(month_len(year, month));
        month = month.saturating_add(1);
    }
    format!("{year:04}-{month:02}-{:02}", left.saturating_add(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "## \u{a7}V INVARIANTS\n\nV1: **a.** x.\nV2: **b.** y.\n\n\
        ## \u{a7}T TASKS\n\nT1|.|do it|V1\n\n\
        ## \u{a7}B BUGS\n\nB1|2026-01-01|c|V1\n";

    fn s(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| (*p).to_owned()).collect()
    }

    fn added(section: &str, cells: &[&str]) -> (String, String) {
        add_row(SPEC, section, &s(cells), "2026-11-02").unwrap_or_default()
    }

    fn refused(section: &str, cells: &[&str]) -> String {
        add_row(SPEC, section, &s(cells), "d")
            .err()
            .unwrap_or_default()
    }

    #[test]
    fn civil_dates_are_right() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(789), "1972-02-29");
        assert_eq!(civil(790), "1972-03-01");
        assert_eq!(civil(20_394), "2025-11-02");
    }

    #[test]
    fn add_assigns_the_next_id_in_each_section() {
        let (t, r) = added("V", &["**c.** z."]);
        assert_eq!(r, "V3: **c.** z.");
        assert!(t.contains("V3: **c.** z.\n\n## \u{a7}T"), "{t}");
        let (t, r) = added("\u{a7}T", &["more", "V1,V2"]);
        assert_eq!(r, "T2|.|more|V1,V2");
        assert!(t.contains("T2|.|more|V1,V2\n\n## \u{a7}B"), "{t}");
    }

    #[test]
    fn add_dates_a_bug_and_escapes_its_pipes() {
        let (t, r) = added("B", &["a|b", "V2"]);
        assert_eq!(r, "B2|2026-11-02|a\\|b|V2");
        assert!(t.ends_with("B2|2026-11-02|a\\|b|V2\n"), "{t}");
    }

    /// V18's rejecting half: every refusal leaves nothing to write.
    #[test]
    fn add_refuses_what_check_would_reject() {
        assert!(refused("T", &["x", "V99"]).contains("V13"));
        assert!(refused("T", &["x"]).contains("takes"));
        assert!(refused("T", &["a\nb", "-"]).contains("one line"));
        assert!(refused("X", &["x"]).contains("expected V, T or B"));
        let bare = add_row("V1: a\n", "T", &s(&["x", "-"]), "d");
        assert!(bare.is_err());
    }

    #[test]
    fn amend_keeps_the_id_and_the_citations() {
        let amend = |id, new| amend_row(SPEC, id, new).unwrap_or_default().1;
        assert_eq!(amend("T1", "changed"), "T1|.|changed|V1");
        assert_eq!(amend("B1", "new"), "B1|2026-01-01|new|V1");
        assert_eq!(amend("V2", "**b2.** y."), "V2: **b2.** y.");
    }

    #[test]
    fn amend_keeps_a_v_row_citation_tail_and_refuses_a_bare_pipe() {
        let spec = SPEC.replace("V2: **b.** y.", "V2: **b.** y.|V1");
        let (_, row) = amend_row(&spec, "V2", "**b2.** z.").unwrap_or_default();
        assert_eq!(row, "V2: **b2.** z.|V1");
        let err = amend_row(&spec, "V2", "a|b").err().unwrap_or_default();
        assert!(err.contains("unescaped"), "{err}");
    }

    #[test]
    fn amend_refuses_a_missing_row_and_a_dangling_citation() {
        let err = |id, new| amend_row(SPEC, id, new).err().unwrap_or_default();
        assert!(err("T9", "x").contains("no row"));
        assert!(err("V1", "**a.** see V77").contains("V13"));
    }

    #[test]
    fn amend_refuses_a_preexisting_violation() {
        let invalid = SPEC.replace("T1|.|do it|V1", "T1|.|do it|V99");
        assert_ne!(invalid, SPEC);
        assert!(amend_row(&invalid, "T1", "changed").is_err());
        assert!(amend_row(&invalid, "V1", "**a.** x2.").is_err());
    }

    const SUP: &str = "## \u{a7}V INVARIANTS\n\nV1: **a.** x.\nV2: **b.** y.|V1\n\
        V3: **c.** z.\n\n## \u{a7}T TASKS\n\nT1|.|do it|V1\nT2|.|more|V1\n";

    fn sup(w: &str, l: &[&str], link: bool) -> Written {
        supersede_rows(SUP, w, &s(l), link)
    }

    #[test]
    fn supersede_marks_losers_and_keeps_a_citation_tail() {
        let (t, rows) = sup("V1", &["V2", "V3"], false).unwrap_or_default();
        assert!(t.contains("V2: **b.** y. [superseded by V1]|V1\n"), "{t}");
        assert!(t.contains("V3: **c.** z. [superseded by V1]\n"), "{t}");
        assert!(!rows.contains("supersedes"), "{rows}");
        assert!(crate::check_spec(&t, &[]).is_empty());
    }

    #[test]
    fn supersede_link_tags_the_winner_and_task_rows_work() {
        let (t, _) = sup("V3", &["V1"], true).unwrap_or_default();
        assert!(t.contains("V3: **c.** z. [supersedes V1]\n"), "{t}");
        let (t, _) = sup("T1", &["T2"], false).unwrap_or_default();
        assert!(t.contains("T2|.|more [superseded by T1]|V1\n"), "{t}");
    }

    /// Re-running changes nothing; a second winner is merged into the mark.
    #[test]
    fn supersede_is_idempotent_and_merges_winners() {
        let (once, _) = sup("V1", &["V3"], true).unwrap_or_default();
        let again = supersede_rows(&once, "V1", &s(&["V3"]), true);
        assert_eq!(again.map(|a| a.0), Ok(once.clone()));
        let (t, _) =
            supersede_rows(&once, "V2", &s(&["V3"]), false).unwrap_or_default();
        assert!(t.contains("[superseded by V1, V2]"), "{t}");
    }

    #[test]
    fn supersede_deduplicates_repeated_losers() {
        let (t, rows) = sup("V1", &["V2", "V2"], true).unwrap_or_default();
        assert!(t.contains("V1: **a.** x. [supersedes V2]\n"), "{t}");
        assert!(!t.contains("supersedes V2, V2"), "{t}");
        assert_eq!(rows.matches("superseded by V1").count(), 1);
    }

    /// V18's rejecting half: each refusal names its reason.
    #[test]
    fn supersede_refuses_what_it_must() {
        let err = |w, l: &[&str]| sup(w, l, false).err().unwrap_or_default();
        assert!(err("V1", &["V9"]).contains("unknown id: V9"));
        assert!(err("V9", &["V1"]).contains("unknown id: V9"));
        assert!(err("V1", &["V1"]).contains("itself"));
        assert!(err("V1", &["V2", "V1"]).contains("itself"));
        assert!(err("V1", &["T1"]).contains("same kind"));
        assert!(err("V1", &[]).contains("at least one"));
    }

    #[test]
    fn supersede_refuses_a_multibyte_loser() {
        let err = sup("V1", &["é"], false).err().unwrap_or_default();
        assert!(err.contains("same kind"));
    }

    #[test]
    fn supersede_refuses_a_winner_that_is_itself_superseded() {
        let (t, _) = sup("V1", &["V3"], false).unwrap_or_default();
        let dead = supersede_rows(&t, "V3", &s(&["V2"]), false);
        assert!(dead.err().unwrap_or_default().contains("itself superseded"));
    }

    #[test]
    fn supersede_marks_a_bulleted_declaration_without_a_colon() {
        let text =
            "## \u{a7}V INVARIANTS\n\n- V1 first rule\n- V2 second rule\n";
        let result = supersede_rows(text, "V1", &s(&["V2"]), false);
        assert!(result.is_ok(), "{result:?}");
        let (out, _) = result.unwrap_or_default();
        assert!(out.contains("- V1 first rule"));
        assert!(out.contains("- V2 second rule [superseded by V1]"));
    }
}
