//! ONE reading of a four-field pipe row, for every section written as one.
//!
//! `§T`, `§B` and `§R` carry the SAME grammar -- an id, a terminator, three
//! cells, FORMAT.md's escape inside them -- and differ only in what the
//! cells are CALLED: `status|task|cites` against `date|cause|fix` against
//! `topic|finding|src`. A second enumeration would have been a second
//! reading of that grammar, which is the defect this crate exists to end
//! (V7), so the reading lives here and the NAMES live with the verb.
//!
//! The order is V14's, applied once: a suffixed id RIDES its base, so `B7a`
//! follows `B7` and never sorts lexically between `B1` and `B2`. Sorted HERE
//! rather than reported in file order, because a section whose rows are out
//! of order is a `check` finding and not a reason to hand a consumer a
//! different sequence than the format promises.

use crate::id::{Id, at_line_start, cells, unescape};

/// One pipe row: its id, and the three cells after it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Row {
    /// The row's id, e.g. `B7a`.
    pub id: String,
    /// The id's NUMBER -- `B7a` is 7. Held because a suffixed row RIDES its
    /// base (V14), so anything looking a row up by id has to ask about the
    /// number rather than about the label.
    pub num: u32,
    /// The three cells after the id, trimmed, with `\|` read back as `|`.
    ///
    /// VERBATIM, never validated: whether cell 0 of a `§T` row is one of
    /// `.` `~` `x` is V25's question, and a reader that silently dropped a
    /// row whose cell it disliked would hide exactly what V25 surfaces.
    pub cells: [String; 3],
}

/// Every row of one KIND, in V14 order.
pub(crate) fn of(text: &str, kind: char) -> Vec<Row> {
    let mut found: Vec<(Id, Row)> = text
        .lines()
        .filter_map(|line| {
            let id = at_line_start(line).filter(|i| i.kind == kind)?;
            let row = one(line, &id)?;
            Some((id, row))
        })
        .collect();
    found.sort_by_key(|(id, _)| id.sort_key());
    found.into_iter().map(|(_, row)| row).collect()
}

/// One row's cells. A line with no `|` at all -- V26's bulleted `- T1 text`
/// dialect, which has no fields -- carries no cells to report, so it is not
/// a row this reads.
fn one(line: &str, id: &Id) -> Option<Row> {
    let cells = cells(line);
    cells.get(1)?;
    let cell = |n: usize| unescape(cells.get(n).copied().unwrap_or("").trim());
    Some(Row {
        id: id.label(),
        num: id.num,
        cells: [cell(1), cell(2), cell(3)],
    })
}

/// `V1,V2` -> the two. `-` is FORMAT.md's empty cell, not a citation.
pub(crate) fn cites(cell: &str) -> Vec<String> {
    cell.split(',')
        .map(str::trim)
        .filter(|c| !c.is_empty() && *c != "-")
        .map(str::to_owned)
        .collect()
}

/// How many rows of one kind this build could not read (V49).
///
/// Asked of the CHECKER rather than counted again here: a second reading of
/// what a dialect row is would be the drift V7 exists to end, and an
/// enumerating verb and `check` disagreeing about whether a section has rows
/// is exactly the confusion the count was added to remove.
pub(crate) fn unread(text: &str, kind: char) -> usize {
    crate::check::unread_rows(text)
        .iter()
        .filter(|u| u.kind == kind)
        .map(|u| u.count)
        .sum()
}

/// Nothing was read -- which is TWO different facts, and saying `none` for
/// both is the whole of what was reported. An empty section and one nobody
/// here can parse are the states a caller must tell apart.
pub(crate) fn none_read(verb: &str, kind: char, unread: usize) -> String {
    match unread {
        0 => format!("{verb}: none -- no \u{a7}{kind} rows here\n"),
        n => format!(
            "{verb}: none READ -- {n} \u{a7}{kind} rows are here in a \
             dialect this build cannot read; `mth migrate` converts them\n"
        ),
    }
}

/// The same fact beside rows that WERE read: a partial answer said to be one.
pub(crate) fn also(unread: usize) -> String {
    match unread {
        0 => String::new(),
        n => format!("; {n} more unread, in a dialect (`mth migrate`)"),
    }
}

/// Enough of a cell to recognise it; `--verbose` prints all of it. The same
/// width `anchors` uses, so every report reads alike.
pub(crate) fn gist(text: &str) -> String {
    let short: String = text.chars().take(60).collect();
    if short.chars().count() < text.chars().count() {
        return format!("{short}...");
    }
    short
}

/// One cell, whole or gisted -- the `--verbose` choice, made once.
pub(crate) fn shown(text: &str, full: bool) -> String {
    if full {
        return text.to_owned();
    }
    gist(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `§T` row and a `§B` row in one fixture, because the point of this
    /// module is that the two are read by the same code.
    const SPEC: &str = "\
## \u{a7}T TASKS
T2|~|a wip task, `Mechanical`\\|`Judgment`|V1
T1|x|a done task|V1

## \u{a7}B BUGS
id|date|cause|fix
B2|2026-08-02|a second cause|V2
B1|2026-08-01|a cause|V1
";

    fn ids(text: &str, kind: char) -> Vec<String> {
        of(text, kind).into_iter().map(|r| r.id).collect()
    }

    /// The acceptance case, for BOTH kinds: rows come back sorted, and a
    /// kind never picks up the other's rows.
    #[test]
    fn each_kind_reads_only_its_own_rows_in_id_order() {
        assert_eq!(ids(SPEC, 'T'), vec!["T1", "T2"]);
        assert_eq!(ids(SPEC, 'B'), vec!["B1", "B2"]);
    }

    /// The companion (V18): the same three cells come back whichever section
    /// they were written in -- which is the claim that lets two verbs share
    /// one reading.
    #[test]
    fn the_three_cells_after_the_id_come_back_as_written() {
        let b1 = of(SPEC, 'B').into_iter().find(|r| r.id == "B1");
        assert_eq!(
            b1.map(|r| r.cells),
            Some([
                "2026-08-01".to_owned(),
                "a cause".to_owned(),
                "V1".to_owned(),
            ])
        );
    }

    /// V14: `B7a` RIDES `B7`. Lexically `"B10" < "B2"` is wrong, which is
    /// the trap the id grammar exists to avoid.
    #[test]
    fn a_suffixed_id_sorts_with_its_base_not_lexically() {
        let text = "## \u{a7}B BUGS\nB10|d|c|f\nB7a|d|c|f\nB7|d|c|f\n";
        assert_eq!(ids(text, 'B'), vec!["B7", "B7a", "B10"]);
        assert_eq!(of(text, 'B').first().map(|r| r.num), Some(7));
    }

    /// FORMAT.md's escape, and the reason `cells` exists: a naive split cuts
    /// this text in half and reads `` `Judgment` `` as a cell of its own.
    #[test]
    fn an_escaped_pipe_stays_inside_the_cell_it_was_written_in() {
        let t2 = of(SPEC, 'T').into_iter().find(|r| r.id == "T2");
        assert_eq!(
            t2.map(|r| r.cells[1].clone()),
            Some("a wip task, `Mechanical`|`Judgment`".to_owned())
        );
    }

    /// V26's other dialect: a declaration behind a markdown bullet is the
    /// SAME row, and a row with no fields at all is not one this can read.
    #[test]
    fn a_bulleted_row_reads_and_a_fieldless_one_does_not() {
        assert_eq!(ids(&SPEC.replace("\nB1|", "\n- B1|"), 'B'), ids(SPEC, 'B'));
        assert_eq!(
            ids("## \u{a7}B BUGS\n- B1 a bare sentence\n", 'B'),
            Vec::<String>::new()
        );
    }

    /// `-` is FORMAT.md's EMPTY cell, not a citation of a rule called `-`.
    #[test]
    fn an_empty_cell_is_no_citations() {
        assert_eq!(cites("-"), Vec::<String>::new());
        assert_eq!(cites("V1, V2"), vec!["V1", "V2"]);
    }

    /// The two spellings of "nothing here", which no verb may collapse.
    #[test]
    fn an_unread_section_is_not_spelled_like_an_empty_one() {
        assert!(none_read("bugs", 'B', 0).starts_with("bugs: none --"));
        assert!(none_read("bugs", 'B', 2).starts_with("bugs: none READ"));
        assert!(none_read("bugs", 'B', 2).contains("2 \u{a7}B rows"));
        assert_eq!(also(0), "");
        assert!(also(3).contains("3 more unread"));
    }

    /// The gist is what `--verbose` opts out of, and it says that it cut.
    #[test]
    fn a_long_cell_is_gisted_unless_the_caller_asked_for_all_of_it() {
        let long = "z".repeat(80);
        assert!(shown(&long, false).ends_with("..."));
        assert_eq!(shown(&long, true), long);
    }

    /// V49, asked per SECTION: an unread `§B` is not counted as an unread
    /// `§T`, which is what lets each verb report its own hole.
    #[test]
    fn the_unread_count_is_asked_per_section() {
        let text = "## \u{a7}B BUGS\n\n| B1 | d | c | f |\n";
        assert_eq!(unread(text, 'B'), 1);
        assert_eq!(unread(text, 'T'), 0);
    }
}
