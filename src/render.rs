//! Two renderings of one violation list: prose for a human, JSON for an
//! agent.
//!
//! ONE source, so the two cannot disagree. The JSON carries the same anatomy
//! as the prose plus nothing extra, which is the point: an agent never has to
//! parse a sentence to find out which rule fired or whether the fix is safe
//! to apply unattended.
//!
//! SILENCE IS SUCCESS. Neither renderer emits anything for an empty list, so
//! a clean run costs a human no attention and an agent no tokens -- and
//! output that always appears is output nobody reads, which is how a real
//! failure hides in noise everyone learned to scroll past.
//!
//! The encoder is hand-written because this crate has zero dependencies and
//! keeps them (§C). That is a real constraint rather than a preference, so
//! the escaping is tested rather than trusted: a message containing a quote
//! would otherwise emit JSON that no parser accepts.

use crate::violation::Violation;

/// `path:12: microlith/V13: msg`, then the why and each ranked direction,
/// indented.
///
/// `file:line:` FIRST and a real line in the line slot -- the shape every
/// editor and `grep -n` already jump to. The rule id used to sit where the
/// COLUMN goes, which made an editor jump to line 13 of the file being
/// checked: silent, plausible, wrong. rustc's shape instead, where the id
/// is named beside the message and never inside the coordinates.
///
/// The id arrives QUALIFIED from `Display` (V19), which matters most here:
/// this line names the CONSUMER's file, so an unqualified id reads as one
/// of the consumer's own rules.
#[must_use]
pub fn human(file: &str, violations: &[Violation]) -> String {
    violations.iter().map(|v| one_human(file, v)).collect()
}

fn one_human(file: &str, v: &Violation) -> String {
    // Line 0 is document-scoped -- no coordinate to print, so print none
    // rather than a fake zero an editor would honour.
    let mut out = if v.line == 0 {
        format!("{file}: {v}\n")
    } else {
        format!("{file}:{}: {v}\n", v.line)
    };
    if !v.why.is_empty() {
        out.push_str(&format!("    why: {}\n", v.why));
    }
    for d in &v.directions {
        out.push_str(&format!("    {}: {}\n", d.kind, d.action));
    }
    out
}

/// The same list as JSON: one object, `violations` in the order found.
#[must_use]
pub fn json(file: &str, violations: &[Violation]) -> String {
    if violations.is_empty() {
        return String::new();
    }
    let items: Vec<String> = violations.iter().map(one_json).collect();
    format!(
        "{{\"file\":{},\"violations\":[{}]}}\n",
        quote(file),
        items.join(",")
    )
}

fn one_json(v: &Violation) -> String {
    let dirs: Vec<String> = v.directions.iter().map(direction_json).collect();
    format!(
        "{{\"rule\":{},\"line\":{},\"msg\":{},\"why\":{},\"directions\":[{}]}}",
        quote(&v.rule),
        v.line,
        quote(&v.msg),
        quote(&v.why),
        dirs.join(",")
    )
}

fn direction_json(d: &crate::violation::Direction) -> String {
    format!(
        "{{\"kind\":{},\"action\":{}}}",
        quote(&d.kind.to_string()),
        quote(&d.action)
    )
}

/// A JSON string literal.
///
/// Escapes the two characters that would end or corrupt the literal, plus
/// the control range, which JSON forbids raw. Multi-byte text passes through
/// unescaped: JSON is UTF-8 and `∴` needs no `\u` form, so escaping it would
/// only make the output unreadable to the human who ends up debugging it.
///
/// Shared with `tasks`, which emits the other JSON this crate produces (V7).
/// Two hand-written encoders in a zero-dependency crate is the drift this
/// crate exists to end, at the smallest scale it can appear.
pub(crate) fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::violation::Fix;

    fn sample() -> Vec<Violation> {
        vec![
            Violation::new("V13", "`V99` is cited but never declared")
                .at(12)
                .why("a dangling reference reads as authoritative")
                .try_(Fix::Mechanical, "point it at the rule that was meant")
                .try_(Fix::Judgment, "declare V99"),
        ]
    }

    /// Silence is success, in BOTH renderings. An agent polling a clean spec
    /// should pay nothing for the answer.
    #[test]
    fn a_clean_run_renders_nothing_either_way() {
        assert_eq!(human("SPEC.md", &[]), "");
        assert_eq!(json("SPEC.md", &[]), "");
    }

    #[test]
    fn the_human_rendering_leads_with_file_and_line() {
        let out = human("SPEC.md", &sample());
        assert!(out.starts_with("SPEC.md:12: microlith/V13: "), "{out}");
        assert!(out.contains("\n    why: "), "{out}");
        assert!(out.contains("\n    mechanical: "), "{out}");
        assert!(out.contains("\n    judgment: "), "{out}");
    }

    /// The PLANTED failure: an id in the COLUMN slot. `SPEC.md:V13:12:` sends
    /// an editor to line 13 of a file whose line 13 is unrelated, and says
    /// nothing while doing it. Asserted absent, not assumed.
    #[test]
    fn no_rule_id_sits_where_a_coordinate_goes() {
        let out = human("SPEC.md", &sample());
        assert!(!out.contains("SPEC.md:V"), "id in the line slot: {out}");
        let head = out.lines().next().unwrap_or_default();
        let coords = head.split(": ").next().unwrap_or_default();
        assert_eq!(coords, "SPEC.md:12", "{out}");
    }

    /// A document-scoped violation has no line, so it prints no coordinate
    /// rather than a zero an editor would honour.
    #[test]
    fn a_document_scoped_violation_prints_no_line() {
        let v = vec![Violation::new("V11", "missing section")];
        assert!(
            human("SPEC.md", &v).starts_with("SPEC.md: microlith/V11: "),
            "{}",
            human("SPEC.md", &v)
        );
    }

    /// The JSON carries the same anatomy, with the fix kind as data rather
    /// than as a word inside a sentence.
    #[test]
    fn the_json_carries_the_rule_line_and_fix_kind() {
        let out = json("SPEC.md", &sample());
        assert!(out.contains("\"rule\":\"V13\""), "{out}");
        assert!(out.contains("\"line\":12"), "{out}");
        assert!(out.contains("\"kind\":\"mechanical\""), "{out}");
        assert!(out.contains("\"kind\":\"judgment\""), "{out}");
    }

    /// A document-scoped violation still reports a line, as 0. JSON has no
    /// business omitting a field a consumer indexes on.
    #[test]
    fn a_document_scoped_violation_is_line_zero_in_json() {
        let v = vec![Violation::new("V11", "missing section")];
        assert!(json("SPEC.md", &v).contains("\"line\":0"));
    }

    /// The escaping this crate has to own, having no serde. A quote in a
    /// message would otherwise emit JSON no parser accepts.
    #[test]
    fn a_quote_or_backslash_in_a_message_is_escaped() {
        let v = vec![Violation::new("V1", "a \"quoted\" path C:\\x")];
        let out = json("f", &v);
        assert!(out.contains(r#"a \"quoted\" path C:\\x"#), "{out}");
        assert!(!out.contains("\"quoted\" path"), "raw quote leaked: {out}");
    }

    /// A newline inside a message must not break the one-object-per-run
    /// shape, and a control character must not appear raw.
    #[test]
    fn control_characters_are_escaped() {
        let v = vec![Violation::new("V1", "two\nlines\u{7}bell")];
        let out = json("f", &v);
        assert!(out.contains("two\\nlines"), "{out}");
        assert!(out.contains("\\u0007"), "{out}");
        assert_eq!(out.lines().count(), 1, "one line per run: {out}");
    }

    /// Multi-byte symbols pass through unescaped: JSON is UTF-8, and `\u`
    /// forms would only hurt the human who ends up reading the output.
    #[test]
    fn a_multibyte_symbol_survives_unescaped() {
        let v = vec![Violation::new("V6", "CPU only \u{22a5} inference")];
        assert!(json("f", &v).contains("\u{22a5}"));
    }
}
