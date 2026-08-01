//! `nanokit` -- the cavekit SPEC format, enforced.
//!
//! One implementation of the format rules, callable as a library, so a
//! consumer embeds them instead of re-porting them (V7). That is the whole
//! reason this crate exists: two hand-maintained copies of one rule set
//! disagreed, both gates stayed green, and 88 tasks belonged to no
//! milestone.
//!
//! CPU only -- no inference, no network (V6). Every operation is a
//! deterministic function of the text it is given.

pub mod anchors;
pub mod check;
pub mod derive;
pub mod format;
pub mod id;
pub mod render;
pub mod violation;

/// What a run produced: streams and an exit code. No I/O, so the whole
/// command surface is testable without spawning the binary.
#[derive(Debug, PartialEq, Eq)]
pub struct Output {
    pub out: String,
    pub err: String,
    pub code: u8,
}

impl Output {
    #[must_use]
    pub fn ok(out: String) -> Self {
        Self {
            out,
            err: String::new(),
            code: 0,
        }
    }

    /// A violation the caller asked us to gate on: exit 1 (V10).
    #[must_use]
    pub fn drift(err: String) -> Self {
        Self {
            out: String::new(),
            err,
            code: 1,
        }
    }

    /// A usage error: exit 2.
    #[must_use]
    pub fn usage(err: String) -> Self {
        Self {
            out: String::new(),
            err,
            code: 2,
        }
    }
}

/// The formatted text, or the reason it must not be written.
///
/// V1's proof runs HERE, before any caller can persist the result -- so a
/// lossy transform is impossible to write out by mistake rather than
/// merely discouraged.
///
/// # Errors
/// When the transform would change content, or a line exceeds the cap.
pub fn format_spec(text: &str) -> Result<String, String> {
    let out = format::unwrap_wraps(text);
    if !format::is_lossless(text, &out) {
        return Err("refusing to write: the transform changed content, \
                    not just whitespace"
            .to_owned());
    }
    within_cap(&out)?;
    Ok(out)
}

/// Every structural violation in `text`, in rule order.
///
/// V11-V15 are computable from the text alone. V16 needs a baseline of named
/// records, which is a claim about edits over time rather than about this
/// file, so the caller supplies it -- empty means the V16 gate is off, not
/// that it passed.
#[must_use]
pub fn check_spec(
    text: &str,
    records: &[check::Record],
) -> Vec<violation::Violation> {
    let mut out = check::sections_ordered(text);
    out.extend(check::ids_unique(text));
    out.extend(check::citations_resolve(text));
    out.extend(check::rows_sorted(text));
    out.extend(check::tasks_in_one_milestone(text));
    out.extend(check::statuses_valid(text));
    out.extend(check::records_survive(text, records));
    out
}

/// V5: an over-long line names its place, so the reader can go there.
fn within_cap(text: &str) -> Result<(), String> {
    match format::over_cap(text, format::MAX_LINE).first() {
        Some((line, len)) => Err(format!(
            "line {line} is {len} chars, over the {} cap -- split the \
             statement; raising the cap is a reviewed decision",
            format::MAX_LINE
        )),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_returns_the_unwrapped_text() {
        let got = format_spec("V1: a rule\nwrapped\n").unwrap_or_default();
        assert_eq!(got, "V1: a rule wrapped\n");
    }

    /// V5: an over-long line is refused with its number and its length,
    /// rather than written out and discovered later.
    #[test]
    fn an_over_long_line_is_refused_with_its_place() {
        let long = format!("V1: {}\n", "x".repeat(format::MAX_LINE));
        let err = format_spec(&long).err().unwrap_or_default();
        assert!(err.contains("line 1"), "names the line: {err}");
        assert!(err.contains("over the"), "names the rule: {err}");
    }

    #[test]
    fn already_formatted_text_is_unchanged() {
        let src = "# h\n\nV1: a rule\nV2: another\n";
        assert_eq!(format_spec(src).ok(), Some(src.to_owned()));
    }
}
