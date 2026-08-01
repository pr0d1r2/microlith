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

use crate::check::{is_header_for, CANONICAL_WORDS, SECTIONS, SYNONYMS};

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
pub const NOTE: &str = "Migrated by nanokit from: ";

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
    let mut out = String::new();
    for line in text.split_inclusive('\n') {
        let ending = line.strip_prefix(line.trim_end()).unwrap_or("");
        match plan(line) {
            Some(Plan::Rewrite(c)) => push(&mut out, c, ending),
            Some(Plan::Annotate(c, was)) => {
                push(&mut out, c, ending);
                push(&mut out, &format!("{NOTE}{was}"), ending);
            }
            _ => out.push_str(line),
        }
    }
    out
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

/// What migrate could NOT do -- the collisions it refuses to touch.
///
/// Reported separately because it is the one case where a successful run
/// leaves the file still not canonical. Folding it into silence would say
/// the job was finished when the hard half was declined.
#[must_use]
pub fn unfinished(text: &str) -> String {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| match plan(line)? {
            Plan::Leave(was) => {
                Some(describe(i.saturating_add(1), &Plan::Leave(was)))
            }
            _ => None,
        })
        .collect()
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
                "Migrated by nanokit from: \u{2014} Bugs / Known Issues"
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
        assert!(out.contains("{NOTE}SURFACES".replace("{NOTE}", NOTE).as_str()));
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
}
