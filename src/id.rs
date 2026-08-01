//! The id grammar, defined ONCE.
//!
//! `V42:` · `T30a|` · `B12|` · `M1|` -- a section id opening its own line.
//! Both the formatter and the checker need to recognise these, and they must
//! agree exactly: `format` uses it to decide what OPENS a statement (V4), and
//! `check` uses it to decide what DECLARES an id (V12).
//!
//! Two parsers for one grammar is the defect this crate exists to end (V7),
//! one level down from the copies it was built to remove. `T30a|` is a real
//! row, so "a digit then a terminator" is not enough, and a checker that
//! disagreed with the formatter about that would report violations the
//! formatter had just created.

/// A section id at the start of a line.
#[derive(Debug, PartialEq, Eq)]
pub struct Id {
    /// `V`, `T`, `B` or `M`.
    pub kind: char,
    /// The numeric part. `T30a` is 30.
    pub num: u32,
    /// Anything after the digits. `T30a` is `"a"`, `T30` is `""`.
    pub suffix: String,
    /// `:` for invariants, `|` for rows.
    pub terminator: char,
}

impl Id {
    /// `V42` -- the id as it is written and cited.
    #[must_use]
    pub fn label(&self) -> String {
        format!("{}{}{}", self.kind, self.num, self.suffix)
    }

    /// The sort key: a suffixed id RIDES its base (V14). `T30a` sorts after
    /// `T30` and before `T31`, which lexical ordering gets wrong.
    #[must_use]
    pub fn sort_key(&self) -> (u32, String) {
        (self.num, self.suffix.clone())
    }
}

/// The id opening `line`, if it has one.
///
/// Deliberately anchored at the START of the line: an id mentioned mid-line is
/// a CITATION, not a declaration, and conflating the two would make every rule
/// that cites `V7` look like a second declaration of it.
#[must_use]
pub fn at_line_start(line: &str) -> Option<Id> {
    let mut chars = line.chars();
    let kind = section_kind(chars.next()?)?;
    let rest = chars.as_str();
    let body: String = rest
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .collect();
    let terminator = terminator_after(rest, body.len())?;
    let (num, suffix) = split_number(&body)?;
    Some(Id {
        kind,
        num,
        suffix,
        terminator,
    })
}

/// The four section letters. Anything else opens no id.
fn section_kind(c: char) -> Option<char> {
    matches!(c, 'V' | 'T' | 'B' | 'M').then_some(c)
}

/// `:` or `|` immediately after the id -- the thing that makes it a
/// DECLARATION rather than a word that happens to start with a section letter.
fn terminator_after(rest: &str, at: usize) -> Option<char> {
    rest.get(at..)?
        .chars()
        .next()
        .filter(|c| matches!(c, ':' | '|'))
}

/// `30a` -> `(30, "a")`. An empty or non-numeric lead fails the parse, which
/// is what rejects `Vx:`.
fn split_number(body: &str) -> Option<(u32, String)> {
    let digits: String =
        body.chars().take_while(char::is_ascii_digit).collect();
    let num = digits.parse().ok()?;
    Some((num, body.get(digits.len()..).unwrap_or("").to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_real_id_shape_parses() {
        for (line, want) in [
            ("V42: a rule", ('V', 42, "")),
            ("T30a|x|task|V1", ('T', 30, "a")),
            ("B12|2026-08-01|cause|fix", ('B', 12, "")),
            ("M1|scope", ('M', 1, "")),
        ] {
            let got =
                at_line_start(line).map(|i| (i.kind, i.num, i.suffix.clone()));
            let (kind, num, suffix) = want;
            assert_eq!(got, Some((kind, num, suffix.to_owned())), "{line}");
        }
    }

    /// The guard cannot pass by accepting everything either.
    #[test]
    fn a_non_id_is_rejected() {
        for line in [
            "Vx: not a numbered id",
            "already use, even when",
            "",
            "# a header",
            "| M1 | a table row |",
            "V42 no terminator",
        ] {
            assert!(at_line_start(line).is_none(), "accepted: {line:?}");
        }
    }

    /// V14: `T30a` rides `T30`. Lexically `"T30a" < "T4"`, which is the wrong
    /// answer and the reason the key is (number, suffix) rather than the text.
    #[test]
    fn a_suffixed_id_sorts_after_its_base() {
        let key = |l: &str| at_line_start(l).map(|i| i.sort_key());
        assert!(key("T30|") < key("T30a|"));
        assert!(key("T30a|") < key("T31|"));
        assert!(key("T4|") < key("T30|"), "numeric, not lexical");
    }

    #[test]
    fn the_label_round_trips() {
        assert_eq!(
            at_line_start("T30a|x").map(|i| i.label()).as_deref(),
            Some("T30a")
        );
        assert_eq!(
            at_line_start("V7: x").map(|i| i.label()).as_deref(),
            Some("V7")
        );
    }
}
