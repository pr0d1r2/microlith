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
#[must_use]
pub fn carries_a_marker(line: &str) -> bool {
    if line.trim().is_empty() || line.starts_with(['#', '|']) {
        return true;
    }
    if line.starts_with("- ") || line == "id|date|cause|fix" {
        return true;
    }
    id_prefixed(line)
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
            _ => out.push(line.to_owned()),
        }
    }
    out.join("\n")
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
