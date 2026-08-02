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

use crate::check::{cited, declared, outside_backticks, ITEM_KINDS};
use crate::id::at_line_start;
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
    out.push_str(&duplication_section(text, false));
    out
}

/// The same, plus every statement's own size, biggest FIRST.
///
/// `derive` already speaks, so verbose here DEEPENS rather than confirms
/// (§I). The aggregate answers "is the spec big?"; this answers "what do I
/// cut?", which is the question anyone actually has when the ceiling fires
/// -- and the aggregate cannot answer it, because a mean hides the one
/// statement that is three times the others.
///
/// Biggest first because a list sorted by id makes a reader do the ranking
/// themselves, and the whole point is to put the next edit at the top.
#[must_use]
pub fn report_verbose(text: &str) -> String {
    let mut out = report(text).replace(&duplication_section(text, false), "");
    out.push_str(&duplication_section(text, true));
    for kind in ITEM_KINDS {
        let mut all = sizes(text, kind);
        all.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for (id, chars) in all {
            out.push_str(&format!("stmt {id}: {chars} chars\n"));
        }
    }
    out
}

/// The MEASURED duplication threshold, in words.
///
/// Not chosen, and deliberately not a flag (V5). Measured against the two
/// real finds this repo has on record: 5-6 words admit coincidences of
/// ordinary phrasing, 7-10 yield only the genuine hit, and 12 misses it. A
/// threshold with a knob is one somebody lowers to make a report go quiet.
pub const PHRASE: usize = 8;

/// One phrase said twice, and the two statements that say it.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Duplicate {
    /// The earlier statement's id.
    pub a: String,
    /// The later one's.
    pub b: String,
    /// The shared wording, lowercased and stripped of backticked spans.
    pub phrase: String,
}

impl Duplicate {
    /// How long the shared phrase is, in words -- the number worth ranking by.
    #[must_use]
    pub fn words(&self) -> usize {
        self.phrase.split_whitespace().count()
    }
}

/// Every statement's id and its comparable words.
///
/// Backticked spans are stripped by V13's own boundary rather than a second
/// one: a shared command name is not shared REASONING, and `mth check
/// --records` appearing in two rows says nothing about either. Lowercased,
/// because case was a bug in the hand-run version -- the same clause in two
/// registers is the same clause.
fn statements(text: &str) -> Vec<(String, Vec<String>)> {
    text.lines()
        .filter_map(|line| {
            let id = at_line_start(line)?;
            Some((id.label(), significant(line)))
        })
        .collect()
}

fn significant(line: &str) -> Vec<String> {
    outside_backticks(line)
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Every phrase of `PHRASE` words or more that two statements share.
///
/// MAXIMAL only. A fourteen-word overlap is one finding, not the seven
/// eight-word windows inside it -- reporting those would bury the answer in
/// its own restatements, which is the failure this whole pass exists to name.
#[must_use]
pub fn duplicates(text: &str) -> Vec<Duplicate> {
    let stmts = statements(text);
    let mut out: Vec<Duplicate> = grams(&stmts)
        .windows(2)
        .flat_map(|w| pair(w, &stmts))
        .collect();
    out.sort_by(|x, y| y.words().cmp(&x.words()).then(x.cmp(y)));
    out.dedup();
    out
}

/// Every `PHRASE`-word window in every statement, sorted so equal windows
/// become neighbours. The index is what keeps this linear-ish rather than
/// comparing all pairs of statements word by word.
fn grams(stmts: &[(String, Vec<String>)]) -> Vec<(String, usize, usize)> {
    let mut out: Vec<(String, usize, usize)> = Vec::new();
    for (s, (_, words)) in stmts.iter().enumerate() {
        for p in 0..words.len().saturating_sub(PHRASE.saturating_sub(1)) {
            if let Some(g) = words.get(p..p.saturating_add(PHRASE)) {
                out.push((g.join(" "), s, p));
            }
        }
    }
    out.sort();
    out
}

/// Two neighbouring windows: a finding if they are equal, from different
/// statements, and this is the LEFTMOST window of the shared run.
fn pair(
    w: &[(String, usize, usize)],
    stmts: &[(String, Vec<String>)],
) -> Option<Duplicate> {
    let ((ga, sa, pa), (gb, sb, pb)) = (w.first()?, w.get(1)?);
    if ga != gb || sa == sb {
        return None;
    }
    let (ida, wa) = stmts.get(*sa)?;
    let (idb, wb) = stmts.get(*sb)?;
    if !leftmost(wa, *pa, wb, *pb) {
        return None;
    }
    let len = extended(wa, *pa, wb, *pb);
    let phrase = wa.get(*pa..pa.saturating_add(len))?.join(" ");
    Some(ordered(ida, idb, phrase))
}

/// Whether the word before each position differs -- so a run is reported
/// once, at its start, instead of once per window inside it.
fn leftmost(wa: &[String], pa: usize, wb: &[String], pb: usize) -> bool {
    match (pa.checked_sub(1), pb.checked_sub(1)) {
        (Some(a), Some(b)) => wa.get(a) != wb.get(b),
        _ => true,
    }
}

/// How far the match runs to the right, in words.
fn extended(wa: &[String], pa: usize, wb: &[String], pb: usize) -> usize {
    let mut len = PHRASE;
    while wa.get(pa.saturating_add(len)).is_some()
        && wa.get(pa.saturating_add(len)) == wb.get(pb.saturating_add(len))
    {
        len = len.saturating_add(1);
    }
    len
}

/// An id's sort position: kind first, then the number, then the suffix.
///
/// Parsed by the shared grammar rather than compared as text (V7).
fn rank(label: &str) -> (char, u32, String) {
    at_line_start(&format!("{label}:"))
        .map(|i| (i.kind, i.num, i.suffix))
        .unwrap_or((' ', 0, String::new()))
}

/// Ids in a stable order, so the same finding reads the same way twice.
///
/// Ordered by the id GRAMMAR, not lexically. `V21` sorts before `V5` as a
/// string, which is the same trap V14 exists to avoid, and it made the pass
/// report a pair backwards from how anyone would cite it.
fn ordered(x: &str, y: &str, phrase: String) -> Duplicate {
    let (a, b) = if rank(x) <= rank(y) { (x, y) } else { (y, x) };
    Duplicate {
        a: a.to_owned(),
        b: b.to_owned(),
        phrase,
    }
}

/// The duplication section of the report, longest phrase first.
///
/// Names BOTH ids and recommends NEITHER (V10). Which copy dies is
/// judgement: in both finds on record the general rule should own the
/// reasoning and the record should cite it -- but deciding which statement is
/// the general one is a call only a reader can make.
fn duplication_section(text: &str, full: bool) -> String {
    let found = duplicates(text);
    if found.is_empty() {
        return "dup: none\n".to_owned();
    }
    found
        .iter()
        .map(|d| {
            let shown = if full {
                d.phrase.clone()
            } else {
                gist(&d.phrase)
            };
            format!("dup {} {}: {} words -- \"{shown}\"\n", d.a, d.b, d.words())
        })
        .collect()
}

/// Enough of the phrase to recognise it; `--verbose` prints all of it.
fn gist(phrase: &str) -> String {
    let short: String = phrase.chars().take(50).collect();
    if short.len() < phrase.len() {
        return format!("{short}...");
    }
    short
}

fn sizes_section(text: &str) -> String {
    let mut out = String::new();
    for kind in ITEM_KINDS {
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

    /// Verbose adds per-statement sizes, biggest first, and keeps
    /// everything the plain report said -- it deepens, never replaces.
    #[test]
    fn verbose_ranks_every_statement_biggest_first() {
        let out = report_verbose(SPEC);
        assert!(out.contains("size V:"), "keeps the aggregate: {out}");
        assert!(out.contains("orphan V2:"), "keeps the orphans: {out}");
        let stmts: Vec<&str> =
            out.lines().filter(|l| l.starts_with("stmt ")).collect();
        assert_eq!(stmts.len(), 4, "three invariants and a task: {stmts:?}");
        assert!(
            stmts.first().is_some_and(|l| l.starts_with("stmt V3:")),
            "longest first: {stmts:?}"
        );
    }

    /// The two finds this repo has on record, planted (V18). Both were
    /// missed by reading -- a repeated clause reads perfectly well in both
    /// places -- and both were found by measuring.
    #[test]
    fn the_two_recorded_finds_are_caught() {
        let text = "## \u{a7}V INVARIANTS\n\
            V5: a ceiling a hair above current turns every legitimate \
            addition into a raise here\n\
            V21: a ceiling a hair above current turns every legitimate \
            addition into a raise\n";
        let got = duplicates(text);
        assert_eq!(got.len(), 1, "{got:?}");
        assert_eq!(
            got.first().map(|d| (d.a.as_str(), d.b.as_str())),
            Some(("V5", "V21"))
        );
        // 13 here, where the real find was 14: this fixture is a trimmed
        // stand-in, not the historical text. What matters is that the run
        // EXTENDED past the threshold rather than stopping at it.
        assert!(got.first().is_some_and(|d| d.words() > PHRASE), "{got:?}");
    }

    /// A fourteen-word overlap is ONE finding, not the seven eight-word
    /// windows inside it. Reporting those would bury the answer in its own
    /// restatements -- the very failure this pass exists to name.
    #[test]
    fn a_long_overlap_is_reported_once_at_its_full_length() {
        let shared = "one two three four five six seven eight nine ten";
        let text = format!(
            "## \u{a7}V INVARIANTS\nV1: {shared} alpha\nV2: {shared} beta\n"
        );
        let got = duplicates(&text);
        assert_eq!(got.len(), 1, "maximal only: {got:?}");
        assert_eq!(got.first().map(Duplicate::words), Some(10));
    }

    /// Under the threshold is silence. Seven words is where coincidence
    /// lives, and a pass that reports it is one people stop reading.
    #[test]
    fn a_short_overlap_is_not_reported() {
        let shared = "one two three four five six seven";
        let text = format!(
            "## \u{a7}V INVARIANTS\nV1: {shared} alpha\nV2: {shared} beta\n"
        );
        assert_eq!(duplicates(&text), Vec::new());
    }

    /// V13's boundary, reused rather than re-derived: a shared COMMAND is
    /// not shared reasoning. Two rows both quoting the same invocation say
    /// nothing about each other.
    #[test]
    fn a_shared_backticked_span_is_not_duplication() {
        let cmd = "`mth check --records .spec-records SPEC.md --format json`";
        let text = format!(
            "## \u{a7}V INVARIANTS\nV1: run {cmd} now\nV2: run {cmd} later\n"
        );
        assert_eq!(duplicates(&text), Vec::new());
    }

    /// Case was a bug in the hand-run version: the same clause in two
    /// registers is the same clause.
    #[test]
    fn duplication_is_case_insensitive() {
        let text = "## \u{a7}V INVARIANTS\n\
            V1: a rule and its bug record each restate the justification\n\
            V2: A RULE AND ITS BUG RECORD EACH RESTATE THE JUSTIFICATION\n";
        assert_eq!(duplicates(text).len(), 1);
    }

    /// It names BOTH and recommends NEITHER (V10). Which copy dies is a
    /// judgement about which statement is the general one, and only a reader
    /// can make it.
    #[test]
    fn the_report_names_both_and_recommends_neither() {
        let shared = "one two three four five six seven eight nine";
        let text = format!(
            "## \u{a7}V INVARIANTS\nV1: {shared} alpha\nV2: {shared} beta\n"
        );
        let out = duplication_section(&text, false);
        assert!(out.contains("dup V1 V2: 9 words"), "{out}");
        assert!(!out.to_lowercase().contains("remove"), "{out}");
        assert!(!out.to_lowercase().contains("delete"), "{out}");
    }

    /// A clean spec says so, rather than saying nothing -- silence would be
    /// indistinguishable from the pass not having run.
    #[test]
    fn a_spec_with_no_duplication_says_none() {
        assert_eq!(duplication_section(SPEC, false), "dup: none\n");
        assert!(report(SPEC).contains("dup: none"));
    }

    /// `--verbose` prints the phrase in full; the plain report truncates.
    #[test]
    fn verbose_prints_the_whole_phrase() {
        let shared = "alpha bravo charlie delta echo foxtrot golf hotel india \
                      juliet kilo lima mike november";
        let text = format!(
            "## \u{a7}V INVARIANTS\nV1: {shared} one\nV2: {shared} two\n"
        );
        assert!(
            duplication_section(&text, false).contains("..."),
            "truncates"
        );
        let full = duplication_section(&text, true);
        assert!(full.contains("november"), "{full}");
        assert!(!full.contains("..."), "{full}");
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
