//! The anatomy of a failure.
//!
//! A violation is a STRUCTURE, not a sentence. Ported from the host checker
//! this crate replaces, because the shape is the part T7 nearly deleted by
//! mistake (B4) and because a flat string is only usable by a human reading
//! carefully, which is the smaller half of the audience.
//!
//! Four things a caller needs and cannot recover from prose:
//!
//! * `rule` -- a stable id to match on. Prose is NOT stable and is not meant
//!   to be: the wording of a diagnostic should improve without breaking
//!   every caller that keyed off it.
//! * `line` -- where. 0 means document-scoped, because "this file has no §B
//!   header" has no line to point at and inventing one would be a lie.
//! * `why` -- one clause on what goes wrong if it stands. A rule id tells an
//!   agent what fired; this tells a reader whether they care.
//! * `directions` -- what to do, RANKED, correct fix first.

use std::fmt;

/// Whether a direction is safe to apply without judgement.
///
/// This distinction is the reason the type exists. A `Mechanical` fix is
/// deterministic and reversible, so an agent may take it unattended. A
/// `Judgment` fix accepts a regression or changes intent -- lowering a
/// threshold, deleting a rule, widening a cap -- and an agent that applies
/// one blindly has SILENCED the guardrail rather than fixed the defect,
/// while leaving a green gate behind to say so.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fix {
    /// Deterministic and reversible. Safe to apply unattended.
    Mechanical,
    /// Accepts a regression or changes intent. Needs a human, or an explicit
    /// instruction naming this specific trade.
    Judgment,
}

impl fmt::Display for Fix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Mechanical => "mechanical",
            Self::Judgment => "judgment",
        })
    }
}

/// One possible next action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Direction {
    pub kind: Fix,
    pub action: String,
}

/// A format rule violation, with everything a caller needs to act.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// Stable id, e.g. `V13`. Match on this, never on `msg`.
    pub rule: String,
    /// 1-based line, or 0 when the rule is document-scoped.
    pub line: usize,
    /// What was found.
    pub msg: String,
    /// One clause: what goes wrong if this stands.
    pub why: String,
    /// Ranked, correct fix FIRST, escape hatch last.
    pub directions: Vec<Direction>,
}

impl Violation {
    /// A document-scoped violation: no line to point at.
    #[must_use]
    pub fn new(rule: &str, msg: impl Into<String>) -> Self {
        Self {
            rule: rule.to_owned(),
            line: 0,
            msg: msg.into(),
            why: String::new(),
            directions: Vec::new(),
        }
    }

    /// Place it. 1-based, matching what an editor and `grep -n` both show.
    #[must_use]
    pub fn at(mut self, line: usize) -> Self {
        self.line = line;
        self
    }

    #[must_use]
    pub fn why(mut self, why: impl Into<String>) -> Self {
        self.why = why.into();
        self
    }

    /// Append a direction. CALL ORDER IS THE RANKING -- the correct fix is
    /// added first and any escape hatch last, so a caller taking
    /// `directions[0]` takes the right one.
    #[must_use]
    pub fn try_(mut self, kind: Fix, action: impl Into<String>) -> Self {
        self.directions.push(Direction {
            kind,
            action: action.into(),
        });
        self
    }

    /// Whether any direction may be applied unattended.
    #[must_use]
    pub fn is_mechanical(&self) -> bool {
        self.directions.iter().any(|d| d.kind == Fix::Mechanical)
    }
}

impl fmt::Display for Violation {
    /// `V13:12: msg` -- the `file:line:` shape every editor already jumps to.
    /// The line is omitted when it is 0 rather than printed as a fake zero.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {}", self.rule, self.msg)
        } else {
            write!(f, "{}:{}: {}", self.rule, self.line, self.msg)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_scoped_violation_omits_the_line() {
        let v = Violation::new("V11", "missing section");
        assert_eq!(v.line, 0);
        assert_eq!(v.to_string(), "V11: missing section");
    }

    /// The `file:line:` shape, so an editor and `grep -n` both jump to it.
    #[test]
    fn a_placed_violation_prints_its_line() {
        let v = Violation::new("V13", "`V99` never declared").at(12);
        assert_eq!(v.to_string(), "V13:12: `V99` never declared");
    }

    /// Call order IS the ranking, so `directions[0]` is the correct fix and
    /// an escape hatch can never be reached first by accident.
    #[test]
    fn directions_keep_the_order_they_were_added_in() {
        let v = Violation::new("V5", "over the cap")
            .why("a long line re-sends whole on every edit")
            .try_(Fix::Mechanical, "split the statement")
            .try_(Fix::Judgment, "raise the cap");
        assert_eq!(v.directions.first().map(|d| d.kind), Some(Fix::Mechanical));
        assert_eq!(v.directions.get(1).map(|d| d.kind), Some(Fix::Judgment));
        assert!(v.is_mechanical());
    }

    /// A violation whose only way out is a judgement call says so, which is
    /// what stops an agent applying it unattended.
    #[test]
    fn a_judgment_only_violation_is_not_mechanical() {
        let v = Violation::new("V16", "a record was edited away").try_(
            Fix::Judgment,
            "restore it, or record why it no longer applies",
        );
        assert!(!v.is_mechanical());
    }

    #[test]
    fn a_fix_kind_renders_for_both_humans_and_json() {
        assert_eq!(Fix::Mechanical.to_string(), "mechanical");
        assert_eq!(Fix::Judgment.to_string(), "judgment");
    }
}
