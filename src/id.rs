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
/// Anchored at the START of the line: an id mentioned mid-line is a CITATION,
/// not a declaration, and conflating the two would make every rule that cites
/// `V7` look like a second declaration of it.
///
/// V26: a markdown LIST MARKER may precede it. FORMAT.md's example is column
/// zero and that stays canonical, but reading ONLY column zero made `check`
/// report 1,134 dangling citations against invariants declared three lines
/// above (B8) -- 1,097 of 1,750 fleet declarations carry a bullet.
#[must_use]
pub fn at_line_start(line: &str) -> Option<Id> {
    let (rest, bulleted) = past_bullet(line);
    let mut chars = rest.chars();
    let kind = section_kind(chars.next()?)?;
    let rest = chars.as_str();
    let body = alnum_run(rest);
    let terminator = terminator_after(rest, body.len(), bulleted)?;
    let (num, suffix) = split_number(&body)?;
    Some(Id {
        kind,
        num,
        suffix,
        terminator,
    })
}

/// The leading run of letters and digits -- the id's `42` or `30a`.
fn alnum_run(s: &str) -> String {
    s.chars().take_while(char::is_ascii_alphanumeric).collect()
}

/// The line past one markdown list marker, and whether there was one.
///
/// One marker, not a nested indent: `  - V1:` under another bullet is a
/// sub-point of that bullet, not a top-level declaration, and treating it as
/// one would invent declarations out of prose structure.
fn past_bullet(line: &str) -> (&str, bool) {
    for marker in ["- ", "* ", "+ "] {
        if let Some(rest) = line.strip_prefix(marker) {
            return (rest, true);
        }
    }
    (line, false)
}

/// The section letters that open an id. Anything else opens none.
///
/// `R` arrived with FORMAT.md 4.1.0. Its rows are `R1|topic|finding|src` --
/// an ordinary pipe row -- so teaching the grammar the letter is the whole
/// change: V12 (unique ids) and V14 (rows sorted) already work in terms of
/// this parser, and cover `\u{a7}R` the moment it parses.
fn section_kind(c: char) -> Option<char> {
    matches!(c, 'V' | 'T' | 'B' | 'M' | 'R').then_some(c)
}

/// `:` or `|` immediately after the id -- the thing that makes it a
/// DECLARATION rather than a word that happens to start with a section letter.
///
/// A SPACE counts only behind a bullet. 131 fleet declarations are written
/// `- V1 every venue entry has...`, with no colon at all, and the bullet is
/// what distinguishes those from a prose line opening `V1 is broken` -- which
/// stays unrecognised at column zero, exactly as before, so widening the
/// grammar cannot invent a declaration where none was read yesterday.
fn terminator_after(rest: &str, at: usize, bulleted: bool) -> Option<char> {
    rest.get(at..)?
        .chars()
        .next()
        .filter(|c| matches!(c, ':' | '|') || (bulleted && *c == ' '))
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

    /// V26, planted: the shape that made `check` report 1,134 dangling
    /// citations against invariants that were right there (B8).
    #[test]
    fn a_bulleted_declaration_is_recognised() {
        for (line, want) in [
            ("- V1: agent sandboxed", ('V', 1)),
            ("* V2: another marker", ('V', 2)),
            ("+ V3: a third", ('V', 3)),
            // No colon at all -- 131 fleet declarations look like this.
            ("- V4 every venue entry has a name", ('V', 4)),
        ] {
            let got = at_line_start(line).map(|i| (i.kind, i.num));
            assert_eq!(got, Some(want), "{line}");
        }
    }

    /// The companion, and the reason the space terminator is gated on the
    /// bullet: prose opening with a section letter is still not a declaration.
    /// Widening the grammar must not invent one where none was read before.
    #[test]
    fn prose_opening_with_an_id_is_still_not_a_declaration() {
        assert_eq!(at_line_start("V1 is broken and needs a rewrite"), None);
        assert_eq!(at_line_start("V1 every venue entry has a name"), None);
    }

    /// One marker, not an indent: a nested bullet is a sub-point of the
    /// bullet above it, so reading it as a top-level declaration would
    /// manufacture ids out of prose structure.
    #[test]
    fn an_indented_bullet_is_not_a_declaration() {
        assert_eq!(at_line_start("  - V1: a sub-point"), None);
    }

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
