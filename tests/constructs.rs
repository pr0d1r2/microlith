//! V29: every construct FORMAT.md permits has a FIXTURE, and the rules are
//! verified against all of them -- not against our own spec alone.
//!
//! WHY THIS EXISTS, and it is not a code lesson. B12 (indented list items
//! merged), B13 (ordered lists merged) and B14 (fenced blocks merged) are one
//! root cause: our own `SPEC.md` contains none of those three constructs, so
//! V24's dogfood was structurally UNABLE to catch any of them. It proves rules
//! against ONE FILE. The corpus caught all three; the dogfood caught zero.
//!
//! The corpus cannot ship -- it is private, and §C forbids naming those repos
//! -- so the coverage it provided is reconstructed here as fixtures. These
//! travel with the crate, so a consumer re-runs them against their own build
//! instead of trusting that we ran something they cannot see.
//!
//! THE GUARD IS BIDIRECTIONAL. A name in `CONSTRUCTS` with no file fails, and
//! a file not in `CONSTRUCTS` fails. That is what makes "a construct with no
//! fixture fails" true rather than aspirational: adding a construct to the
//! list forces a fixture, and adding a fixture forces it to be named.

use nanokit::check::parse_records;
use nanokit::{check_spec, format_spec};

/// Every construct FORMAT.md permits, each with a fixture of the same name.
///
/// The first seven are the format's own sections. The rest are markdown
/// shapes the format allows inside them -- and the three marked below are the
/// ones that silently merged until the corpus said so.
const CONSTRUCTS: [&str; 14] = [
    "goal-prose",
    "constraint-bullets",
    "interface-bullets",
    "research-table",
    "invariant-statements",
    "task-table",
    "bug-table",
    // B12: indented list items were merged into the line above.
    "nested-sub-points",
    // B13: ordered list items were merged into one line.
    "ordered-list",
    // B14: a fenced block became a single line.
    "fenced-block",
    // V26: a declaration behind a markdown bullet.
    "bulleted-declarations",
    // V11 tolerates an unknown letter.
    "extension-section",
    // V14: a suffixed id rides its base.
    "suffixed-ids",
    // V13: backticks are verbatim, so a literal is not a citation.
    "inline-code-span",
];

fn dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/constructs")
}

fn fixture(name: &str) -> String {
    let path = dir().join(format!("{name}.md"));
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(!text.is_empty(), "fixture {name} is missing or empty");
    text
}

/// Every named construct has a file. A name without one is a construct
/// nothing verifies.
#[test]
fn every_construct_has_a_fixture() {
    let missing: Vec<&str> = CONSTRUCTS
        .into_iter()
        .filter(|n| !dir().join(format!("{n}.md")).exists())
        .collect();
    assert_eq!(missing, Vec::<&str>::new(), "constructs with no fixture");
}

/// ...and every file is named. An orphan fixture is one nobody decided to
/// keep, which is how a corpus rots into a directory of old attempts.
#[test]
fn every_fixture_is_a_named_construct() {
    let entries = std::fs::read_dir(dir()).ok();
    let orphans: Vec<String> = entries
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|f| f.ends_with(".md"))
        .filter(|f| {
            let stem = f.trim_end_matches(".md");
            !CONSTRUCTS.contains(&stem)
        })
        .collect();
    assert_eq!(orphans, Vec::<String>::new(), "fixtures not in CONSTRUCTS");
}

/// THE REGRESSION GUARD. Every fixture is already canonical, so `fmt` must
/// change nothing. Before B12, B13 and B14 this failed on three of them --
/// sub-points, ordered lists and fenced blocks each collapsed into the line
/// above, and V1's proof passed every time because a merge is whitespace-only.
#[test]
fn fmt_is_a_no_op_on_every_construct() {
    for name in CONSTRUCTS {
        let text = fixture(name);
        let out = format_spec(&text).unwrap_or_default();
        assert_eq!(out, text, "fmt changed the {name} fixture");
    }
}

/// V2 at corpus scale: a formatter that changes its mind on any construct
/// cannot be a `--check` gate for that construct.
#[test]
fn fmt_is_idempotent_on_every_construct() {
    for name in CONSTRUCTS {
        let once = format_spec(&fixture(name)).unwrap_or_default();
        let twice = format_spec(&once).unwrap_or_default();
        assert_eq!(twice, once, "fmt is not idempotent on {name}");
    }
}

/// Each fixture is a WELL-FORMED spec, so `check` must be silent. This is the
/// other half: `fmt` leaving a file alone proves nothing if the checker
/// rejects it, and a rule that fires on a legal construct is B4's defect.
#[test]
fn check_is_clean_on_every_construct() {
    let none = parse_records("");
    for name in CONSTRUCTS {
        let found = check_spec(&fixture(name), &none);
        let named: Vec<String> =
            found.iter().map(ToString::to_string).collect();
        assert_eq!(named, Vec::<String>::new(), "check fired on {name}");
    }
}

/// The three merges, planted directly (V18) rather than inferred from the
/// no-op above -- so the guard names what it is guarding, and a future reader
/// sees the failure rather than a passing assertion.
#[test]
fn the_three_recorded_merges_are_planted() {
    for (name, marker) in [
        ("nested-sub-points", "  - a genuine sub-point"),
        ("ordered-list", "2. the second step."),
        ("fenced-block", "nanokit fmt"),
    ] {
        let text = fixture(name);
        assert!(text.contains(marker), "{name} lost its planted shape");
        let out = format_spec(&text).unwrap_or_default();
        let before = text.lines().count();
        assert_eq!(out.lines().count(), before, "{name} lost lines to a merge");
        assert!(out.contains(marker), "{name}: `{marker}` was merged away");
    }
}
