//! The CORPUS SWEEP, stored rather than retyped.
//!
//! T15 measured the rules against the fleet and found three miscalibrations
//! -- each a TRUE rule delivering a FALSE message -- because they had been
//! generalised from n=2 without measuring. V39 needed the same measurement
//! one letter further out: is `§F` unclaimed, and at what denominator. Every
//! future change to the section set needs it again.
//!
//! That is three runs of one thing, and the first two were shell pipelines
//! typed from memory and thrown away. A measurement whose method is not
//! stored is one nobody can repeat or check -- so the sweep lives here, in
//! the language the rules are written in, calling the same `check_spec` a
//! consumer calls (V7).
//!
//! NOT A VERB, and not shipped. `Cargo.toml` excludes `examples/` from the
//! `.crate`: this reads paths that exist only on a developer's disk, and a
//! crate a consumer downloads has no fleet to sweep. It stays inside the
//! gate all the same -- `cargo clippy --all-targets` lints it like `src/`.
//!
//! PRIVACY (§C): the fleet repos are not named here or in the output. What
//! the sweep prints is COUNTS -- a denominator, a per-rule tally, a letter
//! census -- which is what a calibration record needs and all it needs.
//!
//! ```text
//! cargo run --example corpus -- ~/projects
//! ```
//!
//! One or more roots, each walked for files named `SPEC.md`. Directories a
//! sweep must not count -- backups, vendored copies, build output -- are
//! skipped by name, because a corpus that counts the same spec four times
//! reports a denominator that means nothing.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use microlith::{check_spec, format_spec, migrate_declined};

/// Directory names that hold a COPY rather than a spec of their own.
///
/// A backup, a vendored tree and a build directory all contain real
/// `SPEC.md` files, and counting them inflates every number in the report --
/// which is precisely the failure T15 named: a count whose denominator is
/// wrong says nothing, however carefully it was taken.
const SKIP: [&str; 8] = [
    ".git",
    ".direnv",
    "node_modules",
    "target",
    "result",
    "vendor",
    ".cache",
    ".claude",
];

/// A directory whose NAME says it is a copy of another one.
///
/// WIDENED after it let four through. `foo--before--rewind`,
/// `foo-2025-10-16--11-54` and `foo-old` are all snapshots of a project that
/// is also present live, and counting them makes one project look like two
/// or three. That is not a tidiness problem: `\u{a7}F` was reported in twice
/// as many specs as actually use it, because one project's pre-rewrite copy
/// was counted as a second adopter -- and a letter's claim rests entirely on
/// how many INDEPENDENT projects spell it.
///
/// Shapes rather than a list of names, since the next snapshot will be
/// stamped with a different hour.
fn is_copy(name: &str) -> bool {
    SKIP.contains(&name)
        || name.contains("backup")
        || name.contains("-bak")
        || name.contains("before")
        || name.starts_with("old")
        || name.ends_with("-old")
        || looks_dated(name)
}

/// Whether the name carries a `YYYY-MM-DD` stamp, which a hand-made
/// directory almost never does and a snapshot almost always does.
fn looks_dated(name: &str) -> bool {
    let b = name.as_bytes();
    b.windows(10).any(|w| {
        w.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
    })
}

/// SYMLINKS ARE NOT FOLLOWED. `is_dir()` follows them and this recursion has
/// no visited set, so one link pointing at an ancestor is an unbounded
/// descent. Not hypothetical for a `~/projects` sweep: this very repo carries
/// `result -> /nix/store/...`, which survived only because `result` happens
/// to be a skipped name.
///
/// A link also cannot hold a spec unreachable by its real path, so following
/// one could only ever DOUBLE-COUNT -- B27, B28 and B31's defect arriving by
/// a fourth route.
fn walk(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        visit(&entry, out);
    }
}

/// One directory entry: descend, collect, or ignore.
fn visit(entry: &std::fs::DirEntry, out: &mut Vec<PathBuf>) {
    let Ok(kind) = entry.file_type() else {
        return;
    };
    if kind.is_symlink() {
        return;
    }
    let name = entry.file_name().to_string_lossy().into_owned();
    if kind.is_dir() {
        if !is_copy(&name) {
            walk(&entry.path(), out);
        }
    } else if name == "SPEC.md" {
        out.push(entry.path());
    }
}

/// The section LETTER a `## §X` line opens, if it opens one.
///
/// Deliberately NOT `check::is_header_for`, and not a second copy of it
/// either: that function answers "is this the header for THIS letter",
/// which is a rule. This asks only which letter a header carries, so the
/// census can count letters the rules know nothing about -- the question
/// V39 had to answer BEFORE `§F` was a known letter at all.
fn header_letter(line: &str) -> Option<char> {
    let rest = line.strip_prefix("## \u{a7}")?;
    let mut chars = rest.chars();
    let letter = chars.next().filter(|c| c.is_ascii_uppercase())?;
    chars
        .next()
        .is_none_or(|c| !c.is_ascii_alphanumeric())
        .then_some(letter)
}

/// A report line, or nothing when the count is zero. A `0` printed for
/// every category a corpus happens not to contain is noise the reader has
/// to skip past to reach the numbers that moved.
fn line_if(label: &str, n: usize) -> String {
    match n {
        0 => String::new(),
        n => format!("{label}: {n}\n"),
    }
}

/// One more of `key`, whatever the map counts.
fn bump<K: Ord>(counts: &mut BTreeMap<K, usize>, key: K) {
    let n = counts.entry(key).or_default();
    *n = n.saturating_add(1);
}

/// A one-line tally, keys in map order: `by rule: V11=9 V13=8`.
fn tally<K: std::fmt::Display>(
    head: &str,
    counts: &BTreeMap<K, usize>,
    sigil: &str,
) -> String {
    let mut out = head.to_owned();
    for (key, n) in counts {
        out.push_str(&format!(" {sigil}{key}={n}"));
    }
    out.push('\n');
    out
}

/// Everything after `## §X` on a header line, lowercased -- the LABEL.
///
/// The census counts letters; this reads what a letter is CALLED, which is
/// the question that decides whether a letter is free to claim. `§F` is
/// unclaimed only if nobody is already using it for something else, and a
/// count of headers cannot tell you that -- V39's whole argument turns on
/// the labels, not the tally.
fn header_label(line: &str, letter: char) -> Option<String> {
    (header_letter(line)? == letter).then(|| {
        line.trim_end()
            .strip_prefix("## \u{a7}")
            .and_then(|r| r.get(letter.len_utf8()..))
            .unwrap_or_default()
            .trim()
            .to_lowercase()
    })
}

/// What one sweep found. Counts only -- see the privacy note above.
#[derive(Default)]
struct Sweep {
    files: usize,
    projects: usize,
    unreadable: usize,
    duplicates: usize,
    clean: usize,
    violations: usize,
    by_rule: BTreeMap<String, usize>,
    rule_specs: BTreeMap<String, usize>,
    /// Specs carrying at least one header of this letter. Per SPEC, not per
    /// header: a spec that writes `## §V` twice has one `§V` section as far
    /// as a claim about the fleet goes.
    letters: BTreeMap<char, usize>,
    /// What the two WRITING verbs would do, measured without writing.
    /// `unstable` is the only one of these that is a DEFECT.
    fmt_refused: usize,
    fmt_over_cap: usize,
    fmt_rewrites: usize,
    fmt_unstable: usize,
    migrate_declined: usize,
    /// The letter whose LABELS are being read, and what they say.
    watched: Option<char>,
    labels: BTreeMap<String, usize>,
}

impl Sweep {
    fn watching(letter: Option<char>) -> Self {
        Self {
            watched: letter,
            ..Self::default()
        }
    }

    /// Read one spec and count it -- unless its bytes were already counted.
    ///
    /// B28: a spec identical to one already read is a COPY, whatever the
    /// directory is called. The name shapes above catch the copies somebody
    /// NAMED as copies; this catches the rest, and there turned out to be
    /// far more of the rest -- one workspace holding hundreds of checkouts
    /// of other projects, none of them named like a backup.
    /// `project` is counted only if the spec SURVIVES, which B31 is the
    /// cost of getting wrong: the denominator was taken over every path
    /// DISCOVERED, while the numerator counted only the distinct texts. A
    /// project whose specs were all duplicates of another's contributed 0
    /// to one side and 1 to the other -- an inflated denominator, which is
    /// precisely what B27 and B28 were filed for.
    fn take(&mut self, path: &Path, project: Option<&Path>, seen: &mut Sieve) {
        let Ok(text) = std::fs::read_to_string(path) else {
            self.unreadable = self.unreadable.saturating_add(1);
            return;
        };
        let fingerprint = digest(&text);
        if seen.texts.contains(&fingerprint) {
            self.duplicates = self.duplicates.saturating_add(1);
            return;
        }
        seen.texts.push(fingerprint);
        seen.credit(project);
        self.add(&text);
    }

    fn add(&mut self, text: &str) {
        self.files = self.files.saturating_add(1);
        self.census(text);
        self.read_labels(text);
        self.rewrite(text);
        // V16 needs a baseline of named records, which is a claim about ONE
        // repo's edits over time. No such baseline exists for somebody
        // else's spec, so the sweep passes none: an empty list means the V16
        // gate is OFF, not that it passed.
        let found = check_spec(text, &[]);
        if found.is_empty() {
            self.clean = self.clean.saturating_add(1);
            return;
        }
        self.violations = self.violations.saturating_add(found.len());
        self.blame(&found);
    }

    /// Findings per rule, and SPECS per rule -- the second is the number a
    /// consumer feels, and the sweep could not answer it until now.
    ///
    /// Every hand-made spec count on this branch drifted: B27, B28, B31, and
    /// V42's own "11 specs", which a recount put at 10. Every count the tool
    /// rendered held. The difference was never care -- it was whether the
    /// tool could answer the question at all.
    fn blame(&mut self, found: &[microlith::Violation]) {
        let mut here: Vec<&str> = Vec::new();
        for v in found {
            bump(&mut self.by_rule, v.rule.clone());
            if !here.contains(&v.rule.as_str()) {
                here.push(&v.rule);
                bump(&mut self.rule_specs, v.rule.clone());
            }
        }
    }

    /// The two WRITING verbs, run over real specs WITHOUT writing anything.
    ///
    /// This is where the expensive bugs have been. B12, B13 and B14 were all
    /// `fmt` merging a construct our own spec does not contain, and all three
    /// passed V1's losslessness proof because a merge is whitespace-only. The
    /// corpus caught every one; the dogfood caught none. Yet the sweep ran
    /// `check` alone until now, which is the half that never had the bugs.
    ///
    /// NOTHING IS WRITTEN, and that is structural rather than careful:
    /// `format_spec` and `migrate_spec` take `&str` and return `String`. Every
    /// write in this crate lives in `cli.rs`. So the corpus is read, never
    /// touched, and no copy of somebody else's repo has to be made to keep it
    /// that way.
    ///
    /// What is counted:
    ///
    /// * REFUSED -- the transform declined, which means V1's proof failed or
    ///   a line is over the cap. A refusal is the tool working.
    /// * REWRITES -- `fmt` would change the file. Expected and not a defect:
    ///   most specs in the wild are hand-wrapped.
    /// * UNSTABLE -- `fmt` twice differs from `fmt` once. That IS a defect
    ///   (V2), and on a real file rather than a fixture.
    /// * DECLINED -- `migrate` found a letter collision it will not touch.
    fn rewrite(&mut self, text: &str) {
        match format_spec(text) {
            // WHY the refusal, not just that there was one. Over the cap is
            // EXPECTED -- other projects never opted into our line limit. A
            // refusal on the LOSSLESSNESS proof is a different animal: the
            // transform would have dropped or changed content on a real
            // file, which is the failure V1 exists to make impossible.
            // Counting them together would hide the second behind 100 of
            // the first.
            Err(why) if why.contains("over the") => {
                self.fmt_over_cap = self.fmt_over_cap.saturating_add(1);
            }
            Err(_) => self.fmt_refused = self.fmt_refused.saturating_add(1),
            Ok(once) => self.rewrote(text, &once),
        }
        if !migrate_declined(text).is_empty() {
            self.migrate_declined = self.migrate_declined.saturating_add(1);
        }
    }

    /// `fmt` returned: did it change anything, and does it change its mind?
    fn rewrote(&mut self, text: &str, once: &str) {
        if once != text {
            self.fmt_rewrites = self.fmt_rewrites.saturating_add(1);
        }
        if format_spec(once).ok().as_deref() != Some(once) {
            self.fmt_unstable = self.fmt_unstable.saturating_add(1);
        }
    }

    /// What the watched letter is called here, if a letter is watched.
    fn read_labels(&mut self, text: &str) {
        let Some(letter) = self.watched else {
            return;
        };
        for label in text.lines().filter_map(|l| header_label(l, letter)) {
            bump(&mut self.labels, label);
        }
    }

    /// Which letters this ONE spec carries, counted once each.
    fn census(&mut self, text: &str) {
        let mut seen: Vec<char> = Vec::new();
        for letter in text.lines().filter_map(header_letter) {
            if !seen.contains(&letter) {
                seen.push(letter);
                bump(&mut self.letters, letter);
            }
        }
    }

    fn report(&self) -> String {
        let letters =
            tally("letters, specs carrying each:", &self.letters, "\u{a7}");
        format!(
            "{}{}{}{}{}",
            self.counts(),
            tally("by rule, findings:", &self.by_rule, ""),
            tally("by rule, specs touched:", &self.rule_specs, ""),
            letters,
            self.label_report()
        )
    }

    /// The denominator and what it splits into. A count with no denominator
    /// names nothing, which is the lesson T15 paid for.
    fn counts(&self) -> String {
        let red = self.files.saturating_sub(self.clean);
        let unreadable = line_if("unreadable", self.unreadable);
        let duplicates = line_if("duplicate copies skipped", self.duplicates);
        format!(
            "specs: {} in {} projects\n{unreadable}{duplicates}clean: {}\nwith violations: {red}\n\
             violations: {}\n{}",
            self.files,
            self.projects,
            self.clean,
            self.violations,
            self.rewrites()
        )
    }

    /// What the writing verbs would do. `fmt unstable` is the line to read:
    /// the others describe the corpus, that one describes a bug in us.
    fn rewrites(&self) -> String {
        format!(
            "fmt would rewrite: {}\nfmt over the cap: {}\n\
             fmt REFUSED its own proof (a defect, V1): {}\n\
             fmt UNSTABLE (a defect, V2): {}\nmigrate declined: {}\n",
            self.fmt_rewrites,
            self.fmt_over_cap,
            self.fmt_refused,
            self.fmt_unstable,
            self.migrate_declined
        )
    }

    /// The labels the watched letter wears, commonest last.
    ///
    /// This is the half a tally cannot answer: `\u{a7}F=5` reads as five
    /// specs agreeing until you see that one of them means feature flags.
    fn label_report(&self) -> String {
        let Some(letter) = self.watched else {
            return String::new();
        };
        let mut out = format!("\u{a7}{letter} is headed:\n");
        for (label, n) in &self.labels {
            let said = if label.is_empty() { "(bare)" } else { label };
            out.push_str(&format!("  {n} x `{said}`\n"));
        }
        out
    }
}

/// Every `SPEC.md` under these roots, once each.
fn specs_under(roots: &[String]) -> Vec<PathBuf> {
    let mut specs = Vec::new();
    for root in roots {
        walk(Path::new(root), &mut specs);
    }
    specs.sort();
    specs.dedup();
    specs
}

/// What has already been counted: distinct texts, and the projects those
/// texts came from.
///
/// One structure so the two cannot drift apart. They did: the projects were
/// tallied over every path DISCOVERED and the specs over every text KEPT,
/// which is how a denominator ends up describing a larger corpus than its
/// numerator (B31).
#[derive(Default)]
struct Sieve {
    texts: Vec<u64>,
    projects: Vec<PathBuf>,
}

impl Sieve {
    /// Count this project, once, and only for a spec that was KEPT.
    fn credit(&mut self, project: Option<&Path>) {
        let Some(project) = project else {
            return;
        };
        if !self.projects.iter().any(|p| p == project) {
            self.projects.push(project.to_path_buf());
        }
    }
}

/// The PROJECT a spec belongs to -- the directory immediately under a root.
///
/// A letter's claim rests on how many INDEPENDENT projects spell it, and one
/// project federated over 19 nested specs is one adopter, not nineteen.
fn project_of(roots: &[String], spec: &Path) -> Option<PathBuf> {
    roots
        .iter()
        .filter_map(|r| spec.strip_prefix(r).ok())
        .find_map(|rest| rest.components().next())
        .map(|c| PathBuf::from(c.as_os_str()))
}

fn sweep_over(roots: &[String], watched: Option<char>) -> Sweep {
    let mut sweep = Sweep::watching(watched);
    let mut seen = Sieve::default();
    for path in &specs_under(roots) {
        let project = project_of(roots, path);
        sweep.take(path, project.as_deref(), &mut seen);
    }
    sweep.projects = seen.projects.len();
    sweep
}

/// A content fingerprint, for telling a second copy from a second adopter.
///
/// `DefaultHasher` rather than a real digest: this decides whether two files
/// on one disk are the same text, not whether an adversary can forge one, and
/// a cryptographic hash would be a dependency in a crate that has none (§C).
fn digest(text: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    h.finish()
}

/// `--label F` asks what `§F` is CALLED across the fleet, which is the
/// question a letter claim turns on. Everything else is a root to walk.
fn watched(args: &[String]) -> Option<char> {
    let at = args.iter().position(|a| a == "--label")?;
    args.get(at.saturating_add(1))?.chars().next()
}

/// The roots, which is every argument `--label` did not account for.
fn roots_in(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--label" {
            rest.next();
        } else {
            out.push(arg.clone());
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let letter = watched(&args);
    let roots = roots_in(&args);
    if roots.is_empty() {
        eprintln!(
            "corpus: give one or more roots to walk, e.g.\n  \
             cargo run --example corpus -- ~/projects\n  \
             cargo run --example corpus -- --label F ~/projects"
        );
        return;
    }
    print!("{}", sweep_over(&roots, letter).report());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// EVERY TEST HERE PINS A BUG THAT HAPPENED. This file produced four
    /// wrong measurements -- B27, B28, B31, B33 -- and every one was
    /// published before anybody noticed, because 530 lines of it had no
    /// tests at all while every number in `FORMAT-EXTENSIONS.md` came from
    /// them. The suite is written against that history rather than against
    /// the function list.
    ///
    /// It needs its own gate step: `cargo nextest run` does NOT run tests
    /// inside an example, so these would have had no runner (V17) -- which
    /// is the same defect one layer out.
    fn temp(name: &str, body: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir()
            .join(format!("microlith-corpus-{name}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        let f = p.join("SPEC.md");
        let _ = std::fs::write(&f, body);
        f
    }

    const SPEC: &str = "## \u{a7}V INVARIANTS\nV1: **a rule.**\n";

    /// B27: `is_copy` knew `backup` and `-bak` and missed three shapes, so
    /// one project's snapshots were counted as separate adopters.
    #[test]
    fn a_snapshot_is_recognised_by_every_shape_that_slipped_through() {
        for name in [
            "foo-backup",
            "foo-bak",
            "sherd--before--rewind",
            "hallucinogen-old",
            "htz-nixos-2025-10-16--11-54",
        ] {
            assert!(is_copy(name), "not seen as a copy: {name}");
        }
    }

    /// ...and the companion, which is the half that keeps it usable: an
    /// ordinary project name is NOT a copy. A predicate that swallowed
    /// everything would have made the denominator zero and still passed the
    /// test above.
    #[test]
    fn an_ordinary_project_name_is_not_a_copy() {
        for name in ["microlith", "itok", "nix-hk", "set-and-setting"] {
            assert!(!is_copy(name), "wrongly seen as a copy: {name}");
        }
    }

    #[test]
    fn a_date_stamp_is_read_anywhere_in_the_name() {
        assert!(looks_dated("2026-07-27-analysis"));
        assert!(looks_dated("htz-nixos-2025-10-16--11-54"));
        assert!(!looks_dated("microlith"));
        assert!(!looks_dated("v0-6-1"), "a version is not a date");
    }

    /// B28: the largest distortion was never NAMED like a copy -- one
    /// workspace held hundreds of checkouts under ordinary directory names.
    /// Identical text is one spec however many paths carry it.
    #[test]
    fn identical_text_is_counted_once() {
        let mut sweep = Sweep::default();
        let mut seen = Sieve::default();
        let a = temp("dup-a", SPEC);
        let b = temp("dup-b", SPEC);
        sweep.take(&a, Some(Path::new("one")), &mut seen);
        sweep.take(&b, Some(Path::new("two")), &mut seen);
        assert_eq!(sweep.files, 1, "the second copy was counted");
        assert_eq!(sweep.duplicates, 1, "and not reported as skipped");
    }

    /// B31, and the reason it took three goes: the project tally ran over
    /// every path DISCOVERED while the spec tally ran over every text KEPT,
    /// so a project whose specs were all duplicates contributed 0 to one
    /// side and 1 to the other. `66 projects` was published; 62 was right.
    #[test]
    fn a_project_is_credited_only_for_a_spec_that_survived() {
        let mut sweep = Sweep::default();
        let mut seen = Sieve::default();
        let a = temp("credit-a", SPEC);
        let b = temp("credit-b", SPEC);
        sweep.take(&a, Some(Path::new("one")), &mut seen);
        sweep.take(&b, Some(Path::new("two")), &mut seen);
        assert_eq!(
            seen.projects.len(),
            1,
            "the duplicate's project was credited anyway"
        );
    }

    /// B33: the sweep reported findings per rule and never SPECS per rule,
    /// so every spec-level claim was hand-made -- and every hand-made count
    /// on that branch drifted. One spec breaking a rule twice is two
    /// findings and ONE spec.
    #[test]
    fn a_rule_broken_twice_in_one_spec_touches_one_spec() {
        let mut sweep = Sweep::default();
        let twice = "## \u{a7}V INVARIANTS\nV1: **a.** see V8 and V9\n";
        sweep.add(twice);
        assert_eq!(sweep.by_rule.get("V13"), Some(&2), "two findings");
        assert_eq!(sweep.rule_specs.get("V13"), Some(&1), "one spec");
    }

    /// The census reads a letter off a header and nothing else. `## §V.2`
    /// and `## §Version` are not section headers.
    #[test]
    fn the_census_reads_a_letter_only_from_a_real_header() {
        assert_eq!(header_letter("## \u{a7}V INVARIANTS"), Some('V'));
        assert_eq!(header_letter("## \u{a7}F \u{2014} Federation"), Some('F'));
        assert_eq!(header_letter("## \u{a7}V2 THINGS"), None);
        assert_eq!(header_letter("### \u{a7}V INVARIANTS"), None);
        assert_eq!(header_letter("V1: a rule"), None);
    }

    #[test]
    fn a_label_is_read_for_the_letter_asked_about() {
        let line = "## \u{a7}F \u{2014} Federation";
        assert_eq!(header_label(line, 'F').as_deref(), Some("— federation"));
        assert_eq!(header_label(line, 'N'), None);
    }

    /// The writing verbs are measured WITHOUT writing: a spec that `fmt`
    /// would rewrite is counted, and the losslessness and idempotence
    /// counters stay at zero on text the transform handles.
    #[test]
    fn a_hard_wrapped_spec_counts_as_a_rewrite_and_no_defect() {
        let mut sweep = Sweep::default();
        sweep.rewrite("## \u{a7}V INVARIANTS\nV1: **a rule**\nwrapped on.\n");
        assert_eq!(sweep.fmt_rewrites, 1);
        assert_eq!(sweep.fmt_refused, 0, "V1's proof must not have failed");
        assert_eq!(sweep.fmt_unstable, 0, "V2 must hold");
    }

    /// A refusal is split by CAUSE, because over the cap is expected of a
    /// project that never opted into our limit and a losslessness refusal
    /// is V1 failing on a real file. Counting them together would hide the
    /// second behind a hundred of the first.
    #[test]
    fn an_over_long_line_is_a_cap_refusal_not_a_proof_refusal() {
        let mut sweep = Sweep::default();
        let long =
            format!("## \u{a7}V INVARIANTS\nV1: **{}**\n", "x".repeat(2000));
        sweep.rewrite(&long);
        assert_eq!(sweep.fmt_over_cap, 1);
        assert_eq!(sweep.fmt_refused, 0, "the cap is not a proof failure");
    }

    #[test]
    fn a_project_is_the_directory_under_the_root() {
        let roots = vec!["/tmp/projects".to_owned()];
        let spec = Path::new("/tmp/projects/thing/nested/SPEC.md");
        assert_eq!(project_of(&roots, spec), Some(PathBuf::from("thing")));
        assert_eq!(project_of(&roots, Path::new("/elsewhere/SPEC.md")), None);
    }

    /// The report states its denominator, and says what it EXCLUDED. A
    /// count that silently dropped rows would be the same defect one layer
    /// up -- a number that looks careful because you cannot see its edges.
    #[test]
    fn the_report_names_the_denominator_and_what_it_skipped() {
        let mut sweep = Sweep::default();
        sweep.files = 5;
        sweep.projects = 2;
        sweep.duplicates = 3;
        let out = sweep.report();
        assert!(out.contains("specs: 5 in 2 projects"), "{out}");
        assert!(out.contains("duplicate copies skipped: 3"), "{out}");
    }
}
