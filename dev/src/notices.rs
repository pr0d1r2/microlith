//! `notices`: holds `docs/THIRD-PARTY-NOTICES.md`'s dependency claim to the
//! runtime closure `cargo tree` measures. Pure over `&str`; the runner that
//! produces the tree is injected, so tests hand it recorded output.

/// The measured closure's command, as a human would paste it.
pub const TREE: [&str; 9] = [
    "tree",
    "-e",
    "normal",
    "--prefix",
    "none",
    "-f",
    "{p}|{l}",
    "--locked",
    "--offline",
];

const BEGIN: &str = "<!-- BEGIN crates -->\n";
const END: &str = "<!-- END crates -->\n";

/// How the closure is obtained: `cargo tree` output, or why it failed.
pub type Tree<'a> = &'a dyn Fn() -> Result<String, String>;

/// `(name, version, license)`, sorted and distinct, the root excluded: with
/// `--prefix none` the first line is the package itself.
pub fn closure(tree: &str) -> Vec<(String, String, String)> {
    let mut out: Vec<_> = tree.lines().skip(1).filter_map(entry).collect();
    out.sort();
    out.dedup();
    out
}

/// The count the prose states: `zero runtime dependencies` or `N runtime
/// dependencies` (also `dependency`), emphasis ignored.
pub fn stated(doc: &str) -> Option<usize> {
    let words: Vec<&str> = doc.split(|c: char| !c.is_alphanumeric()).collect();
    words.windows(3).find_map(|w| match w {
        [n, "runtime", "dependencies" | "dependency"] => {
            if *n == "zero" {
                Some(0)
            } else {
                n.parse().ok()
            }
        }
        _ => None,
    })
}

/// The table between the markers, one row per crate.
pub fn table(crates: &[(String, String, String)]) -> String {
    let mut s =
        String::from("| Crate | Version | License |\n| --- | --- | --- |\n");
    for (name, version, license) in crates {
        s.push_str(&format!("| {name} | {version} | {license} |\n"));
    }
    s
}

/// The document as it should read: unchanged while the closure is empty,
/// else with the closure table spliced between its markers.
pub fn render(doc: &str, tree: &str) -> Result<String, String> {
    let crates = closure(tree);
    let claim = stated(doc).ok_or("docs/THIRD-PARTY-NOTICES.md states no runtime dependency count (`zero runtime dependencies`, or `N runtime dependencies`). A claim that vanished is not a claim that is right.")?;
    if claim != crates.len() {
        return Err(format!(
            "docs/THIRD-PARTY-NOTICES.md claims {claim} runtime dependencies; `cargo tree -e normal` measures {}. Fix the sentence, and list each new crate's notice.",
            crates.len()
        ));
    }
    if crates.is_empty() {
        return Ok(doc.to_string());
    }
    let (head, rest) = doc.split_once(BEGIN).ok_or("docs/THIRD-PARTY-NOTICES.md has runtime dependencies but no `<!-- BEGIN crates -->` line")?;
    let (_, tail) = rest.split_once(END).ok_or("docs/THIRD-PARTY-NOTICES.md has no `<!-- END crates -->` after its BEGIN")?;
    Ok(format!("{head}{BEGIN}{}{END}{tail}", table(&crates)))
}

/// Exit 0 clean or rewritten, 1 stale or unreadable.
pub fn run(
    root: &std::path::Path,
    check: bool,
    tree: Tree<'_>,
) -> Result<(), String> {
    let path = root.join("docs/THIRD-PARTY-NOTICES.md");
    let have = std::fs::read_to_string(&path)
        .map_err(|e| format!("docs/THIRD-PARTY-NOTICES.md: {e}"))?;
    let want = render(&have, &tree()?)?;
    if have == want {
        return Ok(());
    }
    if check {
        return Err("docs/THIRD-PARTY-NOTICES.md crate table is stale. Run `microlith-dev notices` to regenerate it.".into());
    }
    std::fs::write(&path, want)
        .map_err(|e| format!("docs/THIRD-PARTY-NOTICES.md: {e}"))
}

/// The real runner: `cargo tree` in `root`.
pub fn cargo_tree(root: &std::path::Path) -> Result<String, String> {
    let out = std::process::Command::new("cargo")
        .args(TREE)
        .current_dir(root)
        .output()
        .map_err(|e| format!("cargo: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "`cargo {}` failed: {}",
            TREE.join(" "),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// One `{p}|{l}` line: `name vX.Y.Z [(path|*|proc-macro)...]|license`.
fn entry(line: &str) -> Option<(String, String, String)> {
    let (pkg, license) = line.split_once('|')?;
    let mut words = pkg.split_whitespace();
    Some((
        words.next()?.to_string(),
        words.next()?.trim_start_matches('v').to_string(),
        license.trim().to_string(),
    ))
}
