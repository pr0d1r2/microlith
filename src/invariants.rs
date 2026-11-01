//! §V ENUMERATED for a machine: which rules exist, and which are retired.
//!
//! The sibling of `tasks` and `bugs`, for the section whose rows are not
//! pipe rows but `V<n>: statement` lines. Nothing here decides what a rule
//! is: the ids are the ones `check::declared_at` calls declared, so this list
//! and `check`'s count cannot differ, and a rule is superseded exactly when
//! `check::supersessions` -- the reading V41 gates on -- says so, with the
//! winners it names (V7). A second reading of either would be the drift this
//! crate exists to end, and the consumer that prompted this verb had one that
//! printed nothing on `V1: ...` lines.
//!
//! Order is V14's. Report-only (V10): `--active` and `--superseded`
//! PARTITION the ids, they do not rank them (V6).

use crate::id::Id;

/// Which states a caller asked for.
///
/// An enum rather than a string, because `mth` refuses an unknown spelling
/// and a library caller deserves the same: a typo that fell back to "all"
/// would hand retired rules to someone who asked for live ones.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Filter {
    /// Every rule.
    All,
    /// Only the rules still in force.
    Active,
    /// Only the rules a `[superseded by ...]` mark retires.
    Superseded,
}

impl Filter {
    fn keeps(self, inv: &Invariant) -> bool {
        match self {
            Self::All => true,
            Self::Active => inv.winners.is_none(),
            Self::Superseded => inv.winners.is_some(),
        }
    }
}

/// One `§V` declaration.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Invariant {
    pub id: String,
    /// The statement after `V<n>:`, trimmed.
    pub text: String,
    /// `Some(winners)` when retired, never empty: a mark naming no id is not
    /// a mark.
    pub winners: Option<Vec<String>>,
}

impl Invariant {
    fn state(&self) -> &'static str {
        if self.winners.is_some() {
            "superseded"
        } else {
            "active"
        }
    }
}

/// Every `§V` declaration in V14 order; the first of a repeated id wins.
pub(crate) fn invariants(text: &str) -> Vec<Invariant> {
    let marks = crate::check::supersessions(text);
    let lines: Vec<&str> = text.lines().collect();
    let mut found: Vec<(Id, Invariant)> = Vec::new();
    for (at, id) in crate::check::declared_at(text, 'V') {
        if found.iter().all(|(_, v)| v.id != id.label()) {
            let line = lines.get(at.saturating_sub(1)).copied();
            let inv = declared(line.unwrap_or(""), at, &id, &marks);
            found.push((id, inv));
        }
    }
    found.sort_by_key(|(id, _)| id.sort_key());
    found.into_iter().map(|(_, v)| v).collect()
}

/// One declaration line, with the winners V41's reading gives it.
fn declared(
    line: &str,
    line_no: usize,
    id: &Id,
    marks: &[crate::check::Retirement],
) -> Invariant {
    let label = id.label();
    let winners = marks
        .iter()
        .find(|(at, mid, _)| *at == line_no && *mid == label)
        .map(|(_, _, w)| w.clone());
    Invariant {
        text: statement(line, id).to_owned(),
        id: label,
        winners,
    }
}

/// What follows the id and the terminator the grammar found after it.
///
/// Not a search for the first `:`, which a bulleted `- V1 a rule: why` only
/// has inside its statement, and `* V1 a rule` does not have at all. Nor for
/// the label: `V01:` is labelled `V1`, so that text is not on the line, or is
/// a citation further along it. The first `kind` is the id's own, because a
/// list marker holds none, and the first terminator past it is the grammar's.
fn statement<'a>(line: &'a str, id: &Id) -> &'a str {
    line.split_once(id.kind)
        .and_then(|(_, rest)| rest.split_once(id.terminator))
        .map_or("", |(_, rest)| rest.trim())
}

fn unread(text: &str) -> usize {
    crate::rows::unread(text, 'V')
}

/// The human rendering: a count line, then one line per rule.
pub(crate) fn report(text: &str, filter: Filter, full: bool) -> String {
    let all = invariants(text);
    let rows: Vec<&Invariant> =
        all.iter().filter(|v| filter.keeps(v)).collect();
    let unread = unread(text);
    if rows.is_empty() {
        return empty_report(&all, filter, unread);
    }
    let mut out = head(&rows, unread);
    for v in rows {
        out.push_str(&line(v, full));
    }
    out
}

fn empty_report(all: &[Invariant], filter: Filter, unread: usize) -> String {
    if !all.is_empty() {
        // Rules exist and none was kept, so a narrowing filter is why.
        let state = if filter == Filter::Active {
            "active"
        } else {
            "superseded"
        };
        return format!(
            "invariants: none -- no readable {state} rules match{}\n",
            crate::rows::also(unread)
        );
    }
    crate::rows::none_read("invariants", 'V', unread)
}

/// The count line: how many rules, split by state, and how many were there
/// but unreadable.
fn head(rows: &[&Invariant], unread: usize) -> String {
    let retired = rows.iter().filter(|v| v.winners.is_some()).count();
    format!(
        "invariants: {} rules -- {} active, {retired} superseded{}\n",
        rows.len(),
        rows.len().saturating_sub(retired),
        crate::rows::also(unread)
    )
}

/// One rule's line: its state, the winners when retired, then its gist.
fn line(v: &Invariant, full: bool) -> String {
    let shown = crate::rows::shown(&v.text, full);
    let state = match &v.winners {
        Some(w) => format!("superseded by {}", w.join(", ")),
        None => "active".to_owned(),
    };
    format!("invariant {}: {state} -- {shown}\n", v.id)
}

/// The machine rendering: one object, `invariants` in V14 order.
pub(crate) fn json(file: &str, text: &str, filter: Filter) -> String {
    let items: Vec<String> = invariants(text)
        .iter()
        .filter(|v| filter.keeps(v))
        .map(one_json)
        .collect();
    format!(
        "{{\"file\":{},\"invariants\":[{}],\"unread\":{}}}\n",
        crate::render::quote(file),
        items.join(","),
        unread(text)
    )
}

fn one_json(v: &Invariant) -> String {
    let winners: Vec<String> = v
        .winners
        .iter()
        .flatten()
        .map(|w| crate::render::quote(w))
        .collect();
    format!(
        "{{\"id\":{},\"state\":{},\"superseded_by\":[{}]}}",
        crate::render::quote(&v.id),
        crate::render::quote(v.state()),
        winners.join(",")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "\
## \u{a7}V INVARIANTS
V1: **kept.** still law.
V2: **old.** [superseded by V1]
V10: **split.** [superseded by V1, V3]
V3: **kept too.** see V2.
V2a: **suffixed.** a rider.
";

    fn ids(text: &str, f: Filter) -> Vec<String> {
        invariants(text)
            .into_iter()
            .filter(|v| f.keeps(v))
            .map(|v| v.id)
            .collect()
    }

    /// Acceptance: the two filters partition the ids.
    #[test]
    fn active_and_superseded_partition_the_ids() {
        let all = ids(SPEC, Filter::All);
        let mut both = ids(SPEC, Filter::Active);
        both.extend(ids(SPEC, Filter::Superseded));
        both.sort();
        let mut want = all.clone();
        want.sort();
        assert_eq!(both, want);
        assert_eq!(all, vec!["V1", "V2", "V2a", "V3", "V10"]);
        assert_eq!(ids(SPEC, Filter::Superseded), vec!["V2", "V10"]);
    }

    /// The consumer's failing case: canonical `V1: ...` lines are listed.
    #[test]
    fn canonical_lines_are_not_an_empty_list() {
        assert_eq!(
            ids(SPEC, Filter::Active).first().map(String::as_str),
            Some("V1")
        );
    }

    /// A marked pipe row with the same id does not retire the declaration.
    #[test]
    fn marks_match_their_declaration_line() {
        let t = "V1: kept.\nV1|x|old|[superseded by V2]\nV2: replacement.\n";
        let got = invariants(t);
        assert_eq!(
            got.iter().map(|v| v.id.as_str()).collect::<Vec<_>>(),
            ["V1", "V2"]
        );
        assert!(
            got.iter()
                .find(|v| v.id == "V1")
                .is_some_and(|v| v.winners.is_none())
        );
    }

    /// V7: the list is `check`'s. A pipe row declares its id there, so
    /// dropping it here would name a winner the list does not carry.
    #[test]
    fn the_ids_are_the_ones_check_declares() {
        let t = "V1: old. [superseded by V2]\nV2|a row|x|y\n";
        let declared: Vec<String> = crate::check::declared(t, 'V')
            .iter()
            .map(Id::label)
            .collect();
        assert_eq!(ids(t, Filter::All), declared);
        assert_eq!(ids(t, Filter::Active), ["V2"]);
    }

    /// The first declaration answers, and a later one cannot retire it.
    #[test]
    fn the_first_of_a_repeated_id_wins() {
        let got = invariants("V1: first.\nV1: second. [superseded by V2]\n");
        assert_eq!(got.len(), 1, "{got:?}");
        assert_eq!(got.first().map(|v| v.text.as_str()), Some("first."));
        assert!(got.first().is_some_and(|v| v.winners.is_none()));
    }

    /// V26: every declaration shape gives its whole statement. A search for
    /// the first `:` cut the `- ` line and blanked the `* ` and `+ ` ones.
    #[test]
    fn every_declaration_shape_gives_its_whole_statement() {
        let t = "- V1 why: because\n* V2 starred\n+ V3 plussed\n\
                 - V4: colon\nV5: why: because\nV6|a|b|c\n";
        let got: Vec<String> =
            invariants(t).into_iter().map(|v| v.text).collect();
        assert_eq!(
            got,
            [
                "why: because",
                "starred",
                "plussed",
                "colon",
                "why: because",
                "a|b|c"
            ]
        );
    }

    /// The label normalises `V01` to `V1`; the statement is still the one
    /// written, not blank and not whatever follows a later `V1:`.
    #[test]
    fn a_padded_id_keeps_its_own_statement() {
        let got: Vec<String> = invariants("V01: alone\nV02: on V2: a cite\n")
            .into_iter()
            .map(|v| v.text)
            .collect();
        assert_eq!(got, ["alone", "on V2: a cite"]);
    }

    /// `--verbose` stops truncating, as it does on `tasks` and `bugs`.
    #[test]
    fn verbose_prints_the_whole_statement() {
        let text = format!("V1: {}\n", "z".repeat(80));
        assert!(report(&text, Filter::All, false).contains("..."), "gist");
        assert!(!report(&text, Filter::All, true).contains("..."), "whole");
    }

    /// The JSON is the contract.
    #[test]
    fn json_is_stable() {
        let out = json("S.md", SPEC, Filter::Superseded);
        assert_eq!(
            out,
            "{\"file\":\"S.md\",\"invariants\":[\
             {\"id\":\"V2\",\"state\":\"superseded\",\"superseded_by\":[\"V1\"]},\
             {\"id\":\"V10\",\"state\":\"superseded\",\"superseded_by\":[\"V1\",\"V3\"]}\
             ],\"unread\":0}\n"
        );
        assert!(json("S.md", SPEC, Filter::Active).contains(
            "{\"id\":\"V1\",\"state\":\"active\",\"superseded_by\":[]}"
        ));
    }

    /// `"invariants":[]` is always emitted: a filter nothing matches is an
    /// answer, in the same shape as one that matched.
    #[test]
    fn json_with_nothing_kept_is_still_the_object() {
        assert_eq!(
            json("S.md", "V1: kept.\n", Filter::Superseded),
            "{\"file\":\"S.md\",\"invariants\":[],\"unread\":0}\n"
        );
    }

    /// Prose mentioning the words is not a mark (B30), and a mark in code
    /// is a literal.
    #[test]
    fn prose_and_code_are_not_marks() {
        let t = "V1: was superseded by V2 [B1].\nV2: `[superseded by V1]`\n";
        assert!(ids(t, Filter::Superseded).is_empty());
    }

    /// A bordered-dialect §V reports the dialect, not an empty list.
    #[test]
    fn a_dialect_section_is_reported_not_silent() {
        let t = "## \u{a7}V INVARIANTS\n\n| V1 | a rule |\n| V2 | another |\n";
        assert!(
            report(t, Filter::All, false).starts_with("invariants: none READ")
        );
        assert!(json("f", t, Filter::All).contains("\"unread\":2"));
        let empty = "## \u{a7}V INVARIANTS\n\nnothing.\n";
        assert!(
            report(empty, Filter::All, false)
                .starts_with("invariants: none --")
        );
    }

    #[test]
    fn an_empty_filter_is_not_an_empty_section() {
        let out = report("V1: kept.\n", Filter::Superseded, false);
        assert_eq!(
            out,
            "invariants: none -- no readable superseded rules match\n"
        );
    }

    #[test]
    fn an_empty_filter_reports_unread_mixed_dialect_rows() {
        let mixed =
            "## §V INVARIANTS\n\nV1: active.\n| V2 | unread superseded |\n";
        let out = report(mixed, Filter::Superseded, false);
        assert_eq!(
            out,
            "invariants: none -- no readable superseded rules match; 1 more \
             unread, in a dialect (`mth migrate`)\n"
        );
    }

    #[test]
    fn human_report_names_state_and_winners() {
        let out = report(SPEC, Filter::All, false);
        assert!(
            out.starts_with("invariants: 5 rules -- 3 active, 2 superseded\n"),
            "{out}"
        );
        assert!(
            out.contains("invariant V10: superseded by V1, V3 -- "),
            "{out}"
        );
        assert!(out.contains("invariant V1: active -- "), "{out}");
    }
}
