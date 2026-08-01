//! The formatter: unwrap hard wraps to one line per statement (V3), and
//! report any line over the cap (V5).
//!
//! Every function here is a pure transform over `&str`. That is deliberate
//! and load-bearing: a consumer embeds these rules by CALLING them, never
//! by re-porting them (V7), and a pure function is what makes that cheap.

/// The line cap.
///
/// Set from measured data, not taste (V5): ~12% above the longest statement
/// this repo's own spec produces after `fmt`. Real slack, so a legitimate
/// addition is not a raise -- a ceiling a hair above current turns every
/// edit into a threshold bump, and a threshold bumped reflexively trains
/// the reflex instead of catching the problem.
pub const MAX_LINE: usize = 1650;

/// Whether a line opens a statement by its own syntax: a header, a bullet,
/// a table row, or a section id (`V3:`, `T30a|`, `B12|`, `M1|`).
///
/// V28: read on the TRIMMED line. This tested the raw line until B12, so
/// anything INDENTED read as a continuation and was merged into the line
/// above -- two invariants became one line, three bullets became one, and
/// V1's proof passed every time because a merge is whitespace-only. The
/// corpus carries 112 genuine sub-bullets that adoption would have mangled
/// silently.
#[must_use]
pub fn carries_a_marker(line: &str) -> bool {
    let line = line.trim_start();
    if line.trim().is_empty() || line.starts_with(['#', '|']) {
        return true;
    }
    // The SPACE is required, as it always was: a wrapped line that happens
    // to begin with a dash is prose, not a bullet.
    if ["- ", "* ", "+ "].iter().any(|m| line.starts_with(m))
        || line == "id|date|cause|fix"
    {
        return true;
    }
    ordered_item(line) || id_prefixed(line)
}

/// `1. `, `2) ` -- an ORDERED list item opens a statement too.
///
/// B13: B12's fix taught this function three bullet characters and stopped,
/// so `1.` `2.` `3.` still read as continuations and a nine-item ordered
/// list collapsed into ONE line. Worse than the bullet case it replaced:
/// those items are not declared ids, so V28's id-set proof cannot see them
/// go either, and both proofs pass on a spec that lost eight statements.
///
/// The digits must be followed by `.` or `)` AND a space, so a wrapped line
/// beginning with a number is still prose.
fn ordered_item(line: &str) -> bool {
    let digits: String =
        line.chars().take_while(char::is_ascii_digit).collect();
    if digits.is_empty() {
        return false;
    }
    matches!(line.get(digits.len()..), Some(r) if r.starts_with(". ") || r.starts_with(") "))
}

/// Every id DECLARED in the text, as labels -- V28's unit.
///
/// V1 compares normalised whitespace, so it cannot see a MERGE. This can:
/// a statement that stopped existing is an id that stopped being declared.
fn declared_ids(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|l| Some(crate::id::at_line_start(l)?.label()))
        .collect()
}

/// Ids declared before a format and not after it.
///
/// SUPERSET, not equality: `fmt` may GAIN an id by dedenting one that was
/// hidden (T19), and that is a repair rather than a loss.
#[must_use]
pub fn lost_statements(before: &str, after: &str) -> Vec<String> {
    let kept = declared_ids(after);
    declared_ids(before)
        .into_iter()
        .filter(|id| !kept.contains(id))
        .collect()
}

/// Delegated to `id`, never re-derived here: the checker reads the same
/// grammar to find DECLARATIONS, and a formatter that disagreed with it about
/// what an id looks like would produce violations the checker then reported
/// (V7).
fn id_prefixed(line: &str) -> bool {
    crate::id::at_line_start(line).is_some()
}

/// Whether `cur` CONTINUES `prev` -- i.e. is a hard wrap.
///
/// The property is PAIRWISE, not per-line (V4), and that is the whole
/// subtlety:
/// a blank-separated paragraph and the text under a header carry no marker
/// and are still whole statements. A per-line predicate rejects them, which
/// is exactly how the first version of this rule failed on a spec's own
/// intro paragraph.
#[must_use]
pub fn continuation(prev: &str, cur: &str) -> bool {
    !prev.trim().is_empty()
        && !prev.starts_with('#')
        && !cur.trim().is_empty()
        && !carries_a_marker(cur)
}

/// Join every hard-wrapped run into one line per statement.
///
/// `split('\n')` rather than `lines()`: the latter discards a trailing
/// newline, so a formatter built on it would strip the final one and then
/// disagree with itself on the next run (V2).
#[must_use]
pub fn unwrap_wraps(text: &str) -> String {
    let mut out: Vec<String> = Vec::new();
    for line in text.split('\n') {
        match out.last_mut() {
            Some(prev) if continuation(prev, line) => join(prev, line),
            _ => out.push(folded(line)),
        }
    }
    out.join("\n")
}

/// T19: an indented DECLARATION dedented back to column zero.
///
/// Whitespace-only, so V1's existing proof already covers it -- which is why
/// this belongs in `fmt` rather than `migrate`, whose word-level proof is
/// weaker. Without it V28 stops the merge but the declaration stays
/// invisible, and `check` still reports it as cited-but-never-declared.
///
/// ONLY id-shaped lines. Promoting a genuine sub-point to top level would
/// change what the document says, and the corpus has 112 of those.
fn folded(line: &str) -> String {
    let bare = line.trim_start();
    if bare.len() == line.len() || !carries_an_id(bare) {
        return line.to_owned();
    }
    bare.to_owned()
}

/// Whether the trimmed line declares an id, bullet or not.
fn carries_an_id(bare: &str) -> bool {
    id_prefixed(bare)
        || ["- ", "* ", "+ "]
            .iter()
            .filter_map(|m| bare.strip_prefix(m))
            .any(id_prefixed)
}

fn join(prev: &mut String, line: &str) {
    *prev = format!("{} {}", prev.trim_end(), line.trim());
}

/// Lines over the cap, as `(line number, length)` -- 1-based, so the
/// diagnostic names a place a reader can go (V5).
#[must_use]
pub fn over_cap(text: &str, cap: usize) -> Vec<(usize, usize)> {
    text.split('\n')
        .enumerate()
        .map(|(i, l)| (i.saturating_add(1), l.chars().count()))
        .filter(|(_, n)| *n > cap)
        .collect()
}

/// All whitespace collapsed -- the canonical form two texts must share to
/// be the same document (V1).
#[must_use]
pub fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// V1: the proof, run BEFORE any write is allowed. If this is false the
/// formatter has changed content, and no diff review substitutes for it.
#[must_use]
pub fn is_lossless(before: &str, after: &str) -> bool {
    normalize(before) == normalize(after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hard_wrap_is_joined_into_one_statement() {
        let src = "V1: **rule.** Default to the form\nalready use, even so.\n";
        assert_eq!(
            unwrap_wraps(src),
            "V1: **rule.** Default to the form already use, even so.\n"
        );
    }

    #[test]
    fn separate_statements_stay_separate() {
        let src = "V1: one\nV2: two\n- a bullet\n| a | row |\n";
        assert_eq!(unwrap_wraps(src), src);
    }

    /// The case a per-line rule gets wrong: a paragraph carries no marker
    /// and is still a statement.
    #[test]
    fn a_blank_separated_paragraph_is_not_a_continuation() {
        assert!(!continuation("", "Self-contained spec."));
        assert!(!continuation("## \u{a7}G GOAL", "Estimate the cost."));
        assert!(continuation("V1: **rule.** Default", "already use"));
    }

    #[test]
    fn suffixed_and_plain_ids_both_open_a_statement() {
        for l in ["V3: x", "T30a|x|y|V1", "B12|d|c|V1", "M1|x", "|--|--|"] {
            assert!(carries_a_marker(l), "{l}");
        }
        assert!(!carries_a_marker("Vx: not a numbered id"));
        assert!(!carries_a_marker("already use, even when"));
    }

    /// B12, planted: an indented declaration was MERGED into the line above,
    /// so two invariants became one. The failure this crate exists to
    /// prevent, in the crate's own flagship verb.
    #[test]
    fn an_indented_declaration_is_not_swallowed_by_the_line_above() {
        let src = "- V1: first rule\n  - V2: an indented one\n";
        let out = unwrap_wraps(src);
        assert!(out.contains("V1: first rule\n"), "{out}");
        assert!(out.contains("V2: an indented one"), "{out}");
        assert_eq!(out.lines().count(), 2, "still two statements: {out}");
    }

    /// ...and the same for a genuine sub-point, which is not a declaration
    /// at all. Three bullets became one line before B12, and the corpus
    /// carries 112 of these.
    #[test]
    fn an_indented_sub_point_keeps_its_own_line() {
        let src = "- parent\n  - sub one\n  - sub two\n";
        assert_eq!(unwrap_wraps(src).lines().count(), 3);
    }

    /// T19: an indented DECLARATION is folded back to column zero, so it is
    /// visible to `check`. Whitespace-only, so V1 still holds.
    #[test]
    fn an_indented_declaration_is_folded_to_column_zero() {
        let out = unwrap_wraps("- V1: one\n  - V2: two\n");
        assert!(out.contains("\n- V2: two"), "dedented: {out}");
        assert!(is_lossless("- V1: one\n  - V2: two\n", &out));
    }

    /// ...but a SUB-POINT is never promoted: that would change what the
    /// document says, and nothing about it is a declaration.
    #[test]
    fn a_sub_point_is_never_promoted() {
        let src = "- parent\n  - sub one\n";
        assert!(
            unwrap_wraps(src).contains("\n  - sub one"),
            "still indented"
        );
    }

    /// V28's proof, which V1 cannot make: a merge is whitespace-only, so
    /// `is_lossless` PASSES on a text that lost a statement.
    ///
    /// Both declarations start visible here, deliberately. An INDENTED one
    /// was never declared in the first place, so no id-set proof could
    /// notice it going -- that case is fixed by the marker rule and the
    /// fold, not by this. Two guards, two different failures.
    #[test]
    fn the_statement_proof_catches_what_the_whitespace_proof_cannot() {
        let before = "V1: first\nV2: second\n";
        let merged = "V1: first V2: second\n";
        assert!(is_lossless(before, merged), "V1 is blind to this");
        assert_eq!(lost_statements(before, merged), ["V2"]);
    }

    /// ...and it does not fire on a repair. Folding GAINS a declaration,
    /// which is why the rule is a superset rather than an equality.
    #[test]
    fn gaining_a_declaration_is_not_a_loss() {
        let before = "- V1: first\n  - V2: second\n";
        let after = unwrap_wraps(before);
        assert_eq!(lost_statements(before, &after), Vec::<String>::new());
    }

    /// V2 still holds with folding in the loop: a second pass sees a line
    /// already at column zero and leaves it alone.
    #[test]
    fn folding_is_idempotent() {
        let once = unwrap_wraps("- V1: one\n  - V2: two\n  - sub\n");
        assert_eq!(unwrap_wraps(&once), once);
    }

    /// B13: an ORDERED list survives fmt. B12's fix taught the marker test
    /// three bullet characters and stopped, so `1.` `2.` `3.` still merged
    /// -- nine invariants into one line on a real spec, with V1's proof AND
    /// V28's id-set proof both passing, because ordered items are not
    /// declared ids.
    #[test]
    fn an_ordered_list_keeps_one_item_per_line() {
        let src = "1. first\n2. second\n3. third\n";
        assert_eq!(unwrap_wraps(src), src);
        assert_eq!(unwrap_wraps("1) a\n2) b\n"), "1) a\n2) b\n");
    }

    /// ...and a wrapped line that merely BEGINS with a number is still
    /// prose, so the marker cannot swallow ordinary continuations.
    #[test]
    fn a_number_without_a_list_marker_is_still_prose() {
        assert!(continuation("V1: costs", "1650 tokens at most"));
        assert!(!carries_a_marker("2026 was the year"));
        assert!(carries_a_marker("2. an item"));
    }

    /// V2: a formatter that keeps changing its mind cannot gate.
    #[test]
    fn formatting_is_idempotent() {
        let src = "# h\n\npara one\nwrapped on\n\n- b\n  cont\nV1: a\nb\n";
        let once = unwrap_wraps(src);
        assert_eq!(unwrap_wraps(&once), once);
    }

    /// V1: the transform is whitespace-only, so the normalized forms match.
    #[test]
    fn the_transform_is_provably_lossless() {
        let src = "V1: a rule\nwrapped here\n\n- bullet\n  continued\n";
        assert!(is_lossless(src, &unwrap_wraps(src)));
    }

    /// ...and the proof actually FAILS when content changes, or it would
    /// be a check that cannot reject anything.
    #[test]
    fn the_proof_rejects_a_planted_content_change() {
        assert!(!is_lossless("V1: a rule", "V1: a RULE"));
        assert!(!is_lossless("V1: a rule", "V1: a"));
    }

    #[test]
    fn a_trailing_newline_survives() {
        assert_eq!(unwrap_wraps("V1: a\n"), "V1: a\n");
        assert_eq!(unwrap_wraps("V1: a"), "V1: a");
    }

    #[test]
    fn over_cap_names_the_line_and_its_length() {
        let text = "short\n".to_owned() + &"x".repeat(20);
        assert_eq!(over_cap(&text, 10), vec![(2, 20)]);
        assert!(over_cap(&text, 100).is_empty());
    }
}
