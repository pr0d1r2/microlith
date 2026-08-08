//! Argument parsing and verb dispatch for the `mth` command.
//!
//! This lives in the LIBRARY, not the binary, so that `run` is one public
//! function a consumer can call to get exactly what the CLI does -- and so
//! the modules it drives can stay `pub(crate)`. `src/main.rs` is the shell:
//! it collects argv, calls `run`, and turns the result into an exit code.

use crate::check::{Record, parse_records};
use crate::render;
use crate::violation::Violation;
use crate::{Output, check_spec, format_spec};

/// Everything the `mth` command does, as one call.
///
/// `args` is argv WITHOUT the program name. The returned [`Output`] carries
/// stdout, stderr and the exit code the CLI would have used, so a caller
/// gets the same verdict without spawning a process.
pub fn run(args: &[String]) -> Output {
    match args.split_first() {
        None => Output::usage(usage()),
        Some((verb, rest)) => dispatch(verb, rest),
    }
}

fn dispatch(verb: &str, rest: &[String]) -> Output {
    match verb {
        "--help" | "-h" => Output::ok(usage()),
        "--version" | "-V" => version(),
        "fmt" => fmt(rest),
        "check" => check(rest),
        "derive" => reporting(rest, derive_report(rest)),
        "anchors" => reporting(rest, anchors_report(rest)),
        "tasks" => tasks(rest),
        "migrate" => migrate(rest),
        "docs" => Output::ok(crate::docs::markdown()),
        other => unknown(other),
    }
}

fn version() -> Output {
    Output::ok(format!("microlith {}\n", env!("CARGO_PKG_VERSION")))
}

fn unknown(verb: &str) -> Output {
    Output::usage(format!("mth: unknown command '{verb}'\n{}", usage()))
}

/// Which `anchors` rendering. Verbose prints each item's full text rather
/// than a 60-char gist -- the same deepen-not-repeat rule (§I).
fn anchors_report(rest: &[String]) -> fn(&str) -> String {
    if verbose(rest) {
        crate::anchors::report_verbose
    } else {
        crate::anchors::report
    }
}

/// Which `derive` rendering. Verbose DEEPENS here rather than confirming:
/// the plain report is already the answer to "is it big", and the verbose
/// one answers "what do I cut" (§I).
fn derive_report(rest: &[String]) -> fn(&str) -> String {
    if verbose(rest) {
        crate::derive::report_verbose
    } else {
        crate::derive::report
    }
}

/// The usage text, rendered from the ONE command registry (V33).
///
/// It used to be a raw string laid out exactly as it printed, which read
/// well and rotted anyway: the README carried a second copy by hand, and
/// both drifted -- `migrate` was missing from one and `derive`'s
/// duplication report from the other. A registry makes a new verb a
/// one-place edit and `docs.rs`'s freeze makes staleness red.
fn usage() -> String {
    crate::docs::usage()
}

/// `fmt [--check] <path>`. The check mode never writes, so it is safe in a
/// gate; the default mode writes only after the losslessness proof (V1).
fn fmt(rest: &[String]) -> Output {
    let check = rest.iter().any(|a| a == "--check");
    let path = target(rest);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return unreadable(&path);
    };
    match format_spec(&text) {
        Err(e) => Output::drift(format!("mth: {path}: {e}\n")),
        Ok(out) => {
            let done =
                apply(&path, (&text, &out), check, ("fmt", "not formatted"));
            said(done, verbose(rest), || fmt_summary(&path, &out))
        }
    }
}

/// `migrate`: headers to canonical 4.1.0, and a report of what it declined.
///
/// Writes like `fmt` and gates like `fmt --check`, but with one difference
/// that matters: a COLLISION cannot be migrated, so a run that rewrote
/// everything it could may still leave the file non-canonical. That is
/// reported and exits 1, because silence here would claim a finished job.
fn migrate(rest: &[String]) -> Output {
    let check = rest.iter().any(|a| a == "--check");
    let path = target(rest);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return unreadable(&path);
    };
    match crate::migrate::migrate(&text) {
        Err(e) => Output::drift(format!("mth: {path}: {e}\n")),
        Ok(out) => migrated(rest, &path, (&text, &out), check),
    }
}

fn migrated(
    rest: &[String],
    path: &str,
    io: (&str, &str),
    check: bool,
) -> Output {
    let done = apply(path, io, check, ("migrate", "not canonical"));
    let left = crate::migrate::unfinished(io.1);
    if done.code == 0 && !left.is_empty() {
        return Output::drift(format!("{}{left}", done.out));
    }
    said(done, verbose(rest), || {
        format!("mth: {path}: headers canonical\n")
    })
}

/// Append what was examined, but only on success and only when asked. A
/// summary next to a failure would bury the failure.
fn said(o: Output, verbose: bool, summary: impl Fn() -> String) -> Output {
    if o.code != 0 || !verbose {
        return o;
    }
    Output::ok(format!("{}{}", o.out, summary()))
}

/// What `fmt` examined, for a caller who asked to be told.
///
/// The cap and the longest line, because those are the two numbers that
/// decide whether the next addition will be refused -- knowing you are at
/// 763 of 1650 is the difference between "it passed" and "it passed with
/// room".
fn fmt_summary(path: &str, text: &str) -> String {
    let lines = text.lines().count();
    let longest = text.lines().map(|l| l.chars().count()).max().unwrap_or(0);
    format!(
        "mth: {path}: formatted, {lines} lines, longest {longest} of {} \
         ({} spare)\n",
        crate::format::MAX_LINE,
        crate::format::MAX_LINE.saturating_sub(longest)
    )
}

/// `check [--records <file>] <path>`. Report-only by construction: it reads
/// two files and writes none, so it is safe anywhere a gate runs (V10).
fn check(rest: &[String]) -> Output {
    let path = target(rest);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return unreadable(&path);
    };
    let how = match wants_json(rest) {
        Err(e) => return Output::usage(e),
        Ok(as_json) => Opts {
            as_json,
            verbose: verbose(rest),
        },
    };
    match records_from(rest) {
        Err(e) => Output::usage(e),
        Ok(r) => checked(&path, &text, &r, how),
    }
}

/// Run the rules and render, with the summary only when it was asked for
/// and only in the human rendering -- appending prose to JSON would break
/// the one thing the JSON is for.
fn checked(path: &str, text: &str, records: &[Record], how: Opts) -> Output {
    let found = check_spec(text, records);
    let done = report(path, &found, how.as_json);
    said(done, how.verbose && !how.as_json, || {
        check_summary(path, text, records.len())
    })
}

/// How the caller asked for the output.
///
/// A struct rather than two bool parameters, because `checked(.., true,
/// false)` at the call site says nothing about which is which -- and the
/// lint that forced this was right to.
#[derive(Debug, Clone, Copy)]
struct Opts {
    as_json: bool,
    verbose: bool,
}

/// What `check` examined, for a caller who asked to be told.
///
/// Counts the ITEMS, not the rules: "6 rules ran" is a fact about microlith,
/// while "24 invariants, 9 tasks, 4 bugs" is a fact about the spec, and
/// only the second changes when someone points the gate at the wrong file.
///
/// It also says whether V16 ran, because that rule is OFF unless a baseline
/// was supplied -- and a gate silently checking five rules instead of six
/// looks exactly like a gate checking six.
fn check_summary(path: &str, text: &str, records: usize) -> String {
    let n = |kind| crate::check::declared(text, kind).len();
    let records = if records == 0 {
        "no records baseline, so V16 did not run".to_owned()
    } else {
        format!("{records} records checked")
    };
    format!(
        "mth: {path}: clean -- {} invariants, {} tasks, {} bugs; {}\n",
        n('V'),
        n('T'),
        n('B'),
        records
    )
}

/// The V16 baseline, or none. A `--records` path that cannot be read is a
/// USAGE error, never a silent pass: a gate that quietly stops checking
/// because a file moved is the failure mode the flag exists to prevent.
fn records_from(rest: &[String]) -> Result<Vec<Record>, String> {
    let Some(path) = flag_value(rest, "--records") else {
        return Ok(Vec::new());
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => Ok(parse_records(&text)),
        Err(e) => Err(format!("mth: cannot read {path}: {e}\n")),
    }
}

/// Report the violations in the requested rendering.
///
/// Both renderings come from the lib, so the binary chooses a format and
/// never owns one -- a consumer calling `render::json` gets byte-identical
/// output to `mth check --format json` (V7).
fn report(path: &str, violations: &[Violation], as_json: bool) -> Output {
    if violations.is_empty() {
        return Output::ok(String::new());
    }
    let out = if as_json {
        render::json(path, violations)
    } else {
        render::human(path, violations)
    };
    Output::drift(out)
}

/// `--format human|json`, defaulting to human. An unknown format is a usage
/// error rather than a silent fallback: a caller who asked for json and got
/// prose would parse it and fail somewhere far away.
fn wants_json(rest: &[String]) -> Result<bool, String> {
    match flag_value(rest, "--format").as_deref() {
        None | Some("human") => Ok(false),
        Some("json") => Ok(true),
        Some(other) => Err(format!(
            "mth: unknown --format '{other}' -- expected human or json\n"
        )),
    }
}

/// The value after `name`, if the flag is present with one.
fn flag_value(rest: &[String], name: &str) -> Option<String> {
    let at = rest.iter().position(|a| a == name)?;
    rest.get(at.saturating_add(1))
        .filter(|v| !v.starts_with('-'))
        .cloned()
}

/// The file every cavekit command reads when told nothing else.
///
/// FORMAT.md: "Single file. Project root." The convention already existed;
/// this is the CLI honouring it rather than making each caller retype it.
pub const DEFAULT_PATH: &str = "SPEC.md";

/// Whether the caller opted out of silence-is-success (V10).
fn verbose(rest: &[String]) -> bool {
    rest.iter().any(|a| a == "--verbose" || a == "-v")
}

/// The first argument that is neither a flag nor a flag's value.
fn positional(rest: &[String]) -> Option<&String> {
    let skip: Vec<String> = ["--records", "--format"]
        .iter()
        .filter_map(|f| flag_value(rest, f))
        .collect();
    rest.iter()
        .find(|a| !a.starts_with('-') && !skip.contains(a))
}

/// The path to work on: what was asked for, or the convention.
///
/// Defaulting the PATH does not make a rewrite less deliberate (V10) --
/// `fmt` is still a verb someone typed, and the transform is still proven
/// lossless before any write. What it removes is retyping the one filename
/// the format says there will be.
fn target(rest: &[String]) -> String {
    positional(rest)
        .cloned()
        .unwrap_or_else(|| DEFAULT_PATH.to_owned())
}

/// A path that cannot be read, said in a way that distinguishes the two
/// reasons: a wrong path, or the right path in the wrong directory.
fn unreadable(path: &str) -> Output {
    if path == DEFAULT_PATH {
        return Output::usage(format!(
            "mth: no {DEFAULT_PATH} here -- run from a project root, \
             or name a path\n"
        ));
    }
    Output::usage(format!("mth: cannot read {path}\n"))
}

/// `tasks [--format human|json] <path>`: enumerate `§T`, exit 0.
///
/// Report-only like `derive`, but with a rendering to choose, so it cannot
/// share `reporting`'s one-function shape. Exit 0 EVEN WITH NO TASKS: a spec
/// with an empty backlog and a spec with none are both legal, and the payload
/// is what tells them apart (`"tasks":[]`). An unreadable path stays a USAGE
/// error at exit 2 -- that is a broken invocation rather than a finding, and
/// it is the signal a caller falls back on when this verb is absent
/// altogether.
fn tasks(rest: &[String]) -> Output {
    let path = target(rest);
    let as_json = match wants_json(rest) {
        Err(e) => return Output::usage(e),
        Ok(j) => j,
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return unreadable(&path);
    };
    Output::ok(if as_json {
        crate::tasks::json(&path, &text)
    } else {
        crate::tasks::report(&text, verbose(rest))
    })
}

/// `derive <path>` and `anchors <path>`: read, report, exit 0.
///
/// One function for both because they differ only in which report they
/// print. Exit 0 EVEN WITH FINDINGS is the whole distinction from `check`:
/// V10 names the gates, and these are not among them, so an orphan or a
/// shifted address goes to stdout for a reader to judge. A path that cannot
/// be read is still a usage error -- that is a broken invocation, not a
/// finding.
fn reporting(rest: &[String], report: fn(&str) -> String) -> Output {
    let path = target(rest);
    match std::fs::read_to_string(&path) {
        Err(_) => unreadable(&path),
        Ok(text) => Output::ok(report(&text)),
    }
}

/// Report the drift, or write the rewritten text.
///
/// Shared by `fmt` and `migrate` (V7): both prove a transform, then either
/// write it or report that it is needed. `verb` carries the command name and
/// the phrase for what the file currently is, so the two say different
/// things without a second copy of the write-or-report logic.
fn apply(
    path: &str,
    io: (&str, &str),
    check: bool,
    verb: (&str, &str),
) -> Output {
    let ((text, out), (verb, drift)) = (io, verb);
    if out == text {
        return Output::ok(String::new());
    }
    if check {
        return Output::drift(format!(
            "mth: {path} is {drift} -- run `mth {verb} {path}`\n"
        ));
    }
    match std::fs::write(path, out) {
        Ok(()) => Output::ok(format!("mth: {verb} rewrote {path}\n")),
        Err(e) => Output::usage(format!("mth: cannot write {path}: {e}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_PATH;

    fn args(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn no_command_is_a_usage_error() {
        assert_eq!(run(&[]).code, 2);
    }

    #[test]
    fn help_and_version_succeed() {
        assert_eq!(run(&args(&["--help"])).code, 0);
        assert!(run(&args(&["--version"])).out.contains("microlith"));
    }

    /// Every verb §I names is now built, so there is no unbuilt-verb arm
    /// left to test. What remains is that an unknown one still says so and
    /// prints the usage rather than failing silently.
    #[test]
    fn an_unknown_verb_says_so_and_shows_the_usage() {
        let o = run(&args(&["ancors", "SPEC.md"]));
        assert_eq!(o.code, 2);
        assert!(o.err.contains("unknown command 'ancors'"), "{}", o.err);
        assert!(o.err.contains("commands:"), "{}", o.err);
    }

    fn write_temp(name: &str, body: &str) -> String {
        let p = std::env::temp_dir()
            .join(format!("microlith-{name}-{}.md", std::process::id()));
        let _ = std::fs::write(&p, body);
        p.to_string_lossy().into_owned()
    }

    /// `check` gates: a violation exits 1 and NAMES the rule, so the reader
    /// goes to the invariant rather than guessing which one fired.
    #[test]
    fn check_names_the_rule_it_failed() {
        let path = write_temp("bad", "# spec\n\nV1: a rule with no header.\n");
        let o = run(&args(&["check", &path]));
        assert_eq!(o.code, 1, "{}", o.err);
        assert!(o.err.contains("V11"), "{}", o.err);
        let _ = std::fs::remove_file(&path);
    }

    /// A `--records` file that cannot be read is a USAGE error, never a
    /// silent pass. A gate that quietly stops checking because a path moved
    /// is exactly what the flag exists to prevent.
    #[test]
    fn an_unreadable_records_file_is_a_usage_error() {
        let path = write_temp("norec", "# spec\n");
        let o = run(&args(&["check", "--records", "no/such/file", &path]));
        assert_eq!(o.code, 2, "{}", o.err);
        assert!(o.err.contains("cannot read"), "{}", o.err);
        let _ = std::fs::remove_file(&path);
    }

    /// The path must survive a flag sitting in front of it, and must not be
    /// confused with the flag's own value.
    #[test]
    fn the_path_is_found_past_a_flag_and_its_value() {
        let rest = args(&["--records", "recs.txt", "SPEC.md"]);
        assert_eq!(positional(&rest).map(String::as_str), Some("SPEC.md"));
        assert_eq!(flag_value(&rest, "--records").as_deref(), Some("recs.txt"));
        assert_eq!(flag_value(&args(&["--records"]), "--records"), None);
    }

    /// The default is exercised end to end, in READ-ONLY mode only: this
    /// runs from the crate root, so a writing verb with no path would target
    /// the real SPEC.md, and a test that edits the repo's own law is a test
    /// nobody can trust.
    #[test]
    fn the_default_path_resolves_to_our_own_spec() {
        assert_eq!(run(&args(&["check"])).code, 0);
        assert_eq!(run(&args(&["fmt", "--check"])).code, 0);
        assert_eq!(run(&args(&["derive"])).code, 0);
    }

    /// §I's verbs are all built now, and the usage says so without a "not
    /// yet" list to keep in sync with reality.
    #[test]
    fn usage_lists_every_built_verb() {
        let u = usage();
        let built = crate::docs::names().join(", ");
        assert!(u.contains(&format!("built in this binary: {built}")), "{u}");
        assert!(!u.contains("not yet built"), "{u}");
    }

    /// Every verb the registry documents is a verb `dispatch` answers to.
    /// A row for a command that does not exist would document a lie, and
    /// `--help` is where a reader would find it.
    #[test]
    fn every_documented_verb_dispatches() {
        for name in crate::docs::names() {
            let o = dispatch(name, &args(&["no/such/file"]));
            assert!(
                !o.err.contains("unknown command"),
                "documented but not dispatched: {name}"
            );
        }
    }

    /// `docs` is report-only (V10): it prints the reference to stdout and
    /// leaves the README alone. The redirect is the user's to type.
    #[test]
    fn docs_prints_the_reference_and_writes_nothing() {
        let readme =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
        let before = std::fs::read_to_string(&readme).unwrap_or_default();
        let o = run(&args(&["docs"]));
        assert_eq!(o.code, 0, "{}", o.err);
        assert!(o.out.starts_with("## Commands"), "{}", o.out);
        assert_eq!(std::fs::read_to_string(&readme).ok(), Some(before));
    }

    /// `migrate` writes once, then is a no-op -- V2, at the process level.
    /// And a COLLISION it declined is reported rather than left silent, or a
    /// clean exit would claim a file was canonical when it is not.
    /// A spec with one migratable header and one collision.
    fn mixed_spec(name: &str) -> String {
        let body = "## \u{a7}I \u{2014} Interfaces\n- a\n\n\
                    ## \u{a7}V VERSIONING\n- pin\n";
        write_temp(name, body)
    }

    #[test]
    fn migrate_rewrites_what_it_can_and_reports_what_it_declined() {
        let path = mixed_spec("mig");
        let o = run(&args(&["migrate", &path]));
        assert_eq!(o.code, 1, "a collision remains: {o:?}");
        assert!(o.err.contains("DIFFERENT concept"), "{}", o.err);
        let after = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(after.contains("## \u{a7}I INTERFACES"), "{after}");
        assert!(after.contains("## \u{a7}V VERSIONING"), "left: {after}");
        let _ = std::fs::remove_file(&path);
    }

    /// V2 at the process level: the second run writes nothing further, and
    /// in particular adds no second note.
    #[test]
    fn migrating_a_file_twice_writes_once() {
        let path = mixed_spec("mig2");
        let _ = run(&args(&["migrate", &path]));
        let after = std::fs::read_to_string(&path).unwrap_or_default();
        let again = run(&args(&["migrate", &path]));
        assert_eq!(std::fs::read_to_string(&path).ok(), Some(after));
        assert_eq!(again.code, 1, "still not canonical");
        let _ = std::fs::remove_file(&path);
    }

    /// `--check` reports without writing, like `fmt --check`.
    #[test]
    fn migrate_check_never_writes() {
        let src = "## \u{a7}G Goal\none line.\n";
        let path = write_temp("migchk", src);
        let o = run(&args(&["migrate", "--check", &path]));
        assert_eq!(o.code, 1, "{}", o.err);
        assert!(o.err.contains("not canonical"), "{}", o.err);
        assert_eq!(std::fs::read_to_string(&path).as_deref().ok(), Some(src));
        let _ = std::fs::remove_file(&path);
    }

    /// Every command is separated by a blank line and no line runs past 76
    /// columns. Help is the first output a user sees, and a wall of
    /// 200-column text is the version they stop reading.
    #[test]
    fn the_usage_stays_readable_in_a_narrow_terminal() {
        let u = usage();
        for verb in ["check ", "derive ", "anchors "] {
            let block = format!("\n\n  {verb}");
            assert!(u.contains(&block), "no blank line before `{verb}`");
        }
        let widest = u.lines().map(str::len).max().unwrap_or(0);
        assert!(widest <= 76, "a line is {widest} columns wide");
    }

    /// Report-only means exit 0 WITH findings, which is the whole difference
    /// from `check`. Planted with a spec that has an orphan and a gap, so
    /// both reports have something to say and neither gates on it (V10).
    #[test]
    fn a_report_exits_zero_even_with_findings() {
        let body = "## \u{a7}V INVARIANTS\nV1: **cited by nobody.**\n\
                    V3: **also nobody, after a gap.**\n";
        let path = write_temp("report", body);
        let d = run(&args(&["derive", &path]));
        assert_eq!(d.code, 0, "{}", d.err);
        assert!(d.out.contains("orphan V1:"), "{}", d.out);
        let a = run(&args(&["anchors", &path]));
        assert_eq!(a.code, 0, "{}", a.err);
        assert!(a.out.contains("shifted \u{a7}V.2"), "{}", a.out);
        let _ = std::fs::remove_file(&path);
    }

    /// `--format json` emits the machine rendering and still gates.
    #[test]
    fn check_can_report_as_json() {
        let path = write_temp("json", "# spec\n\nV1: a rule with no header.\n");
        let o = run(&args(&["check", "--format", "json", &path]));
        assert_eq!(o.code, 1, "{}", o.err);
        assert!(o.err.starts_with("{\"file\":"), "{}", o.err);
        assert!(o.err.contains("\"rule\":\"V11\""), "{}", o.err);
        let _ = std::fs::remove_file(&path);
    }

    /// `tasks` reports and exits 0 in BOTH renderings, and the JSON is one
    /// object a parser accepts -- asserted through the real argv path,
    /// because that is what the consumer actually runs.
    #[test]
    fn tasks_enumerates_and_exits_zero_in_either_rendering() {
        let path = reportable("tasksverb");
        let human = run(&args(&["tasks", &path]));
        assert_eq!(human.code, 0, "{}", human.err);
        assert!(human.out.starts_with("tasks: 1 rows"), "{}", human.out);
        let json = run(&args(&["tasks", "--format", "json", &path]));
        assert_eq!(json.code, 0, "{}", json.err);
        assert!(json.out.contains("\"id\":\"T1\""), "{}", json.out);
        assert!(json.out.contains("\"status\":\"x\""), "{}", json.out);
        let _ = std::fs::remove_file(&path);
    }

    /// A spec with NO tasks still answers, and exits 0 doing it. This is the
    /// case the consumer distinguishes on: an empty backlog is a fact, and
    /// the absence it falls back on is an unreadable file (exit 2) or a
    /// binary too old to know the verb (exit 2, unknown command).
    #[test]
    fn a_spec_with_no_tasks_answers_rather_than_going_silent() {
        let path = write_temp("notasks", "## \u{a7}V INVARIANTS\nV1: alone.\n");
        let o = run(&args(&["tasks", "--format", "json", &path]));
        assert_eq!(o.code, 0, "{}", o.err);
        assert!(o.out.contains("\"tasks\":[]"), "{}", o.out);
        assert_eq!(run(&args(&["tasks", "no/such/file"])).code, 2);
        let _ = std::fs::remove_file(&path);
    }

    /// The same usage error `check` gives: a caller who asked for JSON and
    /// got prose would parse it and fail somewhere far away.
    #[test]
    fn tasks_rejects_an_unknown_format() {
        let o = run(&args(&["tasks", "--format", "yaml"]));
        assert_eq!(o.code, 2, "{}", o.err);
        assert!(o.err.contains("expected human or json"), "{}", o.err);
    }

    /// An unknown format is a usage error, never a silent fallback to prose.
    /// A caller who asked for json and got a sentence parses it and fails
    /// somewhere far away from the mistake.
    #[test]
    fn an_unknown_format_is_a_usage_error() {
        let path = write_temp("fmt", "# spec\n");
        let o = run(&args(&["check", "--format", "yaml", &path]));
        assert_eq!(o.code, 2, "{}", o.err);
        assert!(o.err.contains("expected human or json"), "{}", o.err);
        let _ = std::fs::remove_file(&path);
    }

    /// The path must survive BOTH flags and both their values sitting in
    /// front of it -- the bug a one-flag skip list would have.
    #[test]
    fn the_path_is_found_past_two_flags() {
        let rest =
            args(&["--records", "recs.txt", "--format", "json", "SPEC.md"]);
        assert_eq!(positional(&rest).map(String::as_str), Some("SPEC.md"));
    }

    /// A broken invocation is still a usage error: that is not a finding.
    #[test]
    fn a_report_with_an_unreadable_path_is_a_usage_error() {
        assert_eq!(run(&args(&["anchors", "no/such/file"])).code, 2);
    }

    /// A named path that cannot be read is still an error.
    #[test]
    fn an_unreadable_named_path_is_a_usage_error() {
        assert_eq!(run(&args(&["fmt", "no/such/file"])).code, 2);
        assert_eq!(run(&args(&["check", "no/such/file"])).code, 2);
    }

    /// Convention over configuration: no path means SPEC.md, the one file
    /// FORMAT.md says every cavekit command reads.
    /// Silence stays the default; `--verbose` is the opt-out (V10). On a
    /// clean run it must say what was examined, not merely "ok".
    #[test]
    fn fmt_is_silent_unless_asked() {
        let quiet = run(&args(&["fmt", "--check"]));
        assert_eq!(quiet.code, 0);
        assert!(quiet.out.is_empty(), "silence is success: {}", quiet.out);
        let loud = run(&args(&["fmt", "--check", "--verbose"]));
        assert_eq!(loud.code, 0);
        assert!(loud.out.contains("longest"), "{}", loud.out);
        assert!(loud.out.contains("spare"), "{}", loud.out);
    }

    /// A summary must never accompany a failure -- it would bury it.
    #[test]
    fn a_failure_is_not_dressed_up_with_a_summary() {
        let path = write_temp("verb", "V1: a rule\nwrapped here\n");
        let o = run(&args(&["fmt", "--check", "--verbose", &path]));
        assert_eq!(o.code, 1, "{}", o.err);
        assert!(o.out.is_empty(), "summary leaked onto a failure: {}", o.out);
        let _ = std::fs::remove_file(&path);
    }

    /// `check --verbose` counts what is IN the spec, and says when the
    /// records rule did not run -- five rules and six look identical from
    /// outside, which is the whole point of saying so.
    #[test]
    fn check_says_what_it_examined_and_what_it_skipped() {
        let quiet = run(&args(&["check"]));
        assert!(quiet.out.is_empty(), "silence is success: {}", quiet.out);
        let loud = run(&args(&["check", "--verbose"]));
        assert!(loud.out.contains("invariants"), "{}", loud.out);
        assert!(loud.out.contains("V16 did not run"), "{}", loud.out);
        let with =
            run(&args(&["check", "--verbose", "--records", ".spec-records"]));
        assert!(with.out.contains("records checked"), "{}", with.out);
    }

    /// Prose must never be appended to JSON: an agent parses that stream,
    /// and a trailing sentence turns valid output into a parse error.
    #[test]
    fn verbose_stays_out_of_the_json_rendering() {
        let o = run(&args(&["check", "--verbose", "--format", "json"]));
        assert_eq!(o.code, 0);
        assert!(o.out.is_empty(), "prose leaked into json mode: {}", o.out);
    }

    #[test]
    fn no_path_means_the_convention() {
        assert_eq!(target(&args(&[])), "SPEC.md");
        assert_eq!(target(&args(&["--check"])), "SPEC.md");
        assert_eq!(target(&args(&["other.md"])), "other.md");
        assert_eq!(
            target(&args(&["--records", "r.txt", "other.md"])),
            "other.md",
            "a flag value is not the path"
        );
    }

    /// The default resolving to a file that is not there says so
    /// DIFFERENTLY, because the two causes need different fixes: a wrong
    /// path is a typo, a missing SPEC.md is usually the wrong directory.
    #[test]
    fn a_missing_default_names_the_convention_not_a_typo() {
        let named = unreadable("no/such/file");
        assert!(named.err.contains("cannot read no/such/file"), "{named:?}");
        let missing = unreadable(DEFAULT_PATH);
        assert!(
            missing.err.contains("run from a project root"),
            "{missing:?}"
        );
        assert_eq!(missing.code, 2);
    }

    /// `--check` gates and never writes: exit 1 on drift, and the file on
    /// disk is untouched.
    #[test]
    fn check_reports_drift_without_writing() {
        let p = std::env::temp_dir()
            .join(format!("microlith-{}.md", std::process::id()));
        let src = "V1: a rule\nwrapped here\n";
        let _ = std::fs::write(&p, src);
        let path = p.to_string_lossy().into_owned();
        let o = run(&args(&["fmt", "--check", &path]));
        assert_eq!(o.code, 1, "{}", o.err);
        assert!(o.err.contains("not formatted"), "{}", o.err);
        assert_eq!(std::fs::read_to_string(&p).ok().as_deref(), Some(src));
        let _ = std::fs::remove_file(&p);
    }

    /// The default mode writes, and a second run is a no-op (V2).
    #[test]
    fn fmt_writes_once_then_is_a_no_op() {
        let p = std::env::temp_dir()
            .join(format!("microlith-w-{}.md", std::process::id()));
        let _ = std::fs::write(&p, "V1: a rule\nwrapped here\n");
        let path = p.to_string_lossy().into_owned();
        assert_eq!(run(&args(&["fmt", &path])).code, 0);
        assert_eq!(
            std::fs::read_to_string(&p).ok(),
            Some("V1: a rule wrapped here\n".to_owned())
        );
        let again = run(&args(&["fmt", "--check", &path]));
        assert_eq!(again.code, 0, "already formatted: {}", again.err);
        let _ = std::fs::remove_file(&p);
    }

    /// `--verbose` picks a DIFFERENT renderer for the report verbs, so the
    /// flag is a branch in dispatch rather than a formatting detail. Both
    /// arms were unexercised: the terse one is what every other test uses.
    /// A spec with one of everything the report verbs address.
    fn reportable(name: &str) -> String {
        write_temp(
            name,
            "## \u{a7}V INVARIANTS\nV1: a rule cited by T1.\n\n\
             ## \u{a7}T TASKS\n| id | scope | tasks | done-when |\n\
             |----|-------|-------|-----------|\n\
             | M1 | core | T1 | done |\nT1|x|a task|V1\n",
        )
    }

    fn deepens(verb: &str, path: &str) {
        let terse = run(&args(&[verb, path]));
        let deep = run(&args(&[verb, "--verbose", path]));
        assert_eq!(terse.code, 0, "{}", terse.err);
        assert!(deep.out.len() >= terse.out.len(), "{verb} said less");
    }

    #[test]
    fn verbose_selects_the_deeper_report_verb() {
        let path = reportable("verbrep");
        deepens("anchors", &path);
        deepens("derive", &path);
        deepens("tasks", &path);
        let _ = std::fs::remove_file(&path);
    }

    /// The `migrate` success summary, which only `--verbose` prints: silence
    /// is success, so this line exists precisely to opt out of it.
    #[test]
    fn migrate_verbose_confirms_a_canonical_file() {
        let src = "## \u{a7}G GOAL\none line.\n";
        let path = write_temp("migverb", src);
        let quiet = run(&args(&["migrate", &path]));
        assert_eq!(quiet.code, 0, "{}", quiet.err);
        assert!(quiet.out.is_empty(), "silence is success: {}", quiet.out);
        let loud = run(&args(&["migrate", "--verbose", &path]));
        assert_eq!(loud.code, 0, "{}", loud.err);
        assert!(loud.out.contains("headers canonical"), "{}", loud.out);
        let _ = std::fs::remove_file(&path);
    }
}
