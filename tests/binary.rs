//! The five lines the library cannot reach: argv in, streams out, code back.
//!
//! Every rule and every verb is tested through `run()`, which returns an
//! `Output` struct rather than touching a process -- that is the design, and
//! it is why the suite is fast and the rules are callable by a consumer (V7).
//!
//! It leaves `main.rs` at ZERO percent, and what lives there is not
//! decoration. `ExitCode::from(o.code)` is the whole gate contract: `check`
//! exits 1 on drift, and every hook, CI job and `hk` step downstream believes
//! that number. Drop it, or route `out` to stderr, or forget `.skip(1)` over
//! argv, and the library suite stays entirely green while the binary is
//! broken in the way that matters most -- a gate that cannot fail.
//!
//! So this is the one place a process is worth spawning. `CARGO_BIN_EXE_mth`
//! is cargo's own path to the built binary, so it costs no dependency (§C).

use std::process::Command;

/// Run the real binary. `-1` for a code stands in for "killed by a signal",
/// which is a failure however it is spelled.
fn mth(args: &[&str]) -> (String, String, i32) {
    let Ok(out) = Command::new(env!("CARGO_BIN_EXE_mth")).args(args).output()
    else {
        return (String::new(), String::new(), -1);
    };
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn write_temp(name: &str, body: &str) -> String {
    let p = std::env::temp_dir()
        .join(format!("microlith-bin-{name}-{}.md", std::process::id()));
    let _ = std::fs::write(&p, body);
    p.to_string_lossy().into_owned()
}

/// THE ONE THAT MATTERS: a violation exits 1 through the real process.
///
/// The library has always known this; nothing checked that the number
/// survived the trip to the shell. A gate that cannot fail is worse than no
/// gate, because everything downstream reports success.
#[test]
fn a_violation_exits_one_from_the_binary() {
    let spec =
        write_temp("drift", "## \u{a7}V INVARIANTS\nV1: **a.** see V9\n");
    let (out, err, code) = mth(&["check", &spec]);
    assert_eq!(code, 1, "out={out} err={err}");
    assert!(err.contains("V13"), "the finding goes to stderr: {err}");
    assert!(out.is_empty(), "and nothing to stdout: {out}");
}

/// ...and a clean file exits 0, or the rule above would be satisfied by a
/// binary that always fails.
#[test]
fn a_clean_file_exits_zero_from_the_binary() {
    let spec = write_temp("clean", "## \u{a7}V INVARIANTS\nV1: **a rule.**\n");
    let (out, err, code) = mth(&["check", &spec]);
    assert_eq!(code, 0, "out={out} err={err}");
}

/// A usage error is 2, which is the distinction a caller scripts against:
/// 1 means the file is wrong, 2 means the invocation is.
#[test]
fn a_usage_error_exits_two_from_the_binary() {
    let (_, err, code) = mth(&["nonesuch"]);
    assert_eq!(code, 2);
    assert!(err.contains("unknown command"), "{err}");
}

/// Report output goes to STDOUT, so `mth docs > README-block.md` works.
/// Swapping the two streams would leave every library test green.
#[test]
fn a_report_goes_to_stdout() {
    let (out, err, code) = mth(&["docs"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.starts_with("## Commands"), "{out}");
    assert!(err.is_empty(), "nothing to stderr on success: {err}");
}

/// `.skip(1)` over argv, proven by the verb being read at all: without it
/// the binary's own path would be the verb and everything would be a usage
/// error -- including this.
#[test]
fn the_first_argument_is_the_verb_not_the_binary() {
    let (out, _, code) = mth(&["--version"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("microlith "), "{out}");
}
