//! `mth add` and `mth amend`: write ONE row, through the rules that read it.
//!
//! A hand edit of a row breaks the format in ways that only show up later --
//! a reused id, a hard wrap, a stray pipe -- and shows up in `check` or `fmt`
//! rather than at the edit. So the edit is made here, as a pure function of
//! the text, and PROVEN before it is returned: the result must introduce no
//! violation `check` did not already report and must stay `fmt`-stable. A
//! caller writes the file only on `Ok`, so a refusal changes nothing.
//!
//! The grammar is not re-read here (V7): ids come from `check::declared`,
//! cells are split and written by `id::cells` / `id::escape`, and the verdict
//! is `check_spec` and `format_spec` themselves.

use crate::check::{declared, is_header_for};
use crate::id::{at_line_start, cells, escape};
use crate::{check_spec, format_spec};

/// The section letters a row can be written to.
const WRITABLE: [char; 3] = ['V', 'B', 'T'];

/// The new text, and the one row that was written.
pub(crate) type Edited = (String, String);

/// A section letter from `V`, `B`, `T` or `\u{a7}V`.
pub(crate) fn kind_of(section: &str) -> Result<char, String> {
    let bare = section.strip_prefix('\u{a7}').unwrap_or(section);
    let mut chars = bare.chars();
    match (chars.next(), chars.next()) {
        (Some(k), None) if WRITABLE.contains(&k) => Ok(k),
        _ => Err(format!("unknown section '{section}' -- expected V, B or T")),
    }
}

/// How many cells a row of this kind is given.
pub(crate) const fn arity(kind: char) -> usize {
    match kind {
        'B' | 'T' => 2,
        _ => 1,
    }
}

/// What the cells are called, for the refusal that counts them.
const fn names(kind: char) -> &'static str {
    match kind {
        'V' => "<text>",
        'B' => "<cause> <fix>",
        _ => "<task> <cites>",
    }
}

/// A cell that cannot sit on one row: a newline would hard-wrap it (V3).
fn one_line(cell: &str) -> Result<(), String> {
    if cell.chars().any(char::is_control) {
        return Err("a cell must be one line of text -- no newline or \
                    control character"
            .to_owned());
    }
    Ok(())
}

/// The number after the highest id ever declared, so a gap is never refilled.
fn next_id(text: &str, kind: char) -> u32 {
    declared(text, kind)
        .iter()
        .map(|i| i.num)
        .max()
        .map_or(1, |n| n.saturating_add(1))
}

/// The row `add` would write, with the id it was assigned.
fn new_row(
    text: &str,
    kind: char,
    given: &[String],
    today: &str,
) -> Result<String, String> {
    if given.len() != arity(kind) {
        return Err(format!(
            "\u{a7}{kind} takes {} -- got {} cell(s)",
            names(kind),
            given.len()
        ));
    }
    given.iter().try_for_each(|c| one_line(c))?;
    let next = next_id(text, kind);
    let cell = |n: usize| escape(given.get(n).map_or("", |s| s.trim()));
    Ok(match kind {
        'V' => format!("V{next}: {}", cell(0)),
        'B' => format!("B{next}|{today}|{}|{}", cell(0), cell(1)),
        _ => format!("T{next}|.|{}|{}", cell(0), cell(1)),
    })
}

/// The index of the line ending `kind`'s section: the next `## ` header, or
/// the end of the text.
fn section_end(lines: &[&str], start: usize) -> usize {
    lines
        .iter()
        .enumerate()
        .skip(start.saturating_add(1))
        .find(|(_, l)| l.starts_with("## "))
        .map_or(lines.len(), |(i, _)| i)
}

/// The line index a row of `kind` goes AT: just past the last non-blank line
/// of its section. `None` when the section is absent -- adding one is a
/// decision about where it sits, which is the format's order and not this
/// verb's.
fn insertion_point(text: &str, kind: char) -> Option<usize> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().position(|l| is_header_for(l, kind))?;
    let last = (start..section_end(&lines, start))
        .rev()
        .find(|i| lines.get(*i).is_some_and(|l| !l.trim().is_empty()))?;
    Some(last.saturating_add(1))
}

/// `text` with line `at` replaced by `row`, or `row` inserted there when
/// `replace` is false. The trailing newline is kept as it was.
fn spliced(text: &str, at: usize, row: &str, replace: bool) -> String {
    let mut lines: Vec<&str> = text.lines().collect();
    if replace {
        lines.remove(at.min(lines.len().saturating_sub(1)));
    }
    lines.insert(at.min(lines.len()), row);
    let mut out = lines.join("\n");
    if text.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Append one row to its section.
///
/// `today` is `YYYY-MM-DD`, passed in because the core reads no clock.
pub(crate) fn add(
    text: &str,
    section: &str,
    given: &[String],
    today: &str,
) -> Result<Edited, String> {
    let kind = kind_of(section)?;
    let row = new_row(text, kind, given, today)?;
    let at = insertion_point(text, kind).ok_or_else(|| {
        format!("no \u{a7}{kind} section to add to -- create its header first")
    })?;
    let out = spliced(text, at, &row, false);
    proven(text, &out)?;
    Ok((out, row))
}

/// The line indices of every row declaring `id`.
fn declaring(text: &str, id: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| at_line_start(l).is_some_and(|i| i.label() == id))
        .map(|(n, _)| n)
        .collect()
}

/// The line index of the one row declaring `id`.
fn locate(text: &str, id: &str) -> Result<usize, String> {
    let writable = id.chars().next().is_some_and(|k| WRITABLE.contains(&k));
    match declaring(text, id).as_slice() {
        [at] if writable => Ok(*at),
        [_] | [] => {
            Err(format!("no \u{a7}V, \u{a7}B or \u{a7}T row {id} here"))
        }
        more => Err(format!(
            "{id} is declared {} times -- ids must be unique",
            more.len()
        )),
    }
}

/// Replace the TEXT of row `id`, keeping its id and everything else.
///
/// The text is the statement of a `\u{a7}V` row (its trailing cites cell
/// stays), the task of a `\u{a7}T` row and the cause of a `\u{a7}B` row.
pub(crate) fn amend(text: &str, id: &str, new: &str) -> Result<Edited, String> {
    one_line(new)?;
    let at = locate(text, id)?;
    let old = text.lines().nth(at).unwrap_or("");
    let row = rewritten(old, id, &escape(new.trim()))?;
    let out = spliced(text, at, &row, true);
    proven(text, &out)?;
    Ok((out, row))
}

/// `old` with its text cell swapped for `new` (already escaped).
fn rewritten(old: &str, id: &str, new: &str) -> Result<String, String> {
    if id.starts_with('V') {
        return statement(old, id, new);
    }
    // `T12|status|text|cites`, `B12|date|cause|fix`: the third field.
    let mut parts: Vec<String> =
        cells(old).iter().map(|s| (*s).to_owned()).collect();
    let four = parts.len() >= 4;
    match parts.get_mut(2) {
        Some(slot) if four => *slot = new.to_owned(),
        _ => return Err(format!("{id} is not a four-field row to amend")),
    }
    Ok(parts.join("|"))
}

/// `V12: statement|cites` with the statement swapped. The cites cell, when
/// there is one, is the last field and stays exactly as written.
fn statement(old: &str, id: &str, new: &str) -> Result<String, String> {
    let parts = cells(old);
    let tail = parts
        .last()
        .filter(|_| parts.len() > 1)
        .map_or_else(String::new, |c| format!("|{c}"));
    let colon = old
        .find(':')
        .ok_or_else(|| format!("{id} has no `:` to amend after"))?;
    let lead = old.get(..colon.saturating_add(1)).unwrap_or("");
    Ok(format!("{lead} {new}{tail}"))
}

/// A violation, by what it says rather than where: inserting a row moves
/// every line after it.
fn named(text: &str) -> Vec<(String, String)> {
    check_spec(text, &[])
        .into_iter()
        .map(|v| (v.rule, v.msg))
        .collect()
}

/// What `after` reports that `before` did not, by rule and message.
fn introduced(before: &str, after: &str) -> Vec<String> {
    let known = named(before);
    named(after)
        .into_iter()
        .filter(|v| !known.contains(v))
        .map(|(rule, msg)| format!("{rule}: {msg}"))
        .collect()
}

/// Whether `fmt` would leave the text as it is.
fn tidy(text: &str) -> bool {
    format_spec(text).is_ok_and(|f| f == text)
}

/// The edit passes only if it adds no violation and leaves `fmt` nothing to do.
///
/// Judged AGAINST the file as it was: a spec that already carries findings is
/// still editable, but the edit may not add one.
fn proven(before: &str, after: &str) -> Result<(), String> {
    let fresh = introduced(before, after);
    if !fresh.is_empty() {
        return Err(format!(
            "refusing to write: the edit would add -- {}",
            fresh.join("; ")
        ));
    }
    if tidy(before) && !tidy(after) {
        return Err("refusing to write: the edit would leave the file \
                    needing `mth fmt`"
            .to_owned());
    }
    Ok(())
}

/// Today as `YYYY-MM-DD` (UTC), from the system clock.
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

fn month_len(year: u64, month: usize) -> u64 {
    match month {
        1 if leap(year) => 29,
        1 => 28,
        3 | 5 | 8 | 10 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 -> the year they fall in, and the days into it.
fn year_of(mut days: u64) -> (u64, u64) {
    let mut year: u64 = 1970;
    loop {
        let len = if leap(year) { 366 } else { 365 };
        if days < len {
            return (year, days);
        }
        days = days.saturating_sub(len);
        year = year.saturating_add(1);
    }
}

/// Days into a year -> the 0-based month, and the 0-based day in it.
fn month_of(year: u64, mut days: u64) -> (u64, u64) {
    let mut month = 0;
    while days >= month_len(year, month) {
        days = days.saturating_sub(month_len(year, month));
        month = month.saturating_add(1);
    }
    (u64::try_from(month).unwrap_or(0), days)
}

/// Days since 1970-01-01 -> `YYYY-MM-DD`, counted out year by year and month
/// by month: the crate has no date dependency, and one string does not earn
/// one.
fn civil(days: u64) -> String {
    let (year, into) = year_of(days);
    let (month, day) = month_of(year, into);
    format!(
        "{year:04}-{:02}-{:02}",
        month.saturating_add(1),
        day.saturating_add(1)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "\
## \u{a7}V INVARIANTS
V1: **a rule.** text|T1
V3: **another.** see V1

## \u{a7}T TASKS
id|status|task|cites
T1|x|do it|V1

## \u{a7}B BUGS
id|date|cause|fix
B2|2026-08-02|a cause|V1
";

    fn s(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|p| (*p).to_owned()).collect()
    }

    /// The row an accepted `add` wrote, or "" when it was refused.
    fn added(spec: &str, section: &str, cells: &[&str]) -> String {
        add(spec, section, &s(cells), "2026-11-01")
            .unwrap_or_default()
            .1
    }

    fn refusal(r: Result<Edited, String>) -> String {
        r.err().unwrap_or_default()
    }

    #[test]
    fn civil_dates_are_right() {
        assert_eq!(civil(0), "1970-01-01");
        assert_eq!(civil(20_758), "2026-11-01");
        assert_eq!(civil(11_016), "2000-02-29");
        assert_eq!(civil(19_723), "2024-01-01");
    }

    /// The next id is past the highest ever declared, so a gap is not filled.
    #[test]
    fn add_assigns_the_next_free_id_per_section() {
        assert_eq!(
            added(SPEC, "V", &["**new.** see V1"]),
            "V4: **new.** see V1"
        );
        assert_eq!(added(SPEC, "B", &["why", "V1"]), "B3|2026-11-01|why|V1");
        assert_eq!(added(SPEC, "\u{a7}T", &["more", "-"]), "T2|.|more|-");
    }

    /// The row lands inside its own section, and the result is clean.
    #[test]
    fn add_inserts_at_the_end_of_the_section() {
        let (out, _) =
            add(SPEC, "T", &s(&["more", "V1"]), "d").unwrap_or_default();
        let t2 = out.find("T2|").unwrap_or(usize::MAX);
        assert!(t2 < out.find("## \u{a7}B").unwrap_or(0), "{out}");
        assert!(out.ends_with('\n'));
        assert!(check_spec(&out, &[]).is_empty());
    }

    /// V18's rejection side: a citation that does not resolve.
    #[test]
    fn add_refuses_a_citation_that_does_not_resolve() {
        let e = refusal(add(SPEC, "V", &s(&["see V99"]), "d"));
        assert!(e.contains("V99"), "{e}");
    }

    /// ...and the other ways the edit could break the format.
    #[test]
    fn add_refuses_a_malformed_call() {
        assert!(add(SPEC, "V", &s(&["a\nb"]), "d").is_err());
        assert!(add(SPEC, "V", &s(&["a", "b"]), "d").is_err());
        assert!(add(SPEC, "X", &s(&["a"]), "d").is_err());
        assert!(add("V1: a\n", "V", &s(&["b"]), "d").is_err());
    }

    /// A pipe in a cell is escaped so the row keeps its field count.
    #[test]
    fn add_escapes_a_pipe_in_a_cell() {
        assert_eq!(added(SPEC, "T", &["a|b", "-"]), "T2|.|a\\|b|-");
    }

    fn amended(id: &str, new: &str) -> String {
        amend(SPEC, id, new).unwrap_or_default().1
    }

    /// The companion: amend keeps the id and the other cells, per kind.
    #[test]
    fn amend_replaces_the_text_and_keeps_the_rest() {
        assert_eq!(amended("V1", "**fresh.** text"), "V1: **fresh.** text|T1");
        assert_eq!(amended("V3", "**fresh.**"), "V3: **fresh.**");
        assert_eq!(amended("T1", "redone"), "T1|x|redone|V1");
        assert_eq!(amended("B2", "other"), "B2|2026-08-02|other|V1");
    }

    #[test]
    fn amend_changes_one_line_and_nothing_else() {
        let (out, _) = amend(SPEC, "T1", "redone").unwrap_or_default();
        assert_eq!(out.lines().count(), SPEC.lines().count());
        assert_eq!(out.replace("redone", "do it"), SPEC);
    }

    #[test]
    fn amend_refuses_a_missing_id_and_a_bad_text() {
        assert!(refusal(amend(SPEC, "V9", "x")).contains("V9"));
        assert!(amend(SPEC, "G1", "x").is_err());
        assert!(refusal(amend(SPEC, "V1", "see V99")).contains("V99"));
        assert!(amend(SPEC, "V1", "two\nlines").is_err());
    }

    /// A spec that already has a finding is still editable; the edit just may
    /// not add another.
    #[test]
    fn an_existing_finding_does_not_block_an_unrelated_edit() {
        let spec = SPEC.replace("V3:", "V3: dangling V77 ");
        assert!(add(&spec, "V", &s(&["fine"]), "d").is_ok());
        assert!(add(&spec, "V", &s(&["see V88"]), "d").is_err());
        assert!(amend(&spec, "V1", "fine").is_ok());
    }
}
