//! V24: microlith's own `SPEC.md` is the first file it formats and caps.
//!
//! A rule this crate cannot pass is a rule it may not ship. These run
//! against the library being built, so the spec and the code cannot drift
//! apart without the gate noticing.

use microlith::parse_records;
use microlith::tasks_json;
use microlith::{MAX_LINE, over_cap};
use microlith::{check_spec, format_spec};

fn at_root(name: &str) -> String {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let text = std::fs::read_to_string(p).unwrap_or_default();
    assert!(!text.is_empty(), "{name} is missing or empty");
    text
}

fn spec() -> String {
    at_root("SPEC.md")
}

/// The spec is already formatted: `fmt` is a no-op on it.
#[test]
fn our_own_spec_is_formatted() {
    let text = spec();
    let out = format_spec(&text).unwrap_or_default();
    assert_eq!(out, text, "SPEC.md is not formatted -- run `mth fmt`");
}

/// ...and it passes its own structural rules, records included.
///
/// The rules this crate sells are the rules it is held to. A spec that could
/// not pass `check` would make every violation `check` reports arguable.
#[test]
fn our_own_spec_passes_check() {
    let records = parse_records(&at_root(".spec-records"));
    assert!(!records.is_empty(), ".spec-records parsed to nothing");
    let violations = check_spec(&spec(), &records);
    let named: Vec<String> =
        violations.iter().map(ToString::to_string).collect();
    assert_eq!(named, Vec::<String>::new());
}

/// B29's OTHER half: V42 catches the cause, this asserts the EFFECT.
///
/// A malformed row does not fail to parse — it parses into the wrong
/// fields, so `cites` quietly becomes a fragment of prose and the JSON
/// contract a consumer reads (M11) carries a sentence where it promised
/// ids. Six of our own rows did exactly that while `check` stayed green.
///
/// V42 now makes those rows red, so this can never fire while the rule
/// holds. That is precisely why it is worth having: it pins the PROPERTY
/// the rule exists to protect, so a future loosening of V42 has to fail
/// here too rather than quietly restoring the defect.
///
/// Scanned rather than parsed, because a JSON dependency in a crate with
/// none would cost more than this test is worth (§C).
#[test]
fn every_cites_field_in_the_json_holds_ids() {
    let json = tasks_json("SPEC.md", &spec());
    let bad: Vec<&str> = json
        .split("\"cites\":[")
        .skip(1)
        .filter_map(|rest| rest.split(']').next())
        .flat_map(|list| list.split(','))
        .map(|cite| cite.trim().trim_matches('"'))
        .filter(|cite| !cite.is_empty() && *cite != "-")
        .filter(|cite| !is_id(cite))
        .collect();
    assert_eq!(bad, Vec::<&str>::new(), "cites that are not ids");
}

/// `V13`, `T7a`, `B2` -- a letter, a number, an optional suffix.
fn is_id(cite: &str) -> bool {
    let mut chars = cite.chars();
    let kind = chars.next().is_some_and(|c| "VTBRIGCFN".contains(c));
    kind && chars.next().is_some_and(|c| c.is_ascii_digit())
}

/// V5/V9: and no line is over the cap.
#[test]
fn our_own_spec_is_under_the_cap() {
    let over = over_cap(&spec(), MAX_LINE);
    assert!(over.is_empty(), "lines over the {MAX_LINE} cap: {over:?}");
}

/// V9's evidence, kept honest: the cap is set ABOVE the measured maximum
/// with real slack, so a legitimate addition is not a raise. If this fails
/// the spec has grown into its ceiling and the number needs a REVIEW, not
/// a reflex bump.
/// CHARS, matching `over_cap`, which is what actually enforces the cap.
/// This measured BYTES until it fired on a line that was comfortably under
/// the limit: 1478 chars but 1514 bytes, because the format is dense with
/// `§`, `∴` and `⊥` at three bytes each. A guard must measure in the same
/// unit as the rule it guards, or it reports a breach that does not exist.
#[test]
fn the_cap_still_carries_real_slack() {
    let longest = spec()
        .split('\n')
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    let slack = MAX_LINE.saturating_sub(longest);
    assert!(
        slack.saturating_mul(10) > MAX_LINE,
        "longest line is {longest} of {MAX_LINE} -- under 10% slack, which \
         is the ceiling-a-hair-above-current failure (V5)"
    );
}
