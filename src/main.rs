//! The `nanokit` binary: a thin shell over the library. All the rules and
//! their tests live in the lib, so the format is testable without spawning
//! a process -- and so a consumer can call the same code instead of
//! re-porting it (V7). `main` does only the I/O the core avoids.

use nanokit::check::{parse_records, Record};
use nanokit::{check_spec, format_spec, Output};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = run(&args);
    print!("{}", o.out);
    eprint!("{}", o.err);
    ExitCode::from(o.code)
}

fn run(args: &[String]) -> Output {
    match args.split_first() {
        None => Output::usage(usage()),
        Some((verb, rest)) => dispatch(verb, rest),
    }
}

fn dispatch(verb: &str, rest: &[String]) -> Output {
    match verb {
        "--help" | "-h" => Output::ok(usage()),
        "--version" | "-V" => {
            Output::ok(format!("nanokit {}\n", env!("CARGO_PKG_VERSION")))
        }
        "fmt" => fmt(rest),
        "check" => check(rest),
        // Named in the interface section, not built yet. Saying so beats
        // `unknown command`, which sends the reader hunting for a typo.
        "anchors" | "derive" => Output::usage(format!(
            "nanokit: `{verb}` is specified but not built yet (T5)\n"
        )),
        other => Output::usage(format!(
            "nanokit: unknown command '{other}'\n{}",
            usage()
        )),
    }
}

/// The usage text, a const so the string literal is not counted as
/// function length -- the limit exists to bound BRANCHING, and prose
/// has none.
const USAGE: &str = "nanokit -- the cavekit SPEC format, enforced\n\n\
     usage: nanokit <command> [args]\n\n\
     commands:\n  \
       fmt [--check] <path>\n      \
         One line per statement: joins hard wraps, enforces the line cap. \
         Rewrites the file; `--check` reports drift and exits 1 instead. \
         The transform is proven whitespace-only before any write.\n  \
       check [--records <file>] <path>\n      \
         The structural rules: sections present and ordered, ids unique, \
         citations resolve, rows sorted, every task in exactly one \
         milestone. `--records` adds the rejected-option check, whose \
         baseline the caller owns because survival is a claim about edits \
         rather than about the file.\n\n\
     built in this binary: fmt, check\n\
     specified, not yet built: anchors, derive\n\n\
     exit: 0 ok | 1 drift or violation | 2 usage\n";

fn usage() -> String {
    USAGE.to_owned()
}

/// `fmt [--check] <path>`. The check mode never writes, so it is safe in a
/// gate; the default mode writes only after the losslessness proof (V1).
fn fmt(rest: &[String]) -> Output {
    let check = rest.iter().any(|a| a == "--check");
    let Some(path) = rest.iter().find(|a| !a.starts_with('-')) else {
        return Output::usage("nanokit: fmt needs a path\n".to_owned());
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Output::usage(format!("nanokit: cannot read {path}\n"));
    };
    match format_spec(&text) {
        Err(e) => Output::drift(format!("nanokit: {path}: {e}\n")),
        Ok(out) => apply(path, &text, &out, check),
    }
}

/// `check [--records <file>] <path>`. Report-only by construction: it reads
/// two files and writes none, so it is safe anywhere a gate runs (V10).
fn check(rest: &[String]) -> Output {
    let Some(path) = positional(rest) else {
        return Output::usage("nanokit: check needs a path\n".to_owned());
    };
    let Ok(text) = std::fs::read_to_string(path) else {
        return Output::usage(format!("nanokit: cannot read {path}\n"));
    };
    match records_from(rest) {
        Err(e) => Output::usage(e),
        Ok(records) => report(path, &check_spec(&text, &records)),
    }
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
        Err(e) => Err(format!("nanokit: cannot read {path}: {e}\n")),
    }
}

fn report(path: &str, violations: &[String]) -> Output {
    if violations.is_empty() {
        return Output::ok(String::new());
    }
    let listed = violations
        .iter()
        .map(|v| format!("nanokit: {path}: {v}\n"))
        .collect::<Vec<_>>()
        .concat();
    Output::drift(listed)
}

/// The value after `name`, if the flag is present with one.
fn flag_value(rest: &[String], name: &str) -> Option<String> {
    let at = rest.iter().position(|a| a == name)?;
    rest.get(at.saturating_add(1))
        .filter(|v| !v.starts_with('-'))
        .cloned()
}

/// The first argument that is neither a flag nor a flag's value.
fn positional(rest: &[String]) -> Option<&String> {
    let skip = flag_value(rest, "--records");
    rest.iter()
        .find(|a| !a.starts_with('-') && Some(*a) != skip.as_ref())
}

/// Report the drift, or write the formatted text.
fn apply(path: &str, text: &str, out: &str, check: bool) -> Output {
    if out == text {
        return Output::ok(String::new());
    }
    if check {
        return Output::drift(format!(
            "nanokit: {path} is not formatted -- run `nanokit fmt {path}`\n"
        ));
    }
    match std::fs::write(path, out) {
        Ok(()) => Output::ok(format!("nanokit: formatted {path}\n")),
        Err(e) => Output::usage(format!("nanokit: cannot write {path}: {e}\n")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(run(&args(&["--version"])).out.contains("nanokit"));
    }

    /// A specified-but-unbuilt verb says so, rather than reading as a typo
    /// while the usage text advertises it.
    #[test]
    fn an_unbuilt_verb_names_itself_not_a_typo() {
        let o = run(&args(&["anchors", "SPEC.md"]));
        assert_eq!(o.code, 2);
        assert!(o.err.contains("not built yet"), "{}", o.err);
        assert!(!o.err.contains("unknown command"), "{}", o.err);
    }

    fn write_temp(name: &str, body: &str) -> String {
        let p = std::env::temp_dir()
            .join(format!("nanokit-{name}-{}.md", std::process::id()));
        let _ = std::fs::write(&p, body);
        p.to_string_lossy().into_owned()
    }

    /// `check` gates: a violation exits 1 and NAMES the rule, so the reader
    /// goes to the invariant rather than guessing which one fired.
    #[test]
    fn check_names_the_rule_it_failed() {
        let path = write_temp("bad", "# spec\n\nnothing here\n");
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

    #[test]
    fn check_without_a_path_is_a_usage_error() {
        assert_eq!(run(&args(&["check"])).code, 2);
        assert_eq!(run(&args(&["check", "no/such/file"])).code, 2);
    }

    #[test]
    fn usage_separates_built_from_specified() {
        let u = usage();
        assert!(u.contains("built in this binary: fmt"));
        assert!(u.contains("specified, not yet built"));
    }

    #[test]
    fn fmt_without_a_path_is_a_usage_error() {
        assert_eq!(run(&args(&["fmt"])).code, 2);
        assert_eq!(run(&args(&["fmt", "no/such/file"])).code, 2);
    }

    /// `--check` gates and never writes: exit 1 on drift, and the file on
    /// disk is untouched.
    #[test]
    fn check_reports_drift_without_writing() {
        let p = std::env::temp_dir()
            .join(format!("nanokit-{}.md", std::process::id()));
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
            .join(format!("nanokit-w-{}.md", std::process::id()));
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
}
