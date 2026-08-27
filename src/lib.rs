//! `microlith` -- the cavekit SPEC format, enforced.
//!
//! One implementation of the format rules, callable as a library, so a
//! consumer embeds them instead of re-porting them (V7). That is the whole
//! reason this crate exists: two hand-maintained copies of one rule set
//! disagreed, both gates stayed green, and 88 tasks belonged to no
//! milestone.
//!
//! CPU only -- no inference, no network (V6). Every operation is a
//! deterministic function of the text it is given.

// PUB(CRATE), NOT PUB (V32). These are the parser and the rules -- the
// plumbing the verbs are built from. Exporting them would promise 54 items
// nobody outside asked for, and un-exporting after 1.0 is the breaking
// change nobody plans for. What a consumer gets instead is one function per
// verb, below: the same operations the CLI offers, and nothing else.
//
// Widening later is a MINOR release. Narrowing later is a MAJOR one. That
// asymmetry is the whole reason to start small.
pub(crate) mod anchors;
pub(crate) mod check;
pub(crate) mod cli;
pub(crate) mod derive;
pub(crate) mod docs;
pub(crate) mod extensions;
pub(crate) mod format;
pub(crate) mod id;
pub(crate) mod migrate;
pub(crate) mod render;
pub(crate) mod tasks;
pub(crate) mod violation;

// The types a public signature mentions must themselves be public.
pub use check::Record;
pub use cli::{DEFAULT_PATH, run};
pub use violation::{Direction, Fix, Violation};

// ONE ENTRY POINT PER VERB. `run` is the whole CLI; these are its parts, so
// a caller can reach any single operation without parsing an argv or reading
// an exit code. Between them they are everything `mth` can do -- which is the
// contract V32 freezes at 1.0, and the reason the modules above are not it.
pub use check::parse_records;
pub use format::{MAX_LINE, over_cap};

/// `mth migrate`: section headers rewritten to canonical cavekit 4.1.0.
///
/// `Err` carries the reason a rewrite was declined -- a letter used for a
/// DIFFERENT concept is reported rather than rewritten, because annotating
/// one keeps the characters and inverts the meaning.
pub fn migrate_spec(text: &str) -> Result<String, String> {
    migrate::migrate(text)
}

/// Every header that is not canonical, whether or not it can be fixed.
///
/// The question `migrate_spec` does not answer: what WOULD change. Kept
/// separate from [`migrate_declined`] because "not canonical" and "cannot
/// be repaired" are different sets, and a caller usually wants the first.
pub fn migrate_report(text: &str) -> String {
    migrate::report(text)
}

/// The collisions `migrate_spec` refused to touch.
///
/// A letter used for a DIFFERENT concept is never rewritten -- annotating
/// one keeps the characters and inverts the meaning -- so a run that
/// rewrote everything it could may still leave the file non-canonical.
pub fn migrate_declined(text: &str) -> String {
    migrate::unfinished(text)
}

/// `mth derive`: statement sizes, the citation graph, orphans, duplication.
///
/// Report-only. An orphan is a question for a reader, not a build failure,
/// so nothing here signals an error.
pub fn derive_report(text: &str) -> String {
    derive::report(text)
}

/// `derive_report` with every statement's size, biggest first.
pub fn derive_report_verbose(text: &str) -> String {
    derive::report_verbose(text)
}

/// `mth tasks`: every `§T` row's id, status, text and cites, in V14 order.
///
/// ENUMERATE, never SELECT: which rows are pending is mechanical, which one
/// to work next is judgement and stays with the caller (V6).
pub fn tasks_report(text: &str) -> String {
    tasks::report(text, false)
}

/// `tasks_report` with each task's full text rather than a 60-char gist.
pub fn tasks_report_verbose(text: &str) -> String {
    tasks::report(text, true)
}

/// The same enumeration as JSON -- the rendering a consumer parses.
///
/// `file` is echoed back in the object, so a caller running this over several
/// specs can tell the answers apart. Always emitted, including `"tasks":[]`
/// for a spec with no `§T`: an empty stream would be indistinguishable from a
/// run that never reached the file.
pub fn tasks_json(file: &str, text: &str) -> String {
    tasks::json(file, text)
}

/// `mth anchors`: the section address of every item beside the id it
/// currently resolves to, and whether the two have drifted apart.
pub fn anchors_report(text: &str) -> String {
    anchors::report(text)
}

/// `anchors_report` with each item in full rather than a 60-char gist.
pub fn anchors_report_verbose(text: &str) -> String {
    anchors::report_verbose(text)
}

/// The command reference `mth docs` prints, as markdown.
pub fn docs_markdown() -> String {
    docs::markdown()
}

/// `mth extensions`: the letters this build adds to the vendored format.
///
/// The document a cavekit user adopts WITHOUT this tool (V40) -- the full
/// section order marking whose each letter is, each extension's canonical
/// word, what it holds, an example. `FORMAT-EXTENSIONS.md` is this string,
/// and a test fails if the two drift.
pub fn format_extensions() -> String {
    extensions::markdown()
}

/// What a run produced: streams and an exit code. No I/O, so the whole
/// command surface is testable without spawning the binary.
#[derive(Debug, PartialEq, Eq)]
pub struct Output {
    pub out: String,
    pub err: String,
    pub code: u8,
}

impl Output {
    #[must_use]
    pub fn ok(out: String) -> Self {
        Self {
            out,
            err: String::new(),
            code: 0,
        }
    }

    /// A violation the caller asked us to gate on: exit 1 (V10).
    #[must_use]
    pub fn drift(err: String) -> Self {
        Self {
            out: String::new(),
            err,
            code: 1,
        }
    }

    /// A usage error: exit 2.
    #[must_use]
    pub fn usage(err: String) -> Self {
        Self {
            out: String::new(),
            err,
            code: 2,
        }
    }
}

/// The formatted text, or the reason it must not be written.
///
/// V1's proof runs HERE, before any caller can persist the result -- so a
/// lossy transform is impossible to write out by mistake rather than
/// merely discouraged.
///
/// # Errors
/// When the transform would change content, or a line exceeds the cap.
pub fn format_spec(text: &str) -> Result<String, String> {
    let out = format::unwrap_wraps(text);
    if !format::is_lossless(text, &out) {
        return Err("refusing to write: the transform changed content, \
                    not just whitespace"
            .to_owned());
    }
    // V28, and NOT implied by the line above: a MERGE is whitespace-only, so
    // the losslessness proof passes while a statement stops existing (B12).
    let lost = format::lost_statements(text, &out);
    if !lost.is_empty() {
        return Err(format!(
            "refusing to write: {lost:?} would stop being declared"
        ));
    }
    within_cap(&out)?;
    Ok(out)
}

/// Every structural violation in `text`, in rule order.
///
/// V11-V15 are computable from the text alone. V16 needs a baseline of named
/// records, which is a claim about edits over time rather than about this
/// file, so the caller supplies it -- empty means the V16 gate is off, not
/// that it passed.
///
/// This is the example README.md shows, so the gate compiles it and the
/// README cannot drift from the API it advertises:
///
/// ```
/// let text = "V1: **a rule.** cited by T1.\n";
/// let records = Vec::new();
/// let violations = microlith::check_spec(text, &records);
/// for v in &violations {
///     println!("{}: {} ({})", v.rule, v.msg, v.line);
/// }
/// // A rule with no §V header above it, so V11 has something to say. An
/// // ABSENT section is legal since T15; an item STRANDED by one is not.
/// assert!(violations.iter().any(|v| v.rule == "V11"));
/// ```
#[must_use]
pub fn check_spec(
    text: &str,
    records: &[check::Record],
) -> Vec<violation::Violation> {
    let mut out = check::sections_ordered(text);
    out.extend(check::labels_canonical(text));
    out.extend(check::ids_unique(text));
    out.extend(check::citations_resolve(text));
    out.extend(check::rows_sorted(text));
    out.extend(check::tasks_in_one_milestone(text));
    out.extend(check::statuses_valid(text));
    out.extend(check::supersessions_resolve(text));
    out.extend(check::records_survive(text, records));
    out
}

/// V5: an over-long line names its place, so the reader can go there.
fn within_cap(text: &str) -> Result<(), String> {
    match format::over_cap(text, format::MAX_LINE).first() {
        Some((line, len)) => Err(format!(
            "line {line} is {len} chars, over the {} cap -- split the \
             statement; raising the cap is a reviewed decision",
            format::MAX_LINE
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_returns_the_unwrapped_text() {
        let got = format_spec("V1: a rule\nwrapped\n").unwrap_or_default();
        assert_eq!(got, "V1: a rule wrapped\n");
    }

    /// V5: an over-long line is refused with its number and its length,
    /// rather than written out and discovered later.
    #[test]
    fn an_over_long_line_is_refused_with_its_place() {
        let long = format!("V1: {}\n", "x".repeat(format::MAX_LINE));
        let err = format_spec(&long).err().unwrap_or_default();
        assert!(err.contains("line 1"), "names the line: {err}");
        assert!(err.contains("over the"), "names the rule: {err}");
    }

    #[test]
    fn already_formatted_text_is_unchanged() {
        let src = "# h\n\nV1: a rule\nV2: another\n";
        assert_eq!(format_spec(src).ok(), Some(src.to_owned()));
    }

    /// V32's surface is the CONTRACT, so every verb-level entry point is
    /// exercised here. Untested public API is a promise nobody checked --
    /// and the coverage floor caught these eight the moment they existed.
    const CANON: &str = "## \u{a7}V INVARIANTS\nV1: a rule.\n\n\
        ## \u{a7}T TASKS\n| id | scope | tasks | done-when |\n\
        |----|-------|-------|-----------|\n| M1 | core | T1 | done |\n\
        T1|x|a task|V1\n";

    #[test]
    fn the_writing_verbs_are_reachable_in_one_call() {
        assert!(format_spec(CANON).is_ok());
        assert!(migrate_spec(CANON).is_ok());
    }

    #[test]
    fn the_reporting_verbs_are_reachable_in_one_call() {
        assert!(check_spec(CANON, &[]).is_empty());
        assert_eq!(migrate_report(CANON), "");
        assert_eq!(migrate_declined(CANON), "");
    }

    #[test]
    fn the_derived_reports_are_reachable_in_one_call() {
        assert!(!derive_report(CANON).is_empty());
        assert!(!anchors_report(CANON).is_empty());
        assert!(docs_markdown().contains("## Commands"));
    }

    /// V40's document is reachable the same way, and it is the one entry
    /// point here that answers about the FORMAT rather than about a file.
    #[test]
    fn the_format_extensions_are_reachable_in_one_call() {
        let out = format_extensions();
        assert!(out.contains("## SECTION ORDER"), "{out}");
        assert!(out.contains("\u{a7}F FEDERATION"), "{out}");
    }

    /// The enumeration is reachable in one call, in both renderings -- and
    /// the JSON one takes the file name, because a caller running it over
    /// several specs has to tell the answers apart.
    #[test]
    fn the_task_enumeration_is_reachable_in_one_call() {
        assert!(tasks_report(CANON).contains("task T1: x"));
        assert!(tasks_json("SPEC.md", CANON).contains("\"id\":\"T1\""));
        assert!(tasks_json("SPEC.md", CANON).contains("\"file\":\"SPEC.md\""));
    }

    #[test]
    fn verbose_reports_say_more_than_their_terse_form() {
        assert!(
            derive_report_verbose(CANON).len() >= derive_report(CANON).len()
        );
        assert!(
            anchors_report_verbose(CANON).len() >= anchors_report(CANON).len()
        );
        assert!(tasks_report_verbose(CANON).len() >= tasks_report(CANON).len());
    }

    /// `run` IS the CLI: same argv, same streams, same exit code, no process.
    #[test]
    fn run_is_the_whole_command_without_a_process() {
        assert_eq!(run(&[]).code, 2, "no verb is a usage error");
        assert_eq!(run(&["--version".into()]).code, 0);
        assert!(run(&["docs".into()]).out.contains("## Commands"));
        assert_eq!(run(&["nonesuch".into()]).code, 2);
    }

    /// The re-exports a caller needs to USE the surface above.
    #[test]
    fn the_reexported_helpers_work() {
        assert!(parse_records("# comment\nV1 marker text\n").len() == 1);
        assert!(over_cap(CANON, MAX_LINE).is_empty());
        assert_eq!(DEFAULT_PATH, "SPEC.md");
    }
}
