//! The built binary, run in a fixture repository under the target tmpdir.
//! Never the real tree: a test that rewrote README.md would pass by
//! changing what it checks.

use std::path::{Path, PathBuf};
use std::process::Command;

const STALE: &str =
    "# x\n<!-- BEGIN badges -->\nold\n<!-- END badges -->\ntail\n";

const FILES: [(&str, &str); 6] = [
    ("README.md", STALE),
    (".coverage", "# cache\nkey abc\nlines 97.5\n"),
    (
        "Cargo.toml",
        "[package]\nedition = \"2024\"\nrust-version = \"1.95\"\n\n[dependencies]\n# none\n",
    ),
    (
        "hk.pkl",
        "cargo llvm-cov nextest --fail-under-lines 98 --summary-only\n",
    ),
    (
        "flake.lock",
        r#"{"nodes":{"np":{"original":{"ref":"nixos-26.05"}},"root":{"inputs":{"nixpkgs":"np"}}}}"#,
    ),
    (
        ".github/workflows/ci.yml",
        "matrix:\n  os: [macos-latest]\n",
    ),
];

fn fixture(name: &str) -> Result<PathBuf, String> {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    _ = std::fs::remove_dir_all(&root);
    for (path, text) in FILES {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().ok_or("no parent")?)
            .map_err(|e| e.to_string())?;
        std::fs::write(file, text).map_err(|e| e.to_string())?;
    }
    Ok(root)
}

fn run(root: &Path, args: &[&str]) -> Result<(i32, String), String> {
    let out = Command::new(env!("CARGO_BIN_EXE_microlith-dev"))
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    Ok((out.status.code().unwrap_or(-1), stderr))
}

fn readme(root: &Path) -> Result<String, String> {
    std::fs::read_to_string(root.join("README.md")).map_err(|e| e.to_string())
}

#[test]
fn check_reports_stale_and_writes_nothing() -> Result<(), String> {
    let root = fixture("stale")?;
    let (code, stderr) = run(&root, &["readme", "--check"])?;
    assert_eq!(code, 1);
    assert!(
        stderr.contains("- old") && stderr.contains("+ [![nixpkgs 26.05]"),
        "{stderr}"
    );
    assert_eq!(readme(&root)?, STALE);
    Ok(())
}

#[test]
fn fix_rewrites_then_check_is_clean() -> Result<(), String> {
    let root = fixture("fix")?;
    assert_eq!(run(&root, &["readme"])?.0, 0);
    let fixed = readme(&root)?;
    assert!(
        fixed.contains("[![dependencies 0]")
            && fixed.contains("[![coverage 97.5%]"),
        "{fixed}"
    );
    assert!(
        fixed.starts_with("# x\n")
            && fixed.ends_with("<!-- END badges -->\ntail\n")
    );
    assert_eq!(run(&root, &["readme", "--check"])?, (0, String::new()));
    Ok(())
}

#[test]
fn badge_rendering_truncates_a_stale_two_decimal_cache() -> Result<(), String> {
    let root = fixture("truncate")?;
    std::fs::write(root.join(".coverage"), "# cache\nkey abc\nlines 99.45\n")
        .map_err(|e| e.to_string())?;
    assert_eq!(run(&root, &["readme"])?.0, 0);
    assert!(readme(&root)?.contains("[![coverage 99.4%]"));
    Ok(())
}

#[test]
fn a_scoped_run_skips_when_no_input_changed() -> Result<(), String> {
    let root = fixture("scoped")?;
    assert_eq!(run(&root, &["readme", "--check", "SPEC.md"])?.0, 0);
    assert_eq!(run(&root, &["readme", "--check", "Cargo.toml"])?.0, 1);
    Ok(())
}

#[test]
fn a_missing_input_is_named() -> Result<(), String> {
    let root = fixture("missing")?;
    std::fs::remove_file(root.join("flake.lock")).map_err(|e| e.to_string())?;
    let (code, stderr) = run(&root, &["readme", "--check"])?;
    assert_eq!(code, 1);
    assert!(stderr.contains("flake.lock"), "{stderr}");
    Ok(())
}

#[test]
fn usage_exits_two() -> Result<(), String> {
    let root = fixture("usage")?;
    assert_eq!(run(&root, &[])?.0, 2);
    assert_eq!(run(&root, &["readme", "--write"])?.0, 2);
    Ok(())
}
