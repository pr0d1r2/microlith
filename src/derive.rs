//! Facts DERIVED from a spec: sizes, the citation graph, orphan invariants.
//!
//! Report-only, all of it (V10). These answer questions about a spec rather
//! than passing judgement on it, and the difference matters: an orphan
//! invariant is a question for a human -- is the rule dead, or is something
//! failing to cite it? -- and a gate that answered it would answer by fiat.
//!
//! CPU only (V6). Sizes are CHARS, never tokens: a tokenizer is a dependency,
//! a model or a table, and none of those belong in a tool that must run
//! offline and give the same answer twice. `itok` is the tool for token cost,
//! and this crate does not duplicate it.
//!
//! Cheap because of decisions already made. One line per statement (V3) means
//! an invariant IS a line, so sizing is `len()` -- and `check` already owns
//! the id grammar and V13's citation boundary, so the graph is a regroup of
//! code that exists rather than a second parser for the same text (V7).

use crate::check::{cited, declared};
use crate::id::Id;

/// How often each invariant is cited, and by which rows.
#[derive(Debug, PartialEq, Eq)]
pub struct Citations {
    /// The invariant cited, e.g. `V13`.
    pub id: String,
    /// How many times, across the whole file.
    pub count: usize,
}

/// Every declared invariant with its citation count, most-cited first.
///
/// A DECLARATION is not a citation of itself, so `V13:` opening its own line
/// does not count. Without that, nothing is ever an orphan and the report
/// says only that every rule exists -- which nobody asked.
#[must_use]
pub fn citation_counts(text: &str) -> Vec<Citations> {
    let all = cited(text);
    let mut out: Vec<Citations> = declared(text, 'V')
        .iter()
        .map(|id| {
            let label = id.label();
            let count = all.iter().filter(|c| **c == label).count();
            Citations {
                id: label,
                // The declaration itself is in `all`; discount it.
                count: count.saturating_sub(1),
            }
        })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then(a.id.cmp(&b.id)));
    out
}

/// Invariants declared and cited by NOTHING.
///
/// Not a violation. An orphan is either a rule that has outlived its use or a
/// rule something should be citing and is not, and only a reader can say
/// which -- so this reports and stops (V10).
#[must_use]
pub fn orphans(text: &str) -> Vec<String> {
    citation_counts(text)
        .into_iter()
        .filter(|c| c.count == 0)
        .map(|c| c.id)
        .collect()
}

/// Size of every statement of one kind, as `(id, chars)`.
///
/// Chars rather than bytes, so a line of symbols is not reported as three
/// times its length -- this format uses `∴`, `⊥` and `§` heavily.
#[must_use]
pub fn sizes(text: &str, kind: char) -> Vec<(String, usize)> {
    let ids: Vec<Id> = declared(text, kind);
    text.lines()
        .filter_map(|line| {
            let id = crate::id::at_line_start(line)?;
            (id.kind == kind && ids.iter().any(|d| d.num == id.num))
                .then(|| (id.label(), line.chars().count()))
        })
        .collect()
}

/// Largest, smallest and mean of a set of sizes.
///
/// Mean rather than median: this is a report, and the mean is the number that
/// moves when one statement bloats, which is the thing worth seeing.
#[must_use]
pub fn spread(sizes: &[(String, usize)]) -> Option<(usize, usize, usize)> {
    let max = sizes.iter().map(|(_, n)| *n).max()?;
    let min = sizes.iter().map(|(_, n)| *n).min()?;
    let total: usize = sizes.iter().map(|(_, n)| *n).sum();
    Some((max, min, total.checked_div(sizes.len())?))
}

/// The whole report, one line per fact so every line greps (V3's habit,
/// applied to output).
#[must_use]
pub fn report(text: &str) -> String {
    let mut out = String::new();
    out.push_str(&sizes_section(text));
    out.push_str(&graph_section(text));
    out.push_str(&orphans_section(text));
    out
}

fn sizes_section(text: &str) -> String {
    let mut out = String::new();
    for kind in ['V', 'T', 'B'] {
        let s = sizes(text, kind);
        if let Some((max, min, mean)) = spread(&s) {
            out.push_str(&format!(
                "size {kind}: {} statements, max {max} chars, min {min}, \
                 mean {mean}\n",
                s.len()
            ));
        }
    }
    out
}

fn graph_section(text: &str) -> String {
    citation_counts(text)
        .iter()
        .map(|c| format!("cited {}: {}\n", c.id, c.count))
        .collect()
}

fn orphans_section(text: &str) -> String {
    let found = orphans(text);
    if found.is_empty() {
        return "orphan: none\n".to_owned();
    }
    found
        .iter()
        .map(|id| {
            format!("orphan {id}: declared, cited by nothing -- dead rule, or a missing citation?\n")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// V2 is deliberately cited by nothing: the orphan this fixture plants.
    const SPEC: &str = "\
## \u{a7}V INVARIANTS
V1: **a rule.** cited by T1 and by V3.
V2: **an orphan.** nothing points here.
V3: **a third.** see V1, and a literal `V99` that is not a citation.

## \u{a7}T TASKS
T1|x|a task|V1
";

    #[test]
    fn an_uncited_invariant_is_an_orphan() {
        assert_eq!(orphans(SPEC), vec!["V2"]);
    }

    /// The companion: a spec where every invariant is cited reports none, so
    /// the check cannot pass by calling everything an orphan (V18).
    #[test]
    fn a_fully_cited_spec_has_no_orphans() {
        let cited_all =
            SPEC.replace("nothing points here.", "see V2 twice: V2.");
        assert_eq!(orphans(&cited_all), Vec::<String>::new());
    }

    /// A declaration is not a citation of itself, or nothing is ever an
    /// orphan and the report degenerates into a list of what exists.
    #[test]
    fn a_declaration_does_not_cite_itself() {
        let counts = citation_counts(SPEC);
        let v2 = counts.iter().find(|c| c.id == "V2");
        assert_eq!(v2.map(|c| c.count), Some(0));
    }

    /// V13's boundary holds here too: the backticked `V99` in V3 is a
    /// literal, and V1 is cited from prose AND from a row's cites column.
    #[test]
    fn counts_prose_and_row_citations_but_not_backticked_ones() {
        let counts = citation_counts(SPEC);
        let of = |id: &str| counts.iter().find(|c| c.id == id).map(|c| c.count);
        assert_eq!(of("V1"), Some(2), "prose in V3 plus T1's cites column");
        assert!(!counts.iter().any(|c| c.id == "V99"), "backticked literal");
    }

    #[test]
    fn sizes_measure_the_whole_line_in_chars() {
        let s = sizes(SPEC, 'V');
        assert_eq!(s.len(), 3);
        assert_eq!(s.first().map(|(id, _)| id.as_str()), Some("V1"));
        let longest = "V3: **a third.** see V1, and a literal `V99` that is \
                       not a citation.";
        assert_eq!(s.get(2).map(|(_, n)| *n), Some(longest.chars().count()));
    }

    /// Chars, not bytes: this format is full of multi-byte symbols and a
    /// byte count would report them at three times their width.
    #[test]
    fn a_multibyte_symbol_counts_as_one_char() {
        let line = "V1: a \u{22a5} b \u{2234} c";
        let text = format!("## \u{a7}V INVARIANTS\n{line}\n");
        // The gap is the whole point: two 3-byte symbols read as 6 bytes and
        // 2 chars, so a byte count would inflate this line by a third.
        assert_eq!(line.len(), 17, "bytes");
        assert_eq!(sizes(&text, 'V').first().map(|(_, n)| *n), Some(13));
    }

    #[test]
    fn spread_reports_max_min_and_mean() {
        let s = vec![("V1".to_owned(), 10), ("V2".to_owned(), 20)];
        assert_eq!(spread(&s), Some((20, 10, 15)));
        assert_eq!(spread(&[]), None);
    }

    /// Every line of the report is one fact, so `derive | grep orphan` works.
    #[test]
    fn the_report_names_the_orphan_and_asks_the_question() {
        let out = report(SPEC);
        assert!(out.contains("orphan V2:"), "{out}");
        assert!(out.contains("missing citation?"), "{out}");
        assert!(out.contains("size V:"), "{out}");
        assert!(out.contains("cited V1: 2"), "{out}");
    }
}
