use std::fmt::Write as _;
use std::fs;
use std::path::Path;

pub fn dependency_count(toml: &str) -> Result<usize, String> {
    let mut in_deps = false;
    let mut count = 0;
    for line in toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_deps = trimmed == "[dependencies]";
            continue;
        }
        if in_deps
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && !trimmed.starts_with(']')
            && !trimmed.starts_with("features")
            && !trimmed.starts_with("default-features")
            && !trimmed.starts_with("version")
            && !trimmed.starts_with("path")
        {
            if trimmed.split_once('=').is_some() {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn nixpkgs_channel(lock: &str) -> Result<String, String> {
    let start = lock
        .find("\"nixpkgs\": {")
        .ok_or("flake.lock has no nixpkgs node")?;
    let rest = &lock[start..];
    let node =
        &rest[..matching_object_end(rest).ok_or("malformed nixpkgs node")?];
    let marker = "\"ref\": \"nixos-";
    let original = node
        .find("\"original\": {")
        .ok_or("nixpkgs node has no original block")?;
    let p = node[original..]
        .find(marker)
        .ok_or("nixpkgs node has no nixos ref")?
        + original
        + marker.len();
    let tail = &node[p..];
    let end = tail.find('"').ok_or("malformed nixpkgs ref")?;
    Ok(tail[..end].to_string())
}

fn matching_object_end(text: &str) -> Option<usize> {
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in text.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                quoted = false;
            }
        } else if c == '"' {
            quoted = true;
        } else if c == '{' {
            depth += 1;
        } else if c == '}' {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

pub fn platforms(ci: &str) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    let mut matrix = false;
    for line in ci.lines() {
        if line.contains("matrix:") {
            matrix = true;
            continue;
        }
        if matrix && line.trim_start().starts_with("os:") {
            let value = line
                .split_once('[')
                .and_then(|(_, x)| x.split_once(']'))
                .map(|x| x.0)
                .ok_or("ci.yml os matrix is malformed")?;
            for raw in value.split(',') {
                let runner = raw.trim().trim_matches(|c| c == '\'' || c == '"');
                let badge = match runner {
                    "ubuntu-latest" => "intel linux",
                    "ubuntu-24.04-arm" => "arm linux",
                    "macos-latest" => "arm macos",
                    other => {
                        return Err(format!("unknown CI runner `{other}`"));
                    }
                };
                out.push(badge.to_string());
            }
            return Ok(out);
        }
    }
    Err("ci.yml has no os matrix".into())
}

pub fn render_badges(
    cargo: &str,
    lock: &str,
    ci: &str,
    hk: &str,
    coverage: &str,
) -> Result<String, String> {
    let edition = field(cargo, "edition")?;
    let msrv = field(cargo, "rust-version")?;
    let deps = dependency_count(cargo)?;
    let floor = hk
        .split("fail-under-lines ")
        .nth(1)
        .and_then(|x| x.split_whitespace().next())
        .ok_or("hk.pkl has no coverage floor")?;
    let cov = coverage
        .lines()
        .find_map(|x| x.strip_prefix("lines "))
        .ok_or(".coverage has no lines value")?
        .trim();
    let chan = nixpkgs_channel(lock)?;
    let mut s = format!(
        "[![CI](https://github.com/pr0d1r2/microlith/actions/workflows/ci.yml/badge.svg)](https://github.com/pr0d1r2/microlith/actions/workflows/ci.yml)\n[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)\n[![crates.io](https://img.shields.io/crates/v/microlith.svg)](https://crates.io/crates/microlith)\n[![docs.rs](https://docs.rs/microlith/badge.svg)](https://docs.rs/microlith)\n[![edition {edition}](https://img.shields.io/badge/edition-{edition}-000000?logo=rust&logoColor=white)](Cargo.toml)\n[![MSRV {msrv}](https://img.shields.io/badge/MSRV-{msrv}-000000?logo=rust&logoColor=white)](Cargo.toml)\n[![dependencies {deps}](https://img.shields.io/badge/dependencies-{deps}-brightgreen)](Cargo.toml)\n[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](Cargo.toml)\n[![gate hk](https://img.shields.io/badge/gate-hk-6E4AFF)](hk.pkl)\n[![coverage {cov}%](https://img.shields.io/badge/coverage-{cov}%25-brightgreen)](hk.pkl)\n[![floor {floor}%](https://img.shields.io/badge/floor-%E2%89%A5{floor}%25-brightgreen)](hk.pkl)\n\n[![nix flake](https://img.shields.io/badge/nix-flake-5277C3?logo=nixos&logoColor=white)](flake.nix)\n[![nixpkgs {chan}](https://img.shields.io/badge/nixpkgs-{chan}-5277C3?logo=nixos&logoColor=white)](flake.lock)\n"
    );
    for p in platforms(ci)? {
        let (v, o) = p.split_once(' ').unwrap_or((p.as_str(), ""));
        writeln!(s, "[![{p}](https://img.shields.io/badge/{o}-5277C3?logo={v}&logoColor=white)](flake.nix)").map_err(|e| e.to_string())?;
    }
    s.push_str("\n[![built with Claude Code](https://img.shields.io/badge/built_with-Claude_Code-D97757)](https://claude.com/claude-code)\n[![built with Opus 5](https://img.shields.io/badge/built_with-Opus_5-D97757)](https://www.anthropic.com/claude)\n[![built with SDD](https://img.shields.io/badge/built_with-spec--driven_development-D97757)](SPEC.md)");
    Ok(s)
}

fn field(text: &str, name: &str) -> Result<String, String> {
    text.lines()
        .find_map(|l| {
            l.strip_prefix(&format!("{name} = \""))
                .and_then(|x| x.strip_suffix('"'))
                .map(str::to_string)
        })
        .ok_or(format!("Cargo.toml has no {name}"))
}

pub fn splice(readme: &str, badges: &str) -> Result<String, String> {
    let begin = "<!-- BEGIN badges -->";
    let end = "<!-- END badges -->";
    let b = readme.lines().filter(|line| *line == begin).count();
    let e = readme.lines().filter(|line| *line == end).count();
    if b != 1 || e != 1 {
        return Err(
            "README.md must contain exactly one badges marker pair".into()
        );
    }
    let start = readme.find(&format!("{begin}\n")).unwrap() + begin.len() + 1;
    let finish = readme.find(&format!("\n{end}")).unwrap();
    if start > finish {
        return Err("README.md badge markers are misordered".into());
    }
    Ok(format!(
        "{}{}\n{}",
        &readme[..start],
        badges,
        &readme[finish + 1..]
    ))
}

pub fn run(
    args: &[String],
    root: &Path,
    _external: &[(&str, &str)],
    err: &mut dyn std::io::Write,
) -> u8 {
    let check = args.iter().any(|a| a == "--check");
    if args.iter().any(|a| a == "--help") || args.len() < 2 {
        let _ = writeln!(err, "usage: microlith-dev readme [--check]");
        return if args.iter().any(|a| a == "--help") {
            0
        } else {
            2
        };
    }
    if args[0] != "readme" {
        let _ = writeln!(err, "unknown command");
        return 2;
    }
    let read = |name: &str| {
        fs::read_to_string(root.join(name)).map_err(|e| format!("{name}: {e}"))
    };
    let desired = match (
        read("Cargo.toml"),
        read("flake.lock"),
        read(".github/workflows/ci.yml"),
        read("hk.pkl"),
        read(".coverage"),
    ) {
        (Ok(a), Ok(b), Ok(c), Ok(d), Ok(e)) => {
            render_badges(&a, &b, &c, &d, &e)
        }
        _ => Err("cannot read badge inputs".into()),
    };
    let desired = match desired
        .and_then(|x| read("README.md").and_then(|r| splice(&r, &x)))
    {
        Ok(x) => x,
        Err(e) => {
            let _ = writeln!(err, "microlith-dev: {e}");
            return 1;
        }
    };
    let path = root.join("README.md");
    let current = fs::read_to_string(&path).unwrap_or_default();
    if current == desired {
        return 0;
    }
    if check {
        let _ = writeln!(
            err,
            "README.md badge block is stale\nwant: generated badges\nhave: committed badges"
        );
        return 1;
    }
    if fs::write(path, desired).is_err() {
        let _ = writeln!(err, "microlith-dev: cannot write README.md");
        return 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deps_are_keys_not_lines() {
        assert_eq!(dependency_count("[dependencies]\n# comment\nserde = {\n features = [\"derive\"]\n}\nregex = \"1\"\n").unwrap(), 2);
    }
    #[test]
    fn lock_is_anchored_to_node() {
        let x = r#"{"nixpkgs": {"inputs": {"nixpkgs": {"ref": "nixos-99.99"}},"original": {"ref": "nixos-26.05"}},"nix-hk": {"original": {"ref": "nixos-1.01"}}}"#;
        assert_eq!(nixpkgs_channel(x).unwrap(), "26.05");
    }
    #[test]
    fn ci_platforms_and_unknown() {
        assert_eq!(
            platforms("matrix:\n os: [ubuntu-latest, macos-latest]").unwrap(),
            vec!["intel linux", "arm macos"]
        );
        assert!(platforms("matrix:\n os: [windows-latest]").is_err());
    }
    #[test]
    fn splice_is_idempotent() {
        let x = "a\n<!-- BEGIN badges -->\nold\n<!-- END badges -->\nz\n";
        let y = splice(x, "new").unwrap();
        assert_eq!(splice(&y, "new").unwrap(), y);
    }
    #[test]
    fn hk_declares_every_input_and_unscoped_all() {
        let hk = include_str!("../../hk.pkl");
        let step = hk.split("[\"readme-badges\"]").nth(1).unwrap();
        for input in [
            "README.md",
            ".coverage",
            "Cargo.toml",
            "flake.nix",
            "flake.lock",
            "hk.pkl",
            ".github/workflows/ci.yml",
            "dev/**",
        ] {
            assert!(step.contains(input), "missing {input}");
        }
        assert!(
            hk.contains("[\"readme-badges-all\"]")
                && hk.contains("glob = List(\"**/*\")")
        );
    }
}
