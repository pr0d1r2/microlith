//! §V ENUMERATED for a machine: which rules exist, and which are retired.
//!
//! The sibling of `tasks` and `bugs`, for the section whose rows are not
//! pipe rows but `V<n>: statement` lines. Nothing here reads the mark: a
//! rule is superseded exactly when `check::supersessions` -- the reading V41
//! gates on -- says so, and the winners are the ones it names (V7). A second
//! reading of `[superseded by ...]` would be the drift this crate exists to
//! end, and the consumer that prompted this verb had one that printed
//! nothing on `V1: ...` lines.
//!
//! Order is V14's. Report-only (V10): `--active` and `--superseded`
//! PARTITION the ids, they do not rank them (V6).

use crate::id::at_line_start;

/// Which states a caller asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Filter {
    All,
    Active,
    Superseded,
}

impl Filter {
    /// `"active"` / `"superseded"`; anything else asks for both.
    pub(crate) fn of(name: &str) -> Self {
        match name {
            "active" => Self::Active,
            "superseded" => Self::Superseded,
            _ => Self::All,
        }
    }

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
    /// `Some(winners)` when retired; the list may be empty only never --
    /// a mark naming no id is not a mark.
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
    let mut found: Vec<(crate::id::Id, Invariant)> = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let Some(id) = at_line_start(line).filter(|i| i.kind == 'V') else {
            continue;
        };
        if found.iter().all(|(_, v)| v.id != id.label()) {
            let inv = declared(line, index.saturating_add(1), &id, &marks);
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
    id: &crate::id::Id,
    marks: &[crate::check::Retirement],
) -> Invariant {
    let label = id.label();
    let winners = marks
        .iter()
        .find(|(at, mid, _)| *at == line_no && *mid == label)
        .map(|(_, _, w)| w.clone());
    let body = line.split_once(':').map_or("", |(_, rest)| rest.trim());
    Invariant {
        id: label,
        text: body.to_owned(),
        winners,
    }
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
        return crate::rows::none_read("invariants", 'V', unread);
    }
    let mut out = head(&rows, unread);
    for v in rows {
        out.push_str(&line(v, full));
    }
    out
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
