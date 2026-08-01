//! The `nanokit` binary: a thin shell over the library. All the rules and
//! their tests live in the lib, so the format is testable without spawning
//! a process -- and so a consumer can call the same code instead of
//! re-porting it (V7). `main` does only the I/O the core avoids.

use nanokit::{format_spec, Output};
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
        // Named in the interface section, not built yet. Saying so beats
        // `unknown command`, which sends the reader hunting for a typo.
        "check" | "anchors" | "derive" => Output::usage(format!(
            "nanokit: `{verb}` is specified but not built yet (T4/T5)\n"
        )),
        other => Output::usage(format!(
            "nanokit: unknown command '{other}'\n{}",
            usage()
        )),
    }
}

fn usage() -> String {
    "nanokit -- the cavekit SPEC format, enforced\n\n\
     usage: nanokit <command> [args]\n\n\
     commands:\n  \
       fmt [--check] <path>\n      \
         One line per statement: joins hard wraps, enforces the line cap. \
         Rewrites the file; `--check` reports drift and exits 1 instead. \
         The transform is proven whitespace-only before any write.\n\n\
     built in this binary: fmt\n\
     specified, not yet built: check, anchors, derive\n\n\
     exit: 0 ok | 1 drift or violation | 2 usage\n"
        .to_owned()
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
        let o = run(&args(&["check", "SPEC.md"]));
        assert_eq!(o.code, 2);
        assert!(o.err.contains("not built yet"), "{}", o.err);
        assert!(!o.err.contains("unknown command"), "{}", o.err);
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
