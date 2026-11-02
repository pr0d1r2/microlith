//! The two verbs that WRITE a row: `add` appends one, `amend` replaces one's
//! text.
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

/// One line with its text swapped. What "text" is follows the section: the
/// statement of a `V`, the task of a `T`, the cause of a `B`.
fn swapped(line: &str, kind: char, new: &str) -> Result<String, String> {
    if kind == 'V' {
        let (id, _) = line.split_once(':').ok_or("malformed row")?;
        return Ok(format!("{id}: {new}"));
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
}
