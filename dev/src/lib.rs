//! `microlith-dev`: this repository's own tooling, never the product.
//!
//! `publish = false` and `release = false` keep it off crates.io and out of
//! every release, so it reaches the dev shell and the gate (`hk.pkl`) and
//! never a consumer's PATH. Every reader below is a pure function over
//! `&str`; only `run` touches the filesystem.

use std::fmt::Write as _;
use std::path::Path;

/// One line, answered on `--help` and on a usage error.
pub const USAGE: &str =
    "usage: microlith-dev readme [--check] [<changed path>...]";

/// Every file the badge block is read from, plus the README it lands in and
/// this crate itself. `hk.pkl`'s `readme-badges` glob must name exactly
/// these -- a test ties the two, so neither list can drift alone.
pub const INPUTS: [&str; 8] = [
    "README.md",
    ".coverage",
    "Cargo.toml",
    "flake.nix",
    "flake.lock",
    "hk.pkl",
    ".github/workflows/ci.yml",
    "dev/**",
];

const BEGIN: &str = "<!-- BEGIN badges -->\n";
const END: &str = "<!-- END badges -->\n";

/// Dependencies counted where their KEY starts, never per line: a comment,
/// a blank, or a formatter-wrapped continuation (`"derive",`, `] }`) is
/// indented or punctuated and is not a key. `[dependencies.x]` tables count
/// once each.
pub fn dependency_count(toml: &str) -> usize {
    let tables = toml
        .lines()
        .filter(|line| line.starts_with("[dependencies."))
        .count();
    table_lines(toml, "[dependencies]")
        .filter(|line| starts_key(line))
        .count()
        .saturating_add(tables)
}

fn table_lines<'a>(
    toml: &'a str,
    header: &'a str,
) -> impl Iterator<Item = &'a str> {
    toml.lines()
        .scan(false, move |inside, line| {
            if line.starts_with('[') {
                *inside = line.trim_end() == header;
                return Some(None);
            }
            Some(inside.then_some(line))
        })
        .flatten()
}

fn starts_key(line: &str) -> bool {
    line.starts_with(|c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }) && line.contains('=')
}

/// The nixpkgs channel, read from the node the ROOT's `nixpkgs` input
/// resolves to -- following `follows` paths -- never from the first
/// `"ref": "nixos-..."` in the file. A prefix or first-match read enters
/// whichever input happens to sit above it and badges that one's channel.
pub fn nixpkgs_channel(lock: &str) -> Result<String, String> {
    let nodes =
        member(lock, "nodes").ok_or("flake.lock has no `nodes` object")?;
    let name = resolve(nodes, &["nixpkgs"])
        .ok_or("flake.lock: the root's `nixpkgs` input resolves to no node")?;
    let reference = member(nodes, &name)
        .and_then(|node| member(node, "original"))
        .and_then(|original| member(original, "ref"))
        .and_then(unquote)
        .ok_or(format!("flake.lock: node `{name}` has no `original.ref`"))?;
    reference.strip_prefix("nixos-").map(str::to_string).ok_or(format!(
        "flake.lock: nixpkgs follows `{reference}`, not a `nixos-XX.YY` channel -- the badge must say what it follows NOW"
    ))
}

/// Walk an input path from the root. A string input names a node; an array
/// is a `follows`, itself a path from the root.
fn resolve(nodes: &str, path: &[&str]) -> Option<String> {
    let mut node = "root".to_string();
    for step in path {
        let input = member(nodes, &node)
            .and_then(|n| member(n, "inputs"))
            .and_then(|inputs| member(inputs, step))?;
        node = match unquote(input) {
            Some(name) => name.to_string(),
            None => resolve(nodes, &elements(input)?)?,
        };
    }
    Some(node)
}

fn elements(array: &str) -> Option<Vec<&str>> {
    let inner = array.strip_prefix('[')?.strip_suffix(']')?;
    inner.split(',').map(|item| unquote(item.trim())).collect()
}

fn unquote(value: &str) -> Option<&str> {
    value.strip_prefix('"')?.strip_suffix('"')
}

/// The raw text of `key`'s value in the JSON object `obj`, at the object's
/// own depth only: a key of the same name inside a nested value is skipped,
/// which is the whole point.
fn member<'a>(obj: &'a str, key: &str) -> Option<&'a str> {
    let mut rest = obj.trim_start().strip_prefix('{')?;
    loop {
        rest = rest.trim_start_matches(|c: char| c == ',' || c.is_whitespace());
        let name = rest.get(..value_len(rest)?)?;
        rest = rest
            .get(name.len()..)?
            .trim_start()
            .strip_prefix(':')?
            .trim_start();
        let value = rest.get(..value_len(rest)?)?;
        if unquote(name)? == key {
            return Some(value);
        }
        rest = rest.get(value.len()..)?;
    }
}

/// Byte length of the JSON value opening `text`: a string, an object or
/// array, or a bare scalar up to its delimiter. `None` when unbalanced.
fn value_len(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    let mut chars = text.char_indices();
    while let Some((i, c)) = chars.next() {
        depth = match (c, depth) {
            ('"', _) => skip_string(&mut chars).map(|()| depth)?,
            ('{' | '[', _) => depth.checked_add(1)?,
            ('}' | ']' | ',', 0) => return Some(i),
            ('}' | ']', _) => depth.checked_sub(1)?,
            _ => continue,
        };
        if depth == 0 {
            return Some(chars.offset());
        }
    }
    (depth == 0).then_some(text.len())
}

fn skip_string(chars: &mut std::str::CharIndices<'_>) -> Option<()> {
    while let Some((_, c)) = chars.next() {
        match c {
            '\\' => _ = chars.next()?,
            '"' => return Some(()),
            _ => {}
        }
    }
    None
}

/// One badge per CI RUNNER, from `ci.yml`'s `os:` matrix -- never from the
/// flake's `systems`, which can name a system no runner builds. A runner
/// this table does not know is an error, never a guess. x86_64 Linux is
/// both CPU vendors, as it always was.
pub fn platforms(ci: &str) -> Result<Vec<&'static str>, String> {
    let mut out = Vec::new();
    for runner in runners(ci)? {
        match runner {
            "ubuntu-latest" => out.extend(["intel linux", "amd linux"]),
            "ubuntu-24.04-arm" => out.push("arm linux"),
            "macos-latest" => out.push("arm macos"),
            other => {
                return Err(format!(
                    "ci.yml: unknown runner `{other}` -- add its badge to microlith-dev's `platforms`"
                ));
            }
        }
    }
    Ok(out)
}

/// The `os:` list under `matrix:`, in either flow (`[a, b]`) or block
/// (`- a`) style.
fn runners(ci: &str) -> Result<Vec<&str>, String> {
    let mut lines = ci
        .lines()
        .skip_while(|line| line.trim() != "matrix:")
        .skip(1);
    let os = lines
        .find_map(|line| line.trim().strip_prefix("os:"))
        .ok_or("ci.yml has no `os:` under `matrix:`")?
        .trim();
    Ok(listed(os, lines))
}

fn listed<'a>(
    os: &'a str,
    block: impl Iterator<Item = &'a str>,
) -> Vec<&'a str> {
    let names: Vec<&str> = match os.strip_prefix('[') {
        Some(flow) => flow.trim_end_matches(']').split(',').collect(),
        None => block
            .map_while(|line| line.trim().strip_prefix('-'))
            .collect(),
    };
    names
        .into_iter()
        .map(|name| name.trim().trim_matches(|c| c == '\'' || c == '"'))
        .filter(|name| !name.is_empty())
        .collect()
}

/// How `render_badges` reads a file, so a test can hand it strings.
pub type Read<'a> = &'a dyn Fn(&str) -> Result<String, String>;

/// The badge block, from the files that own each number. One renderer
/// serves both `--check` and the rewrite.
pub fn render_badges(read: Read<'_>) -> Result<String, String> {
    let cargo = read("Cargo.toml")?;
    let hk = read("hk.pkl")?;
    let ci = read(".github/workflows/ci.yml")?;
    let values = [
        ("{edition}", field(&cargo, "edition")?),
        ("{msrv}", field(&cargo, "rust-version")?),
        ("{deps}", dependency_count(&cargo).to_string()),
        ("{floor}", floor(&hk)?),
        ("{cov}", coverage(&read(".coverage")?)?),
        ("{chan}", nixpkgs_channel(&read("flake.lock")?)?),
        ("{platforms}", platform_badges(&ci)?),
    ];
    Ok(values
        .iter()
        .fold(TEMPLATE.to_string(), |s, (k, v)| s.replace(k, v)))
}

const TEMPLATE: &str = "\
[![CI](https://github.com/pr0d1r2/microlith/actions/workflows/ci.yml/badge.svg)](https://github.com/pr0d1r2/microlith/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/microlith.svg)](https://crates.io/crates/microlith)
[![docs.rs](https://docs.rs/microlith/badge.svg)](https://docs.rs/microlith)
[![edition {edition}](https://img.shields.io/badge/edition-{edition}-000000?logo=rust&logoColor=white)](Cargo.toml)
[![MSRV {msrv}](https://img.shields.io/badge/MSRV-{msrv}-000000?logo=rust&logoColor=white)](Cargo.toml)
[![dependencies {deps}](https://img.shields.io/badge/dependencies-{deps}-brightgreen)](Cargo.toml)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](Cargo.toml)
[![gate hk](https://img.shields.io/badge/gate-hk-6E4AFF)](hk.pkl)
[![coverage {cov}%](https://img.shields.io/badge/coverage-{cov}%25-brightgreen)](hk.pkl)
[![floor {floor}%](https://img.shields.io/badge/floor-%E2%89%A5{floor}%25-brightgreen)](hk.pkl)

[![nix flake](https://img.shields.io/badge/nix-flake-5277C3?logo=nixos&logoColor=white)](flake.nix)
[![nixpkgs {chan}](https://img.shields.io/badge/nixpkgs-{chan}-5277C3?logo=nixos&logoColor=white)](flake.lock)
{platforms}
[![built with Claude Code](https://img.shields.io/badge/built_with-Claude_Code-D97757)](https://claude.com/claude-code)
[![built with Opus 5](https://img.shields.io/badge/built_with-Opus_5-D97757)](https://www.anthropic.com/claude)
[![built with SDD](https://img.shields.io/badge/built_with-spec--driven_development-D97757)](SPEC.md)
";

fn platform_badges(ci: &str) -> Result<String, String> {
    let mut s = String::new();
    for p in platforms(ci)? {
        let (vendor, os) = p.split_once(' ').unwrap_or((p, ""));
        writeln!(
            s,
            "[![{p}](https://img.shields.io/badge/{os}-5277C3?logo={vendor}&logoColor=white)](flake.nix)"
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(s)
}

fn field(cargo: &str, name: &str) -> Result<String, String> {
    let prefix = format!("{name} = \"");
    cargo
        .lines()
        .find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('"'))
        .map(str::to_string)
        .ok_or(format!("Cargo.toml has no `{name}`"))
}

fn floor(hk: &str) -> Result<String, String> {
    hk.split("fail-under-lines ")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .map(str::to_string)
        .ok_or("hk.pkl has no `fail-under-lines` coverage floor".into())
}

fn coverage(cache: &str) -> Result<String, String> {
    cache
        .lines()
        .find_map(|line| line.strip_prefix("lines "))
        .map(|value| truncate_coverage(value.trim()))
        .transpose()?
        .ok_or(".coverage has no `lines` value -- run `hk fix --all` to measure it".into())
}

/// Keep the badge at the same one-decimal precision as the coverage cache.
/// Truncation is deliberate: rounding can straddle the platform-dependent
/// tenth, while truncation never overstates the measured coverage.
fn truncate_coverage(value: &str) -> Result<String, String> {
    let (whole, fraction) = value
        .split_once('.')
        .ok_or(format!("invalid coverage value `{value}`"))?;
    if whole.is_empty()
        || !whole.chars().all(|c| c.is_ascii_digit())
        || !fraction.chars().all(|c| c.is_ascii_digit())
    {
        return Err(format!("invalid coverage value `{value}`"));
    }
    Ok(format!(
        "{whole}.{}",
        fraction.chars().next().unwrap_or('0')
    ))
}

/// The README with its badge block replaced. Exactly one marker pair, in
/// order, or an error naming which.
pub fn splice(readme: &str, badges: &str) -> Result<String, String> {
    let (head, rest) = readme
        .split_once(BEGIN)
        .ok_or("README.md has no `<!-- BEGIN badges -->` line")?;
    let (_, tail) = rest
        .split_once(END)
        .ok_or("README.md has no `<!-- END badges -->` after its BEGIN")?;
    if tail.contains(BEGIN) || tail.contains(END) || rest.contains(BEGIN) {
        return Err("README.md must hold exactly one badges marker pair".into());
    }
    Ok(format!("{head}{BEGIN}{badges}{END}{tail}"))
}

/// Lines only in `want` as `+`, only in `have` as `-`: what `--check`
/// prints, so a stale block says WHICH badge moved.
pub fn diff(have: &str, want: &str) -> String {
    let only = |a: &str, b: &str, sign: char| -> String {
        a.lines()
            .filter(|line| !b.lines().any(|other| other == *line))
            .map(|line| format!("{sign} {line}\n"))
            .collect()
    };
    format!("{}{}", only(have, want, '-'), only(want, have, '+'))
}

/// Paths a scoped run was handed touch an input, or no paths were handed
/// (an unscoped run compares everything).
pub fn in_scope(paths: &[&str]) -> bool {
    paths.is_empty()
        || paths.iter().any(|path| {
            INPUTS.iter().any(|input| {
                input
                    .strip_suffix("**")
                    .map_or(path == input, |dir| path.starts_with(dir))
            })
        })
}

/// Exit 0 clean or rewritten, 1 stale or unreadable, 2 usage.
pub fn run(args: &[String], root: &Path, err: &mut dyn std::io::Write) -> u8 {
    let mut args = args.iter().map(String::as_str);
    if args.next() != Some("readme") {
        return usage(err, 2);
    }
    let (flags, paths): (Vec<&str>, Vec<&str>) =
        args.partition(|a| a.starts_with("--"));
    match flags.as_slice() {
        ["--help"] => usage(err, 0),
        [] | ["--check"] if !in_scope(&paths) => 0,
        [] | ["--check"] => readme(root, !flags.is_empty(), err),
        _ => usage(err, 2),
    }
}

fn usage(err: &mut dyn std::io::Write, code: u8) -> u8 {
    _ = writeln!(err, "{USAGE}");
    code
}

fn readme(root: &Path, check: bool, err: &mut dyn std::io::Write) -> u8 {
    let outcome = compare(root)
        .and_then(|(have, want)| settle(root, check, &have, &want));
    outcome.map_or_else(
        |e| {
            _ = writeln!(err, "microlith-dev: {e}");
            1
        },
        |()| 0,
    )
}

/// The README as committed, and as the inputs say it should be.
fn compare(root: &Path) -> Result<(String, String), String> {
    let read = |name: &str| {
        std::fs::read_to_string(root.join(name))
            .map_err(|e| format!("{name}: {e}"))
    };
    let have = read("README.md")?;
    let want = splice(&have, &render_badges(&read)?)?;
    Ok((have, want))
}

fn settle(
    root: &Path,
    check: bool,
    have: &str,
    want: &str,
) -> Result<(), String> {
    if have == want {
        return Ok(());
    }
    if check {
        return Err(format!(
            "README.md badge block is stale. Run `hk fix` (or `microlith-dev readme`) to regenerate it.\n{}",
            diff(have, want)
        ));
    }
    std::fs::write(root.join("README.md"), want)
        .map_err(|e| format!("README.md: {e}"))
}

#[cfg(test)]
mod tests;
