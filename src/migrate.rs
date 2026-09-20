//! `migrate`: bring a spec's section HEADERS to canonical 4.1.0.
//!
//! The other mutation in this crate. `fmt` reshapes without changing content
//! and proves it by whitespace identity (V1); migrate changes content ON
//! PURPOSE, so it needs a different proof and a narrower promise.
//!
//! THE PROMISE: every alphanumeric run of the original header survives,
//! case-insensitively, in the canonical label or in the note written beneath
//! it. Asserted before any write. Case-folding is exactly what makes a
//! case-only rewrite provable, and an em dash is why the proof is about
//! alphanumeric runs rather than bytes -- `\u{a7}I \u{2014} Interfaces`
//! becomes `\u{a7}I INTERFACES` and the dash is punctuation, not content.
//!
//! WHAT IT WILL NOT DO. A letter COLLISION -- a section using `\u{a7}V` for
//! versioning -- is left exactly where it is. Annotating one would keep every
//! character and invert the meaning: a build section filed under a BUGS
//! header, which `check` then holds to `B1|date|cause|fix`. Lossless in
//! bytes, wrong in substance, and it PASSES the proof above. Telling a
//! synonym from a collision needs to know what the section CONTAINS, which is
//! judgement (V6), so it is not inferred -- V27's audited `SYNONYMS` table
//! decides, and absence from it means collision.

use crate::check::{CANONICAL_WORDS, KINDS, SECTIONS, SYNONYMS, is_header_for};

/// What `migrate` would do to one header line.
#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    /// Already canonical.
    Keep,
    /// Case and punctuation only -- nothing to preserve.
    Rewrite(&'static str),
    /// The label carries real text, so the original is kept beneath it.
    Annotate(&'static str, String),
    /// A letter collision. Reported, never touched.
    Leave(String),
}

/// The note written beneath an annotated header.
pub const NOTE: &str = "Migrated by microlith from: ";

/// What would happen to this line, if it is a canonical header at all.
#[must_use]
pub fn plan(line: &str) -> Option<Plan> {
    let (kind, word) = CANONICAL_WORDS
        .into_iter()
        .find(|(kind, _)| is_header_for(line, *kind))?;
    let canonical = canonical_header(kind)?;
    let label = crate::check::label_of(line, kind);
    if !label.to_lowercase().contains(word) {
        return Some(without_the_word(canonical, kind, label));
    }
    Some(with_the_word(canonical, line, label, word))
}

/// The label does not name the concept: bare, synonym, or collision.
fn without_the_word(canonical: &'static str, kind: char, label: &str) -> Plan {
    if label.is_empty() {
        return Plan::Rewrite(canonical);
    }
    let lower = label.to_lowercase();
    if SYNONYMS
        .into_iter()
        .any(|(k, syn)| k == kind && lower.contains(syn))
    {
        return Plan::Annotate(canonical, label.to_owned());
    }
    Plan::Leave(label.to_owned())
}

/// The label names the concept, so the only question is what ELSE it says.
fn with_the_word(
    canonical: &'static str,
    line: &str,
    label: &str,
    word: &str,
) -> Plan {
    if line.trim_end() == canonical {
        return Plan::Keep;
    }
    if residual(label, word).is_empty() {
        return Plan::Rewrite(canonical);
    }
    Plan::Annotate(canonical, label.to_owned())
}

/// What the label says BEYOND the canonical word, as bare alphanumerics.
///
/// `\u{2014} Interfaces` leaves nothing -- a dash is punctuation, so that
/// rewrite discards no content. `\u{2014} Bugs / Known Issues` leaves
/// `knownissues`, which is real text somebody wrote, so it earns a note.
///
/// Drops the whole TOKEN that carries the stem, not the stem's letters.
/// Cutting `interface` out of `Interfaces` leaves `s`, and a stray plural
/// suffix would have been read as content worth preserving -- so every
/// canonical-word header in the plural would have earned a pointless note.
fn residual(label: &str, word: &str) -> String {
    words(label)
        .into_iter()
        .filter(|token| !token.contains(word))
        .collect()
}

fn canonical_header(kind: char) -> Option<&'static str> {
    SECTIONS
        .into_iter()
        .find(|h| h.chars().nth(4) == Some(kind))
}

/// A declaration written in a DIALECT, rewritten canonically.
///
/// Three shapes, one family, all measured in the corpus (T20):
///
/// * `| T1 | x | task | V1 |` -> `T1|x|task|V1` -- 21 specs write §T as a
///   markdown table, and those 1,227 rows open with `|`, so the id grammar
///   never sees them. 16 of those specs PASSED `check` while their whole task
///   section was invisible (B15).
/// * `| V1 | text |` -> `V1: text` -- the id is already explicit.
/// * `4. text` -> `V4: text` in `\u{a7}V` -- the ORDINAL IS THE ID. FORMAT.md's
///   `\u{a7}S.n` says item 4 of `\u{a7}V` IS `\u{a7}V.4`, so this reads the
///   format rather than inventing a mapping.
///
/// `M` is excluded deliberately: a `| M1 | scope | tasks | done-when |` row is
/// a milestone, which the format renders AS a table. Converting it would
/// destroy a legal shape.
fn dialect(line: &str, section: char) -> Option<String> {
    if let Some(cells) = table_cells(line) {
        return from_table(&cells, section);
    }
    ordinal(line, section)
}

/// The cells of a markdown table row, or `None` if it is not one.
///
/// Furniture is rejected here: a separator row has nothing but dashes, and a
/// header row's first cell is not an id.
///
/// Split by [`crate::id::cells`], not by `split('|')`. This was the THIRD
/// reading of the escape rule in-tree and the only one with no escape
/// handling at all. The four-column arm HID it -- that one splits and
/// rejoins on the same character, so the damage cancels -- and the arm that
/// BRANCHES on the count did not: a two-column invariant whose text escapes
/// a pipe counted three cells, missed that arm, and came out as `V1|text`,
/// a `§V` row where the format wants a statement (B36).
fn table_cells(line: &str) -> Option<Vec<String>> {
    let inner = line.trim().strip_prefix('|')?.strip_suffix('|')?;
    if inner.chars().all(|c| matches!(c, '-' | ':' | '|' | ' ')) {
        return None;
    }
    Some(
        crate::id::cells(inner)
            .into_iter()
            .map(|c| c.trim().to_owned())
            .collect(),
    )
}

fn from_table(cells: &[String], section: char) -> Option<String> {
    let first = cells.first()?;
    let id = crate::id::at_line_start(&format!("{first}:"))?;
    if id.kind == 'M' || id.kind != section {
        return None;
    }
    if cells.len() == 2 {
        return Some(format!("{first}: {}", cells.get(1)?));
    }
    Some(cells.join("|"))
}

/// `4. text` under `\u{a7}V` -- the ordinal is the id it already addresses.
fn ordinal(line: &str, section: char) -> Option<String> {
    if section != 'V' {
        return None;
    }
    let digits: String =
        line.chars().take_while(char::is_ascii_digit).collect();
    let rest = line.get(digits.len()..)?.strip_prefix(". ")?;
    (!digits.is_empty()).then(|| format!("V{digits}: {rest}"))
}

/// The migrated text, or the reason it was refused.
///
/// Refusal is the safe direction: a proof that does not hold means the
/// rewrite would drop something, and no output is better than quiet loss.
///
/// # Errors
///
/// If the losslessness proof fails -- which is a bug in this module, not a
/// property of the input, and is why it is asserted rather than assumed.
pub fn migrate(text: &str) -> Result<String, String> {
    let out = rewrite(text);
    let lost = lost_words(text, &out);
    if lost.is_empty() {
        return Ok(out);
    }
    Err(format!("migrate would drop {lost:?} -- refusing to write"))
}

fn rewrite(text: &str) -> String {
    let settled = already_canonical(text);
    let mut out = String::new();
    let mut section = ' ';
    let mut fenced = false;
    for line in text.split_inclusive('\n') {
        emit(&mut out, (&mut section, &mut fenced), line, &settled);
    }
    out
}

/// One line, with the section and fence state it is read in.
fn emit(
    out: &mut String,
    at: (&mut char, &mut bool),
    line: &str,
    settled: &[char],
) {
    let (section, fenced) = at;
    if crate::format::is_fence(line) {
        *fenced = !*fenced;
        out.push_str(line);
        return;
    }
    if let Some(k) = section_of(line) {
        *section = k;
    }
    headed(
        out,
        line,
        plan(line).filter(|_| !*fenced),
        (*section, *fenced, settled),
    );
}

/// The header rewrite if there is one, else the dialect pass.
fn headed(
    out: &mut String,
    line: &str,
    plan: Option<Plan>,
    at: (char, bool, &[char]),
) {
    let ending = line.strip_prefix(line.trim_end()).unwrap_or("");
    match plan {
        Some(Plan::Rewrite(c)) => push(out, c, ending),
        Some(Plan::Annotate(c, was)) => {
            push(out, c, ending);
            push(out, &format!("{NOTE}{was}"), ending);
        }
        _ => converted(out, line, ending, at),
    }
}

/// A dialect declaration rewritten canonically, or the line untouched.
///
/// SCOPED: never inside a fence (B14), and never in a section that already
/// holds canonical declarations -- a half-converted section is worse than an
/// untouched one, and the corpus has 0 mixed sections anyway.
fn converted(
    out: &mut String,
    line: &str,
    ending: &str,
    at: (char, bool, &[char]),
) {
    let (section, fenced, settled) = at;
    if fenced || settled.contains(&section) {
        out.push_str(line);
        return;
    }
    match dialect(line.trim_end(), section) {
        Some(canonical) => push(out, &canonical, ending),
        None => out.push_str(line),
    }
}

/// The section a header opens, if it opens one.
fn section_of(line: &str) -> Option<char> {
    KINDS.into_iter().find(|k| is_header_for(line, *k))
}

/// Sections that ALREADY declare canonically, so nothing there is converted.
fn already_canonical(text: &str) -> Vec<char> {
    let mut out = Vec::new();
    let mut section = ' ';
    for line in text.lines() {
        if let Some(k) = section_of(line) {
            section = k;
        } else if declares(line, section) && !out.contains(&section) {
            out.push(section);
        }
    }
    out
}

fn declares(line: &str, section: char) -> bool {
    crate::id::at_line_start(line).is_some_and(|i| i.kind == section)
}

fn push(out: &mut String, body: &str, ending: &str) {
    out.push_str(body);
    out.push_str(if ending.is_empty() { "\n" } else { ending });
}

/// Alphanumeric runs in the text, lowercased -- the unit the proof counts.
fn words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Words present before the rewrite and gone after it.
///
/// A word SURVIVES if some output word contains it, not if some output word
/// equals it. `\u{a7}I INTERFACE` is a real fleet header, singular, and
/// canonicalising it to `INTERFACES` changes the token -- nothing is lost,
/// but exact equality called it a drop and refused to write.
///
/// The containment reading is not a loosening for convenience: `residual`
/// already decides what counts as the same word this way, and the proof
/// disagreeing with the transform about that is two definitions of one
/// relation inside a single module (V7).
fn lost_words(before: &str, after: &str) -> Vec<String> {
    let kept = words(after);
    let mut lost: Vec<String> = words(before)
        .into_iter()
        .filter(|w| !kept.iter().any(|k| k.contains(w)))
        .collect();
    lost.dedup();
    lost
}

/// One line per header that is not canonical, whether or not it can be fixed.
#[must_use]
pub fn report(text: &str) -> String {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            Some(describe(i.saturating_add(1), &plan(line)?))
        })
        .collect()
}

/// What migrate could NOT do -- everything it refuses to touch.
///
/// Reported separately because it is the one case where a successful run
/// leaves the file still not canonical. Folding it into silence would say
/// the job was finished when the hard half was declined.
///
/// TWO kinds, and the second is why this walks the file rather than mapping
/// over lines: a letter COLLISION is decided by the header alone, but an
/// UNPLACEABLE ROW is decided by the header ABOVE it, so the section and the
/// fence have to be carried along exactly as `rewrite` carries them.
#[must_use]
pub fn unfinished(text: &str) -> String {
    let mut out = String::new();
    let mut at = (' ', false);
    for (i, line) in text.lines().enumerate() {
        note(&mut out, &mut at, i.saturating_add(1), line);
    }
    out
}

/// One line, with the section and fence state it is read in.
///
/// `emit`'s bookkeeping, for the READING pass rather than the writing one --
/// same two pieces of state, tracked the same way, because a report that
/// disagreed with the rewrite about which section a line sits in would name
/// lines the rewrite never considered.
fn note(out: &mut String, at: &mut (char, bool), n: usize, line: &str) {
    let (section, fenced) = at;
    if crate::format::is_fence(line) {
        *fenced = !*fenced;
        return;
    }
    if *fenced {
        return;
    }
    if let Some(k) = section_of(line) {
        *section = k;
    }
    out.push_str(&declined(n, line, *section));
}

/// What this one line costs the run, if anything.
fn declined(n: usize, line: &str, section: char) -> String {
    if let Some(Plan::Leave(was)) = plan(line) {
        return describe(n, &Plan::Leave(was));
    }
    unplaceable(line, section).map_or_else(String::new, |id| unplaced(n, &id))
}

/// A row in a convertible DIALECT that no header can place (V47).
///
/// `dialect` converts a table row only into the section its own letter names,
/// which is right -- a `| T1 |` row is a §T row and writing it anywhere else
/// would invent a claim. But declining it in SILENCE is what let a spec carry
/// two defects and report clean from every verb: an unrecognised heading
/// leaves the rows unparsed, and unparsed rows are never ITEMS, so V11 has
/// nothing to call orphaned either. The two hid each other (B38).
///
/// So the row is still not converted -- where it belongs is the reader's
/// judgement (V6) -- and it is no longer silent.
///
/// `M` is excluded for the reason `from_table` excludes it: a milestone row
/// is a table BY DESIGN, so it is placed exactly where it should be.
fn unplaceable(line: &str, section: char) -> Option<String> {
    let cells = table_cells(line.trim_end())?;
    let first = cells.first()?;
    let id = crate::id::at_line_start(&format!("{first}:"))?;
    if id.kind == 'M' || from_table(&cells, section).is_some() {
        return None;
    }
    Some(first.clone())
}

/// Named with the header that WOULD place it, so the fix is one edit.
fn unplaced(n: usize, id: &str) -> String {
    let kind =
        crate::id::at_line_start(&format!("{id}:")).map_or(' ', |i| i.kind);
    let header = canonical_header(kind).unwrap_or("its own section");
    format!(
        "{n}: `{id}` is a row in a CONVERTIBLE dialect with no `{header}` \
         above it -- unplaceable, so it is left alone\n"
    )
}

fn describe(line: usize, plan: &Plan) -> String {
    match plan {
        Plan::Keep => String::new(),
        Plan::Rewrite(c) => format!("{line}: -> `{c}`\n"),
        Plan::Annotate(c, was) => {
            format!("{line}: -> `{c}`, keeping `{was}` as a note\n")
        }
        Plan::Leave(was) => format!(
            "{line}: `{was}` names a DIFFERENT concept -- the CONTENT must \
             move, so this is left alone\n"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "\
## \u{a7}G GOAL
one line.

## \u{a7}I \u{2014} Interfaces
- `cmd foo`

## \u{a7}V INVARIANTS
V1: **a rule.**
";

    /// Case and punctuation only: rewritten silently, because a dash is
    /// punctuation and `Interfaces` is the canonical word in another case.
    /// Nothing was said that the canonical header does not say.
    #[test]
    fn a_case_or_punctuation_difference_is_rewritten_without_a_note() {
        let out = migrate(SPEC).unwrap_or_default();
        assert!(out.contains("## \u{a7}I INTERFACES"), "{out}");
        assert!(!out.contains(NOTE), "nothing was lost, so no note: {out}");
    }

    /// A label carrying REAL text keeps it. `Known Issues` is somebody's
    /// words, so the canonical header is written and the original preserved.
    #[test]
    fn a_label_with_real_text_is_preserved_beneath_the_canonical_header() {
        let text = "## \u{a7}B \u{2014} Bugs / Known Issues\nB1|d|c|f\n";
        let out = migrate(text).unwrap_or_default();
        assert!(out.contains("## \u{a7}B BUGS\n"), "{out}");
        assert!(
            out.contains(
                "Migrated by microlith from: \u{2014} Bugs / Known Issues"
            ),
            "{out}"
        );
    }

    /// A BARE header gains the word and loses nothing.
    #[test]
    fn a_bare_header_gains_its_word() {
        let out = migrate("## \u{a7}V\nV1: **a rule.**\n").unwrap_or_default();
        assert!(out.contains("## \u{a7}V INVARIANTS"), "{out}");
        assert!(!out.contains(NOTE), "{out}");
    }

    /// A listed SYNONYM is migrated, and the old name is kept.
    #[test]
    fn a_listed_synonym_is_migrated_with_a_note() {
        let out = migrate("## \u{a7}I SURFACES\n- a\n").unwrap_or_default();
        assert!(out.contains("## \u{a7}I INTERFACES"), "{out}");
        assert!(
            out.contains("{NOTE}SURFACES".replace("{NOTE}", NOTE).as_str())
        );
    }

    /// THE ONE IT MUST NOT TOUCH. A collision keeps every character if you
    /// annotate it, and inverts the meaning -- version pinning filed under
    /// INVARIANTS. It passes the byte proof, which is exactly why the proof
    /// cannot be the thing that decides.
    #[test]
    fn a_letter_collision_is_left_exactly_alone() {
        let text = "## \u{a7}V VERSIONING\n- pin nixpkgs\n";
        let out = migrate(text).unwrap_or_default();
        assert_eq!(out, text, "must not rewrite a collision");
        assert!(
            report(text).contains("DIFFERENT concept"),
            "but must report"
        );
    }

    /// THE MEASURED ONE, and the reason V39 named its cost out loud. One
    /// fleet repo heads `\u{a7}F` as `Feature Flags (Flipper)`. Claiming the
    /// letter for FEDERATION made that header a collision the same day, and
    /// a `migrate` that rewrote it would keep every character while turning
    /// a feature-flag section into a declaration of directory edges -- the
    /// byte proof passing on a file whose meaning was inverted.
    ///
    /// Nothing new was written to make this hold: V27's collision rule
    /// covers a new letter the moment the letter is known. This pins that it
    /// does, because inheriting a safety silently is how you find out it
    /// stopped applying only after it has cost somebody their section.
    #[test]
    fn the_new_letters_decline_a_collision_like_any_other() {
        for text in [
            "## \u{a7}F Feature Flags (Flipper)\n- a flag\n",
            "## \u{a7}N NOTES\n- a note\n",
        ] {
            let out = migrate(text).unwrap_or_default();
            assert_eq!(out, text, "must not rewrite: {text}");
            assert!(
                report(text).contains("DIFFERENT concept"),
                "but must report: {text}"
            );
        }
    }

    /// The companion: a header that DOES name the concept migrates, so the
    /// rule above is refusing collisions rather than refusing the letters.
    #[test]
    fn the_new_letters_migrate_when_the_label_names_the_concept() {
        let text = "## \u{a7}F \u{2014} federation\n- an edge\n";
        let out = migrate(text).unwrap_or_default();
        assert!(out.contains("## \u{a7}F FEDERATION"), "{out}");
    }

    /// V2: twice is once. The second run sees a canonical header, so it
    /// writes no second note -- the failure that would make migrate unusable
    /// in a gate or a loop.
    #[test]
    fn migrating_twice_is_migrating_once() {
        let once = migrate(SPEC).unwrap_or_default();
        let twice = migrate(&once).unwrap_or_default();
        assert_eq!(once, twice);
    }

    /// The annotated original round-trips: it is recoverable verbatim from
    /// the note, which is what "preserved" has to mean.
    #[test]
    fn an_annotated_header_round_trips_its_original() {
        let text = "## \u{a7}B \u{2014} Bugs / Known Issues\n";
        let out = migrate(text).unwrap_or_default();
        let back = out
            .lines()
            .find_map(|l| l.strip_prefix(NOTE))
            .unwrap_or_default();
        assert_eq!(back, "\u{2014} Bugs / Known Issues");
    }

    /// A spec already canonical is untouched, byte for byte.
    #[test]
    fn a_canonical_spec_is_left_alone() {
        let text = "## \u{a7}G GOAL\none line.\n\n## \u{a7}V INVARIANTS\n";
        assert_eq!(migrate(text).unwrap_or_default(), text);
        assert_eq!(report(text), "");
    }

    /// The proof counts alphanumeric runs, so it catches a rewrite that
    /// drops a word. Planted by asking the checker directly (V18).
    #[test]
    fn the_proof_notices_a_dropped_word() {
        let before = "## \u{a7}B \u{2014} Bugs / Known Issues\n";
        assert_eq!(
            lost_words(before, "## \u{a7}B BUGS\n"),
            ["known", "issues"]
        );
        assert!(lost_words(before, before).is_empty());
    }

    /// A SINGULAR canonical label is a real fleet header, and canonicalising
    /// it changes the token. The proof must not read that as a loss, or it
    /// refuses a migration that discards nothing -- which it did, once,
    /// against the corpus.
    #[test]
    fn a_singular_label_canonicalises_without_tripping_the_proof() {
        let text = "## \u{a7}I INTERFACE\n- a\n";
        let out = migrate(text).unwrap_or_default();
        assert!(out.contains("## \u{a7}I INTERFACES"), "{out}");
        assert!(!out.contains(NOTE), "nothing lost, so no note: {out}");
    }

    /// ...and the proof still catches a genuine drop, so the relaxation
    /// above cannot be it quietly giving up (V18).
    #[test]
    fn the_proof_still_catches_a_real_drop_after_the_relaxation() {
        let before = "## \u{a7}B \u{2014} Bugs / Known Issues\n";
        assert_eq!(
            lost_words(before, "## \u{a7}B BUGS\n"),
            ["known", "issues"]
        );
    }

    /// B15's shape, planted: 21 fleet specs write §T as a markdown table, so
    /// 1,227 task rows never reached the id grammar and 16 specs passed
    /// `check` with their whole task section invisible.
    #[test]
    fn a_task_table_row_becomes_a_canonical_row() {
        let text = "## \u{a7}T TASKS\n\
            | id | status | task | cites |\n\
            |---|---|---|---|\n\
            | T1 | x | a task | V1 |\n";
        let out = migrate(text).unwrap_or_default();
        assert!(out.contains("\nT1|x|a task|V1\n"), "{out}");
        assert!(out.contains("| id | status"), "furniture stays: {out}");
    }

    /// A two-column table is the invariant dialect: the id is explicit, so
    /// the canonical form is a statement rather than a row.
    #[test]
    fn an_invariant_table_row_becomes_a_statement() {
        let text = "## \u{a7}V INVARIANTS\n\
            | id | invariant |\n\
            |---|---|\n\
            | V1 | a rule worth keeping |\n";
        let out = migrate(text).unwrap_or_default();
        assert!(out.contains("\nV1: a rule worth keeping\n"), "{out}");
    }

    /// The ORDINAL IS THE ID: FORMAT.md's §S.n already says item 4 of §V is
    /// §V.4, so this reads the format rather than inventing a mapping.
    #[test]
    fn a_numbered_invariant_becomes_a_statement() {
        let text = "## \u{a7}V INVARIANTS\n1. first rule\n2. second rule\n";
        let out = migrate(text).unwrap_or_default();
        assert!(out.contains("V1: first rule"), "{out}");
        assert!(out.contains("V2: second rule"), "{out}");
    }

    /// A MILESTONE row is a table by design -- the format renders it that
    /// way. Converting it would destroy a legal shape.
    #[test]
    fn a_milestone_row_is_never_converted() {
        let text = "## \u{a7}T TASKS\n| M1 | scope | T1 | done |\nT1|x|a|V1\n";
        assert_eq!(migrate(text).unwrap_or_default(), text);
    }

    /// A section that ALREADY declares canonically is left alone: half
    /// converted is worse than untouched.
    #[test]
    fn a_mixed_section_is_not_half_converted() {
        let text =
            "## \u{a7}V INVARIANTS\nV1: canonical\n| V2 | a table row |\n";
        assert_eq!(migrate(text).unwrap_or_default(), text);
    }

    /// B14 again: a table inside a FENCE is an example, not a declaration.
    #[test]
    fn a_dialect_row_inside_a_fence_is_left_alone() {
        let text = "## \u{a7}V INVARIANTS\n\n```\n| V1 | an example |\n```\n";
        assert_eq!(migrate(text).unwrap_or_default(), text);
    }

    /// V2: converting twice is converting once.
    #[test]
    fn converting_a_dialect_twice_is_converting_once() {
        let text = "## \u{a7}T TASKS\n| T1 | x | a task | V1 |\n";
        let once = migrate(text).unwrap_or_default();
        assert_eq!(migrate(&once).unwrap_or_default(), once);
    }

    /// `## §T55-PLAN` is a heading, not a section, so migrate has no opinion.
    #[test]
    fn a_heading_that_runs_into_the_letter_is_not_migrated() {
        let text = "## \u{a7}T55-PLAN\nprose.\n";
        assert_eq!(migrate(text).unwrap_or_default(), text);
    }

    /// An unknown letter has no canonical word and is not migrate's business.
    #[test]
    fn an_extension_section_is_untouched() {
        let text = "## \u{a7}D DECISIONS\nprose.\n";
        assert_eq!(migrate(text).unwrap_or_default(), text);
        assert_eq!(plan("## \u{a7}D DECISIONS"), None);
    }

    /// A file with no trailing newline keeps not having one.
    #[test]
    fn a_missing_final_newline_is_not_invented() {
        let out = migrate("## \u{a7}G Goal").unwrap_or_default();
        assert_eq!(out, "## \u{a7}G GOAL\n");
    }

    /// `report` answers "what is NOT canonical", including the headers that
    /// CAN be fixed -- the arm `unfinished` never reaches, because that one
    /// reports only what migrate refuses to touch. Exposed as
    /// `microlith::migrate_report`, so this is public contract.
    #[test]
    fn report_names_a_header_that_would_be_rewritten() {
        let text = "## \u{a7}V\nV1: **a rule.**\n";
        let out = report(text);
        assert!(out.contains("-> `"), "expected a rewrite line, got {out:?}");
        assert!(
            unfinished(text).is_empty(),
            "a fixable header is not unfinished work"
        );
    }

    /// The two-defect file from the report, PLANTED (V18).
    ///
    /// An unrecognised heading and rows in the bracketed dialect. Each rule
    /// that could speak is disarmed by the other: the rows never parse, so
    /// they are never ITEMS, so V11 has nothing to call orphaned -- and the
    /// file with TWO problems was the one that reported clean from `tasks`,
    /// `check` and `migrate --check` alike (B38). `migrate` now says so.
    #[test]
    fn a_dialect_row_under_an_unrecognised_heading_is_reported() {
        let text = "# SPEC\n\n## S.T Tasks\n\n\
            | id | status | task | cites |\n\
            | --- | --- | --- | --- |\n\
            | T1 | x | first task | - |\n\
            | T2 | . | second task | - |\n";
        let out = unfinished(text);
        assert!(out.contains("7: `T1`"), "{out}");
        assert!(out.contains("8: `T2`"), "{out}");
        assert!(out.contains("## \u{a7}T TASKS"), "names the fix: {out}");
        assert_eq!(migrate(text).unwrap_or_default(), text, "left alone");
    }

    /// The same silence under a heading that IS recognised but is not the
    /// row's own. A `| T1 |` row under `\u{a7}B` is an item outliving its
    /// header, and V11 cannot see it for the same reason: it never parses.
    #[test]
    fn a_dialect_row_under_the_wrong_section_is_reported() {
        let text = "## \u{a7}B BUGS\n\n| T1 | x | first | - |\n";
        assert!(unfinished(text).contains("3: `T1`"), "{text}");
    }

    /// Naming the header that WOULD place it is the whole direction, so it
    /// is pinned per letter rather than assumed from the one case above.
    #[test]
    fn the_report_names_the_header_for_the_row_it_found() {
        for (id, header) in [
            ("T1 | x | t | -", "## \u{a7}T TASKS"),
            ("B1 | d | c | f", "## \u{a7}B BUGS"),
            ("R1 | t | f | s", "## \u{a7}R RESEARCH"),
        ] {
            let text = format!("# SPEC\n\n| {id} |\n");
            assert!(unfinished(&text).contains(header), "{text}");
        }
    }

    /// The COMPANION (V18): every row that is placed, or is not a row at
    /// all, stays SILENT. A guard that reported the unplaceable ones by
    /// reporting every table would be indistinguishable from this one on
    /// the test above, and would fire on our own spec's milestones.
    #[test]
    fn a_placed_or_furniture_row_is_not_reported() {
        for text in [
            // converted into its own section, so there is nothing to say
            "## \u{a7}T TASKS\n| T1 | x | first | - |\n",
            // a milestone is a table BY DESIGN (V15)
            "## \u{a7}T TASKS\n| M1 | scope | T1-T3 | done-when |\n",
            // furniture: a header row and a separator declare nothing
            "# SPEC\n\n| id | status | task | cites |\n| --- | --- | --- |\n",
            // a table inside a fence is an EXAMPLE, not a declaration (B14)
            "# SPEC\n\n```\n| T1 | x | first | - |\n```\n",
            // not a table at all
            "# SPEC\n\nT1|x|first|-\n",
        ] {
            assert_eq!(unfinished(text), "", "{text}");
        }
    }

    /// The COMPANION, and it passed before this change too -- recorded
    /// because that is the finding. The four-column arm rejoins on the same
    /// character it split on, so an escape that was cut in half is put back
    /// unharmed and the defect is invisible from here. A guard that cannot
    /// go red is not evidence, and this one says so rather than claiming a
    /// catch it never made.
    #[test]
    fn an_escaped_pipe_in_a_table_row_stays_one_field() {
        let text = "## \u{a7}T TASKS\n\
            | id | status | task | cites |\n\
            |---|---|---|---|\n\
            | T1 | x | `Mechanical`\\|`Judgment` | V1 |\n";
        let out = migrate(text).unwrap_or_default();
        assert!(
            out.contains("\nT1|x|`Mechanical`\\|`Judgment`|V1\n"),
            "{out}"
        );
        let row = out
            .lines()
            .find(|l| l.starts_with("T1|"))
            .unwrap_or_default();
        assert_eq!(crate::id::cells(row).len(), 4, "{row}");
    }

    /// B36, planted on the third splitter. `table_cells` split on EVERY pipe,
    /// so a two-column invariant whose text escapes one counted as THREE
    /// cells, missed the two-column arm, and was rejoined as a `§V` ROW --
    /// `V1|text` where the format wants `V1: text`. The four-column arm hid
    /// it: that one splits and rejoins on the same character, so the damage
    /// cancels and only the arm that BRANCHES on the count can show it.
    #[test]
    fn an_escaped_pipe_in_a_two_column_row_still_becomes_a_statement() {
        let text = "## \u{a7}V INVARIANTS\n\
            | id | invariant |\n\
            |---|---|\n\
            | V1 | `Mechanical`\\|`Judgment` is one taxonomy |\n";
        let out = migrate(text).unwrap_or_default();
        assert!(
            out.contains("\nV1: `Mechanical`\\|`Judgment` is one taxonomy\n"),
            "{out}"
        );
    }
}
