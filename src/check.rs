//! The structural rules (V11-V16), as pure functions over `&str`.
//!
//! Each returns the violations it found, empty meaning clean, so a consumer
//! CALLS the rule instead of re-porting it (V7) and the binary is a thin shell
//! over the same code the tests exercise.
//!
//! These are STRUCTURAL. They cannot prove a rewrite preserved meaning, and
//! they are not meant to: what they catch is the dangerous class a byte count
//! cannot see -- a vanished invariant, a citation pointing at nothing, a
//! section that lost its header. That class is what let 88 tasks belong to no
//! milestone while two gates stayed green.
//!
//! Restated here rather than referenced. The first consumer carries a ported
//! copy of these rules today, and T7 deletes it; a rule whose evidence lived
//! only in the copy would lose that evidence with it (V19).

use crate::id::{Id, at_line_start, cells};
use crate::violation::{Fix, Violation};

/// The section LETTER of each entry in `SECTIONS`, in the same order.
///
/// Held alongside rather than read out of the header string: the letter is
/// what the checker matches on now that labels vary across the fleet, and
/// digging it out by character offset would make the position of a word in
/// prose load-bearing.
pub const KINDS: [char; 9] = ['G', 'F', 'N', 'C', 'I', 'R', 'V', 'T', 'B'];

/// The letters the VENDORED `FORMAT.md` defines, at the revision
/// `.format-upstream` pins.
///
/// Held so the EXTENSION set can be COMPUTED -- `KINDS` minus these -- rather
/// than listed a second time (V7). A second list is a thing to forget: the
/// next letter this copy adds would be an extension nothing knew was one,
/// and V40's document would quietly stop describing the format.
///
/// It moves only when the vendored file does, which is the same commit that
/// moves `rev` and `sha256` in `.format-upstream`.
pub const UPSTREAM_KINDS: [char; 7] = ['G', 'C', 'I', 'R', 'V', 'T', 'B'];

/// FORMAT.md fixes the sections and their order, and V39 adds two.
///
/// A SUPERSET of the vendored list, which is the established shape here: V8
/// already splits the vendored reference from the normative `\u{a7}V`, and
/// V13, V14 and V16 have no upstream counterpart either. Forking vendored
/// bytes to carry `\u{a7}F` and `\u{a7}N` would diverge every consumer's
/// copy from upstream to buy nothing, so T26b routes them upstream instead.
///
/// `\u{a7}` is the section sign, written as an escape so this source stays
/// ASCII -- the runtime string is identical either way.
pub const SECTIONS: [&str; 9] = [
    "## \u{a7}G GOAL",
    // V39's pair, and they rank HERE rather than at the end: the edges a
    // directory declares are STRUCTURE, so a reader meets them before the
    // constraints that are written in their terms. Optional like every other
    // section (V11) -- a spec that spans no tree carries neither.
    "## \u{a7}F FEDERATION",
    "## \u{a7}N NAV",
    "## \u{a7}C CONSTRAINTS",
    "## \u{a7}I INTERFACES",
    // 4.1.0's addition: optional, and present only if `/research` ran. It
    // needs no rule of its own -- `R1|topic|finding|src` is a pipe row, so
    // V12 and V14 cover it the moment the id grammar knows the letter.
    "## \u{a7}R RESEARCH",
    "## \u{a7}V INVARIANTS",
    "## \u{a7}T TASKS",
    "## \u{a7}B BUGS",
];

/// The word each canonical letter's header must CARRY (V27).
///
/// Singular stems, so `Bugs`, `bug log` and `— Bugs / Known Issues` all
/// satisfy `B` -- the rule is about the concept being NAMED, not about
/// matching a string. Qualifiers may follow freely.
///
/// `nav` is the stem `Nav`, `NAV` and `Navigation` share. `federation` is
/// the whole word rather than a stem, and deliberately: `federated` is an
/// adjective a dozen sections could wear, while the noun names this one
/// thing. V39 fixes both words, so widening either is a spec edit.
pub const CANONICAL_WORDS: [(char, &str); 9] = [
    ('G', "goal"),
    ('F', "federation"),
    ('N', "nav"),
    ('C', "constraint"),
    ('I', "interface"),
    ('R', "research"),
    ('V', "invariant"),
    ('T', "task"),
    ('B', "bug"),
];

/// The canonical spelling of the SUPERSESSION marker (V41).
///
/// A rule that has been replaced stays in the file -- ids are never reused
/// (V12) and citations to it must still resolve (V13) -- so retirement is a
/// MARK on the line rather than a deletion. This is the word that mark
/// carries, and the parser matches exactly it.
pub const SUPERSEDED_BY: &str = "superseded by";

/// Every marker this format defines beyond the vendored one.
///
/// A section is addressed by its LETTER; a marker is addressed by its WORDS,
/// and both need a canonical spelling for the same reason: two repos writing
/// `[superseded by V2]` and `[dead: see V2]` mean one thing that nothing
/// mechanical can find twice. Held as a list so V40's document renders from
/// what the parser reads rather than from a second copy of it.
pub const MARKERS: [&str; 1] = [SUPERSEDED_BY];

/// Labels that mean the canonical thing under another name.
///
/// AUDITED, one row per entry, and deliberately tiny. A label here is a
/// SYNONYM: the section holds what the letter promises, so the repair is to
/// write the canonical word and keep the old label in a migration note --
/// mechanical, and lossless.
///
/// What is NOT here, and must not be added: `versioning`, `testing` and
/// `build`. Those name DIFFERENT CONCEPTS, and the fleet repos using them for
/// `\u{a7}V`, `\u{a7}T` and `\u{a7}B` mean them literally. No header rewrite
/// can fix that, because the CONTENT has to move to another section, so they
/// fail closed as judgement -- which is what "fails closed" has to mean here:
/// absent from the table is treated as a collision, never as a synonym
/// nobody got round to listing.
pub const SYNONYMS: [(char, &str); 1] = [('I', "surface")];

/// The kinds written as PIPE ROWS, each with exactly four fields.
///
/// `V` is excluded because an invariant is a `V1:` statement rather than a
/// row, and `R` joined `T` and `B` at 4.1.0 for the reason they are here:
/// `R1|topic|finding|src` is the same shape.
///
/// One home rather than a literal in each rule that needs it (V7) -- there
/// are two now, and V42 arrived because the second one had to agree with
/// the first about what a row even is.
pub const ROW_KINDS: [char; 3] = ['T', 'B', 'R'];

/// The kinds that DECLARE an addressable item, in report order.
///
/// `G`, `C` and `I` are prose and bullets with no ids, so they appear in
/// `KINDS` -- which is about section ORDER -- but never here.
pub const ITEM_KINDS: [char; 4] = ['V', 'T', 'B', 'R'];

/// Every id DECLARED in the text, of one kind, in the order they appear.
#[must_use]
pub fn declared(text: &str, kind: char) -> Vec<Id> {
    declared_at(text, kind)
        .into_iter()
        .map(|(_, id)| id)
        .collect()
}

/// The same, each with its 1-based line, so a violation can say WHERE.
#[must_use]
pub fn declared_at(text: &str, kind: char) -> Vec<(usize, Id)> {
    text.lines()
        .enumerate()
        .filter_map(|(i, l)| Some((i.saturating_add(1), at_line_start(l)?)))
        .filter(|(_, id)| id.kind == kind)
        .collect()
}

/// Every `V<n>` CITED -- that is, mentioned OUTSIDE backticks (V13).
///
/// Inside backticks it is a literal: `grep V47` is an example command, and
/// `itok's V82` is another repo's namespace, which V19 requires be written
/// exactly that way. Both were reported as dangling citations on this
/// checker's first real run (B3), and the rule's own illustration of a
/// dangling reference was among them.
///
/// The boundary is READ OFF the format rather than invented here: FORMAT.md
/// already reserves backticks for verbatim text. A boundary invented in the
/// checker is one the next consumer re-derives differently.
///
/// Splitting on non-alphanumerics rather than whitespace, so `(V21,V22)` and
/// `V13.` are found: a citation is rarely followed by a space.
#[must_use]
pub fn cited(text: &str) -> Vec<String> {
    outside_backticks(text)
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| is_invariant_ref(t))
        .map(str::to_owned)
        .collect()
}

/// Every citation with the 1-based line it appears on.
///
/// Per-line rather than whole-file, so a dangling reference can be pointed
/// at. The backtick boundary (V13) is applied line by line, which is a
/// deliberate narrowing: a code span never legitimately spans a line here,
/// and scoping it per line stops one stray backtick suppressing citations
/// for the rest of the file.
#[must_use]
pub fn cited_at(text: &str) -> Vec<(usize, String)> {
    text.lines()
        .enumerate()
        .flat_map(|(i, line)| {
            cited(line)
                .into_iter()
                .map(move |c| (i.saturating_add(1), c))
        })
        .collect()
}

/// The text with every backticked span removed.
///
/// An unclosed backtick swallows the rest of the text, which is the safe
/// direction: it can only SUPPRESS citations, never invent one, so the
/// failure is a check that misses rather than a check that lies.
#[must_use]
pub fn outside_backticks(text: &str) -> String {
    text.split('`').step_by(2).collect::<Vec<_>>().join(" ")
}

fn is_invariant_ref(token: &str) -> bool {
    match token.strip_prefix('V') {
        Some(n) => !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()),
        None => false,
    }
}

/// V11: sections are ORDERED, and no item outlives its header.
///
/// ABSENCE IS LEGAL. This demanded all six until the corpus was measured:
/// across 51 fleet specs only 41 carry `\u{a7}G` and 46 carry `\u{a7}T`, and
/// FORMAT.md 4.1.0 says a section may be absent but is never reordered. The
/// old rule rejected specs that were legal, which is the shape B4 named --
/// a rule generalised from n=2 without measuring.
///
/// What the old rule was FOR survives as `orphaned_items`. "A lost header
/// silently unnames every item under it" is a real failure, but presence was
/// the wrong test for it: a spec with no tasks needs no `\u{a7}T`, while a
/// spec with `T1|` rows and no `\u{a7}T` has exactly the defect. Dropping
/// presence without that replacement would leave V11 unable to catch the one
/// thing it was written for.
///
/// UNKNOWN letters are tolerated -- `\u{a7}D`, `\u{a7}E`, `\u{a7}O`,
/// `\u{a7}P` and `\u{a7}X` are all in fleet use. Only the KNOWN letters are
/// ordered against each other; an extension between them is not the
/// checker's business. `\u{a7}F` and `\u{a7}N` left that list when V39
/// claimed them: tolerance is exactly what would have let a second reader
/// spend either letter on a different concept.
///
/// Document-scoped: a missing header has no line to point at, and pointing
/// at where it OUGHT to be would be a guess dressed as a fact.
#[must_use]
pub fn sections_ordered(text: &str) -> Vec<Violation> {
    let mut out = misplaced(&present(text));
    out.extend(orphaned_items(text));
    out
}

/// Every section that IS here, as `(position, canonical rank)`, in the order
/// a reader meets them.
fn present(text: &str) -> Vec<(usize, usize)> {
    let mut found: Vec<(usize, usize)> = KINDS
        .into_iter()
        .enumerate()
        .filter_map(|(rank, kind)| Some((section_at(text, kind)?, rank)))
        .collect();
    found.sort_unstable();
    found
}

/// The sections whose canonical rank goes BACKWARDS as you read down.
///
/// Walking in reading order and blaming the section that arrives too late is
/// what keeps the message honest. The old scan walked in CANONICAL order and
/// compared positions, so a single section placed at the end made every
/// section before it look early. One fleet spec carries `\u{a7}R` after
/// `\u{a7}B`, and that single misplacement was reported as THREE -- against
/// `\u{a7}V`, `\u{a7}T` and `\u{a7}B`, none of which had moved.
///
/// Third time this shape has cost something (B8, B9): a rule that is right
/// about WHETHER and wrong about WHICH sends every reader to the wrong line.
fn misplaced(found: &[(usize, usize)]) -> Vec<Violation> {
    let mut out = Vec::new();
    let mut highest = 0usize;
    for (_, rank) in found {
        match SECTIONS.get(*rank) {
            Some(header) if *rank < highest => {
                out.push(misordered_section(header))
            }
            _ => highest = *rank,
        }
    }
    out
}

/// Where `## \u{a7}<kind>` starts, whatever LABEL follows it.
///
/// Label-agnostic on purpose. Only 16 of 51 fleet specs spell the header
/// exactly as `SECTIONS` does; 9 write `## \u{a7}V Invariants` and 7 write
/// `## \u{a7}V — Invariants`. Matching the full string reported those as a
/// MISSING section while the section sat right there -- B8's defect again,
/// a true rule delivering a false message.
///
/// So V11 asks only whether the section EXISTS and where. Whether its label
/// carries the canonical word is a separate rule with a separate remedy
/// (T17), and keeping them apart is what lets each say something true.
fn section_at(text: &str, kind: char) -> Option<usize> {
    let mut at = 0usize;
    for line in text.split_inclusive('\n') {
        if is_header_for(line, kind) {
            return Some(at);
        }
        at = at.saturating_add(line.len());
    }
    None
}

/// `## \u{a7}V`, `## \u{a7}V INVARIANTS`, `## \u{a7}V — Invariants` -- yes.
/// `## \u{a7}T55-PLAN` -- no: the letter must not run into a word or digit,
/// or a section id in a heading would read as the section itself.
#[must_use]
pub fn is_header_for(line: &str, kind: char) -> bool {
    let Some(rest) = line.strip_prefix("## \u{a7}") else {
        return false;
    };
    let mut chars = rest.chars();
    chars.next() == Some(kind)
        && chars.next().is_none_or(|c| !c.is_ascii_alphanumeric())
}

/// The canonical header an id of this kind belongs under.
fn header_for(kind: char) -> Option<&'static str> {
    KINDS
        .into_iter()
        .zip(SECTIONS)
        .find(|(k, _)| *k == kind)
        .map(|(_, h)| h)
}

/// V11's evidence, kept after presence was dropped: items whose section
/// header is GONE.
///
/// This is the failure the old presence check was really guarding, and it is
/// strictly narrower -- it fires only when there is something to unname.
fn orphaned_items(text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    for kind in ITEM_KINDS {
        let Some(header) = header_for(kind) else {
            continue;
        };
        if section_at(text, kind).is_some() {
            continue;
        }
        if let Some((line, _)) = declared_at(text, kind).first() {
            out.push(orphaned_item(kind, header).at(*line));
        }
    }
    out
}

fn orphaned_item(kind: char, header: &str) -> Violation {
    Violation::new("V11", format!("`{kind}` items with no `{header}`"))
        .why("a lost header silently unnames every item under it")
        .try_(
            Fix::Mechanical,
            format!("add `{header}` in FORMAT.md order"),
        )
}

fn misordered_section(header: &str) -> Violation {
    Violation::new("V11", format!("`{header}` is out of order"))
        .why("every §S.n address is read against the section order")
        .try_(Fix::Mechanical, "move the section into FORMAT.md order")
}

/// V27: a canonical LETTER carries its canonical MEANING.
///
/// The header must contain the canonical word; qualifiers may follow freely.
/// FORMAT.md's ADDRESSING promises ZERO AMBIGUITY -- `\u{a7}V.2` has to mean
/// the same kind of thing in every repo, or a citation that travels resolves
/// to a different concept. A repo where `\u{a7}V` holds version pinning has
/// broken that for everyone else, silently.
///
/// V11 asks only whether the section is THERE; this asks what it is called.
/// Splitting them is what lets each say something true: before T15 the two
/// were one rule, and a title-cased header was reported as a MISSING section.
///
/// LOW NOISE, measured before it was written: 236 of 261 canonical-letter
/// headers across the fleet already carry the word. The 25 that do not split
/// two ways, and the fix classification is where that split lives -- see
/// `SYNONYMS`. This is why the table has a runner today rather than waiting
/// for `migrate`: it decides what an agent may apply unattended.
#[must_use]
pub fn labels_canonical(text: &str) -> Vec<Violation> {
    text.lines()
        .enumerate()
        .flat_map(|(i, line)| {
            mislabelled(line).map(|v| v.at(i.saturating_add(1)))
        })
        .collect()
}

/// The violation this header carries, if any.
fn mislabelled(line: &str) -> Option<Violation> {
    let (kind, word) = CANONICAL_WORDS
        .into_iter()
        .find(|(kind, _)| is_header_for(line, *kind))?;
    let label = label_of(line, kind).to_lowercase();
    if label.contains(word) {
        return None;
    }
    Some(wrong_label(kind, word, &label))
}

/// Everything after `## \u{a7}X` -- the label, whatever it is.
///
/// Shared with `migrate`, which decides what to do about it (V7).
#[must_use]
pub fn label_of(line: &str, kind: char) -> &str {
    line.trim_end()
        .strip_prefix("## \u{a7}")
        .and_then(|r| r.get(kind.len_utf8()..))
        .unwrap_or("")
        .trim()
}

fn wrong_label(kind: char, word: &str, label: &str) -> Violation {
    let (fix, how) = repair(kind, word, label);
    Violation::new("V27", format!("`\u{a7}{kind}` is not named `{word}`"))
        .why("a citation that travels resolves to a different concept")
        .try_(fix, how)
}

/// What can be done about this label, and whether a tool may do it alone.
///
/// V27's whole judgement lives here rather than in the rule, which is why
/// `SYNONYMS` needs no `migrate` to justify its existence: this decides what
/// an agent may apply unattended, today.
fn repair(kind: char, word: &str, label: &str) -> (Fix, String) {
    if label.is_empty() {
        return (Fix::Mechanical, format!("name it `{word}`"));
    }
    if is_synonym(kind, label) {
        let how =
            format!("say `{word}`, keeping `{label}` as a migration note");
        return (Fix::Mechanical, how);
    }
    let how = format!(
        "`{label}` names a DIFFERENT concept, so the CONTENT must move; \
         no header rewrite can do that"
    );
    (Fix::Judgment, how)
}

fn is_synonym(kind: char, label: &str) -> bool {
    SYNONYMS
        .into_iter()
        .any(|(k, syn)| k == kind && label.contains(syn))
}

/// V12: ids are UNIQUE and never reused; a GAP is fine.
#[must_use]
pub fn ids_unique(text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    for kind in ITEM_KINDS {
        let mut seen: Vec<String> = Vec::new();
        for (line, id) in declared_at(text, kind) {
            let label = id.label();
            if seen.contains(&label) {
                out.push(duplicate_id(&label).at(line));
            }
            seen.push(label);
        }
    }
    out
}

fn duplicate_id(label: &str) -> Violation {
    Violation::new("V12", format!("`{label}` is declared twice"))
        .why("a reused id silently redirects every citation to the old meaning")
        .try_(
            Fix::Judgment,
            "decide which declaration keeps the id, then renumber the OTHER \
             to the next free one; a gap costs nothing, a wrong choice \
             redirects every citation (B2)",
        )
}

/// V13: every citation RESOLVES; a citation is a `V<n>` outside backticks.
#[must_use]
pub fn citations_resolve(text: &str) -> Vec<Violation> {
    let known: Vec<String> =
        declared(text, 'V').iter().map(Id::label).collect();
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    let lines: Vec<&str> = text.lines().collect();
    for (line, cite) in cited_at(text) {
        if !known.contains(&cite) && !seen.contains(&cite) {
            seen.push(cite.clone());
            let crossing = lines
                .get(line.saturating_sub(1))
                .is_some_and(|l| names_a_spec_file(l));
            out.push(dangling(&cite, crossing).at(line));
        }
    }
    out
}

/// Whether this line names ANOTHER spec file, outside backticks.
///
/// The signal that a bare id on it is probably not ours. `\u{a7}F` exists to
/// name neighbouring specs, so this is where the mistake lives -- but the
/// test is the LINE rather than the section, because the same sentence is
/// written in prose elsewhere and is wrong there for the same reason.
///
/// A PATH, not any mention of a `.md` file. `V8: **FORMAT.md ships
/// verbatim.** see V9` names a file and cites nothing across an edge, and
/// it was getting the cross-file repair ranked FIRST, where it is exactly
/// the wrong advice. A neighbour is reached by a path, so the token has to
/// carry a separator.
fn names_a_spec_file(line: &str) -> bool {
    outside_backticks(line)
        .split_whitespace()
        .any(|token| token.contains('/') && token.contains(".md"))
}

/// V13's report, with a THIRD direction when the line names another spec.
///
/// B26: `\u{a7}F` made cross-file references an ordinary thing to write, and
/// on `- down: worker/SPEC.md V2` this rule said `V2` is cited but never
/// declared -- true -- and then offered two fixes that are both WRONG here.
/// The rule was right about WHETHER and useless about WHICH, which is B8 and
/// B9's shape a third time, arriving in the one section this branch added
/// for naming other files.
///
/// The remedy is B3's, already the format's: backticks are verbatim, so a
/// qualified id written in them is a literal rather than a citation, exactly
/// as V19 requires for an id that crosses a namespace. Ranked FIRST when the
/// line names a file, because there it is the likeliest fix -- and offered
/// as JUDGEMENT, since only a reader knows whether `V2` is a rule elsewhere
/// or one missing here.
fn dangling(cite: &str, crossing: bool) -> Violation {
    let v = Violation::new(
        "V13",
        format!("`{cite}` is cited but never declared"),
    )
    .why("a dangling reference reads as authoritative, so nobody follows it");
    let v = if crossing {
        v.try_(Fix::Judgment, cross_file_repair(cite))
    } else {
        v
    };
    v.try_(Fix::Judgment, "point it at the rule that was meant")
        .try_(
            Fix::Judgment,
            format!("declare {cite}, if the rule is real but missing"),
        )
}

fn cross_file_repair(cite: &str) -> String {
    format!(
        "this line names another spec: if `{cite}` is ITS rule, write the \
         reference in backticks -- a bare id is read against THIS file (V19)"
    )
}

/// V42: a row's LITERAL pipe is escaped, so the row splits into its four
/// fields and no others.
///
/// `id|status|text|cites` is positional, so an unescaped `|` in the TEXT
/// does not break the row -- it silently moves the boundary. The last field
/// stops being citations and becomes a fragment of prose, and every reader
/// downstream believes it: `mth tasks --format json` is the contract a
/// consumer parses (M11), and it was emitting a sentence where a caller
/// expected `["V6","V10","V18"]`.
///
/// FORMAT.md's cell rule is `literal | -> escape as \|`, unconditional --
/// "Backticks OK" permits backticks in a cell, it does not exempt them. So
/// a row showing a pipe-table example inside backticks must escape it, and
/// ours did not.
///
/// B29, and B15's shape a second time: a check that passes on a file it
/// could not parse. `check` was green on all eight of our own malformed
/// rows, because every other rule reads the text rather than the fields.
///
/// MEASURED BY THE SWEEP, not by hand: 32 rows in 10 of 256 distinct fleet
/// specs -- 1.4% -- and it turns SEVEN clean specs red, 114 to 107. That
/// second number is the one a consumer pays, and it is stated beside the row
/// count that flatters it (§G).
///
/// The first figure written here read "40 of 2,289 rows over 259 specs". It
/// was hand-counted with a throwaway script, and it counted our own eight
/// malformed rows before they were repaired -- so it described a corpus that
/// no longer exists. `SPEC.md` retracted it and this copy did not, which is
/// the whole reason a number belongs in one place.
#[must_use]
pub fn rows_escape_pipes(text: &str) -> Vec<Violation> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let id = at_line_start(line)?;
            ROW_KINDS.contains(&id.kind).then_some(())?;
            // PIPE DIALECT ONLY. The id grammar also admits `T1: text`, and
            // a colon declaration is not a four-field row -- it has no
            // fields at all. Judging one counts zero pipes and reports a
            // row that was never written as a row: 76 findings across 5
            // fleet specs, on a dialect the format allows.
            (id.terminator == '|').then_some(())?;
            let found = unescaped_pipes(line);
            (found != 3).then(|| {
                split_wrongly(&id.label(), found).at(i.saturating_add(1))
            })
        })
        .collect()
}

/// How many `|` in this line are STRUCTURAL -- that is, not escaped.
///
/// Asked of the SPLITTER rather than counted again here. This carried its own
/// escape toggle, a second reading of one rule (V7) inside the crate that
/// exists to end exactly that; it happened to agree with `cells`, which is
/// the reason nothing caught it and not a reason to keep it. A field count
/// is one more than the boundaries between the fields.
fn unescaped_pipes(line: &str) -> usize {
    cells(line).len().saturating_sub(1)
}

/// BOTH directions, because the rule is "four fields", not "not too many".
///
/// Too FEW was unreported until a review asked: the message already said
/// "not 4" and the rule already said "its 4 fields & no others", while the
/// guard only counted upward. A row missing its last field emits an empty
/// `cites`, which reads exactly like a row that cites nothing -- the same
/// indistinguishability B29 was filed for, from the other side.
///
/// MEASURED before widening: 0 of 2,294 fleet rows carry too few, so this
/// half costs nobody anything today. It is here for the row written
/// tomorrow.
fn split_wrongly(label: &str, found: usize) -> Violation {
    let fields = found.saturating_add(1);
    let v = Violation::new(
        "V42",
        format!("`{label}` splits into {fields} fields, not 4"),
    );
    if found < 3 {
        return too_few(v);
    }
    v.why("the last field stops being citations and becomes prose")
        .try_(
            Fix::Mechanical,
            "escape the literal pipes as `\\|` -- the first two and the last \
             are the row's own",
        )
        .try_(Fix::Judgment, "or reword the text so it carries no pipe")
}

/// A row that STOPS EARLY: the missing field reads as an empty one.
fn too_few(v: Violation) -> Violation {
    v.why("a missing field reads exactly like an empty one")
        .try_(Fix::Judgment, "add the missing field; `-` if it is empty")
}

/// V41: a SUPERSESSION marker points at LIVE law.
///
/// `V3: **an old rule.** [superseded by V9]` retires V3 without deleting it,
/// because deleting would free the id (V12) and strand every citation (V13).
/// The mark is what lets a reader -- and `derive` -- tell a RETIRED rule
/// from one nobody has cited yet.
///
/// Three ways it can lie, none of which V13 can see, because a citation that
/// RESOLVES is all V13 asks:
///
/// * SELF -- `V3 [superseded by V3]` retires nothing and reads as retired.
/// * DEAD WINNER -- the rule named is itself superseded, so a reader
///   following the pointer arrives at law that is also not in force. The
///   chain has a live end; the mark must name it.
/// * DANGLING -- caught by V13 already, and deliberately not restated here
///   (V7): two rules reporting one defect send two people to one line.
#[must_use]
pub fn supersessions_resolve(text: &str) -> Vec<Violation> {
    let retired = retired(text);
    let mut out = Vec::new();
    for (line, id, winners) in supersessions(text) {
        for winner in winners {
            out.extend(bad_winner(&id, &winner, &retired).map(|v| v.at(line)));
        }
    }
    out
}

/// The violation this winner carries, if any.
fn bad_winner(id: &str, winner: &str, retired: &[String]) -> Option<Violation> {
    if winner == id {
        return Some(supersedes_itself(id));
    }
    retired
        .contains(&winner.to_owned())
        .then(|| winner_is_retired(id, winner))
}

/// Every `V` id whose own line carries the marker -- the RETIRED set.
///
/// Shared with `derive`, which asks the same question for a different reason:
/// a retired rule is not an orphan (V7 -- one reading of the mark, not two).
#[must_use]
pub fn retired(text: &str) -> Vec<String> {
    supersessions(text)
        .into_iter()
        .map(|(_, id, _)| id)
        .collect()
}

/// One retirement: where it is written, which rule it retires, and which
/// rules it names as the replacement.
type Retirement = (usize, String, Vec<String>);

/// Each marked line as `(line, the id it declares, the ids it names)`.
///
/// Only a `\u{a7}V` DECLARATION can be superseded: the mark says a rule is no
/// longer in force, and a task or a bug row is not a rule. A marker anywhere
/// else is ordinary prose this says nothing about.
fn supersessions(text: &str) -> Vec<Retirement> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let id = at_line_start(line).filter(|id| id.kind == 'V')?;
            let winners = marked(line);
            (!winners.is_empty())
                .then(|| (i.saturating_add(1), id.label(), winners))
        })
        .collect()
}

/// The ids inside this line's `[superseded by ...]`, if it carries one.
///
/// MANY winners, not one: measured in the fleet as `superseded by V17, V18`
/// -- one rule replaced by a pair is the ordinary case when a rule is split,
/// and a grammar that allowed only one would send that author back to prose.
///
/// Read OUTSIDE backticks, reusing V13's own boundary (V7): a marker shown
/// as an example in `code` is a literal, exactly as a citation there is.
/// The OPENING BRACKET is part of the marker, not decoration around it.
///
/// B30: this matched the words alone, so `V6 was superseded by V7 [B14].`
/// -- ordinary prose with a bracket anywhere after it -- parsed as a mark
/// naming `V7`, and V41 reported V7 as superseded by itself. The rule
/// written to stay quiet on the prose the fleet already writes was loud on
/// exactly that, which is the failure its own companion test claims to
/// prevent: that test used prose with NO `]`, the one prose shape that
/// happened to be safe.
fn marked(line: &str) -> Vec<String> {
    let bare = outside_backticks(line);
    let opening = format!("[{SUPERSEDED_BY}");
    let Some((_, rest)) = bare.split_once(opening.as_str()) else {
        return Vec::new();
    };
    let Some((inside, _)) = rest.split_once(']') else {
        return Vec::new();
    };
    inside
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| is_invariant_ref(t))
        .map(str::to_owned)
        .collect()
}

fn supersedes_itself(id: &str) -> Violation {
    Violation::new("V41", format!("`{id}` is superseded by itself"))
        .why("a rule that retires itself leaves no law in force")
        .try_(
            Fix::Judgment,
            "name the rule that REPLACED it, or drop the mark",
        )
}

fn winner_is_retired(id: &str, winner: &str) -> Violation {
    Violation::new("V41", format!("`{id}` points at `{winner}`, retired too"))
        .why("a reader following the mark arrives at law that is also dead")
        .try_(
            Fix::Mechanical,
            format!("retag `{id}` to whatever superseded `{winner}`"),
        )
}

/// V14: rows appear in SORTED id order, and a suffixed id RIDES its base.
#[must_use]
pub fn rows_sorted(text: &str) -> Vec<Violation> {
    let mut out = Vec::new();
    // ROW kinds -- the pipe-table sections. `V` is excluded because an
    // invariant is a `V1:` statement, not a row, and `R` joined them with
    // 4.1.0 for the same reason `T` and `B` are here: `R1|topic|finding|src`.
    for kind in ROW_KINDS {
        let rows = declared_at(text, kind);
        for pair in rows.windows(2) {
            match (pair.first(), pair.get(1)) {
                (Some(a), Some(b)) if b.1.sort_key() < a.1.sort_key() => {
                    out.push(unsorted(&b.1.label(), &a.1.label()).at(b.0));
                }
                _ => continue,
            }
        }
    }
    out
}

fn unsorted(row: &str, after: &str) -> Violation {
    Violation::new("V14", format!("`{row}` sorts before `{after}` above it"))
        .why("an out-of-order block renders identically to a sorted one")
        .try_(
            Fix::Mechanical,
            "move the row into id order; a suffixed id rides its base",
        )
}

/// V15: every task belongs to EXACTLY ONE milestone -- WHERE MILESTONES EXIST.
///
/// Milestones are an EXTENSION, not part of the format: canonical `\u{a7}T`
/// has none, and 48 of 51 fleet specs carry no `| M<n> |` row at all. Against
/// those this fired `in no milestone` on EVERY task -- one violation per row,
/// for declining to use an optional feature. A check that loud is one nobody
/// reads, which is how the real violation hides.
///
/// So the rule is scoped to specs that OPTED IN. A single milestone row is
/// the opt-in signal, and it is the right one: adding the first milestone is
/// the moment "every task belongs to one" starts being a claim the author is
/// making. Below that line the rule has nothing to say, and says it.
#[must_use]
pub fn tasks_in_one_milestone(text: &str) -> Vec<Violation> {
    if !uses_milestones(text) {
        return Vec::new();
    }
    let claimed = claims(text);
    let mut out = duplicate_claims(&claimed);
    for (line, id) in declared_at(text, 'T') {
        if id.suffix.is_empty() && !claimed.contains(&id.num) {
            out.push(unclaimed(&id.label()).at(line));
        }
    }
    out.extend(claims_without_rows(text, &claimed));
    out
}

fn unclaimed(label: &str) -> Violation {
    Violation::new("V15", format!("{label} is in no milestone"))
        .why("88 tasks once belonged to none while two gates stayed green")
        .try_(
            Fix::Judgment,
            "add it to the RIGHT milestone's tasks cell; ranges expand",
        )
        .try_(Fix::Judgment, "delete the row, if the work is not real")
}

fn duplicate_claims(claimed: &[u32]) -> Vec<Violation> {
    let mut seen: Vec<u32> = Vec::new();
    let mut out = Vec::new();
    for n in claimed {
        if seen.contains(n) {
            out.push(claimed_twice(*n));
        }
        seen.push(*n);
    }
    out
}

fn claimed_twice(n: u32) -> Violation {
    Violation::new("V15", format!("T{n} is claimed by two milestones"))
        .why("EXACTLY one, or the rule is satisfied by claiming everything")
        .try_(
            Fix::Judgment,
            "decide which milestone owns it, then remove the other claims",
        )
}

/// The other direction: a milestone naming a task that has no row. Without it
/// the rule passes by claiming everything, including work that does not exist.
fn claims_without_rows(text: &str, claimed: &[u32]) -> Vec<Violation> {
    let rows: Vec<u32> = declared(text, 'T').iter().map(|i| i.num).collect();
    claimed
        .iter()
        .filter(|n| !rows.contains(n))
        .map(|n| phantom_claim(*n))
        .collect()
}

fn phantom_claim(n: u32) -> Violation {
    Violation::new("V15", format!("a milestone claims T{n}, which has no row"))
        .why("a renamed or deleted task leaves the claim behind")
        .try_(Fix::Mechanical, "drop the claim")
        .try_(
            Fix::Judgment,
            format!("add the T{n} row, if the work is real"),
        )
}

/// The three statuses FORMAT.md allows, and nothing else.
pub const STATUSES: [&str; 3] = [".", "~", "x"];

/// V25: a task STATUS is `.` todo, `~` wip or `x` done.
///
/// Read from the SECOND pipe field of a `T<n>|` row. A status outside the
/// set renders as ordinary text, so a table full of them looks fine while
/// every runner that reads the column disagrees about the state of the work.
#[must_use]
pub fn statuses_valid(text: &str) -> Vec<Violation> {
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let id = at_line_start(line).filter(|id| id.kind == 'T')?;
            let status = crate::id::cells(line).get(1).copied()?;
            (!STATUSES.contains(&status)).then(|| {
                bad_status(&id.label(), status).at(i.saturating_add(1))
            })
        })
        .collect()
}

fn bad_status(label: &str, status: &str) -> Violation {
    Violation::new("V25", format!("{label} has status `{status}`"))
        .why("a status outside . ~ x renders fine and every runner reads it differently")
        .try_(
            Fix::Judgment,
            "set it to whichever of `.` todo, `~` wip or `x` done the work \
             actually is",
        )
}

/// Whether this spec uses milestones at all -- V15's opt-in signal.
///
/// A row with an EMPTY tasks cell still counts: the author declared a
/// milestone, so "which tasks are in it" is a fair question.
#[must_use]
pub fn uses_milestones(text: &str) -> bool {
    text.lines().any(|l| l.starts_with("| M"))
}

/// Every task number claimed by a `| M<n> |` row's THIRD field -- the same
/// cell the original checker reads, so the two cannot disagree about where to
/// look.
#[must_use]
pub fn claims(text: &str) -> Vec<u32> {
    text.lines()
        .filter(|l| l.starts_with("| M"))
        .flat_map(|l| {
            let fields = crate::id::cells(l);
            expand_cell(fields.get(3).copied().unwrap_or(""))
        })
        .collect()
}

/// `T1-T4, T12` -> the task numbers it names.
///
/// Ranges are the format's own affordance and the reason the mapping is cheap
/// to maintain: not knowing about them is why the column once got judged a
/// burden and deleted.
#[must_use]
pub fn expand_cell(cell: &str) -> Vec<u32> {
    let mut out = Vec::new();
    for token in cell.split(',') {
        match token.trim().split_once('-') {
            Some((a, b)) => match (task_num(a), task_num(b)) {
                (Some(lo), Some(hi)) => out.extend(lo..=hi),
                _ => continue,
            },
            None => out.extend(task_num(token)),
        }
    }
    out
}

fn task_num(s: &str) -> Option<u32> {
    s.trim().strip_prefix('T')?.parse().ok()
}

/// One named record: the id that owns it, and a marker substring of it.
pub type Record = (String, String);

/// V16: considered-and-CLOSED records SURVIVE -- rejected, or DEFERRED
/// under a stated trigger.
///
/// An option recorded with its rejection & the trigger that would reopen it
/// is what makes a decision AUDITABLE rather than merely obeyable, and
/// compaction must never trade one away for bytes.
///
/// `expected` is the CALLER's, and that is the whole design: survival is a
/// claim about edits over time, so no single text can say what used to be in
/// it. The checker owns the rule; the repo being checked owns the list.
///
/// Checked by NAMED records rather than a word count, because an arbitrary
/// threshold on how often a word appears either fires on nothing or fires on
/// prose edits that changed no decision.
#[must_use]
pub fn records_survive(text: &str, expected: &[Record]) -> Vec<Violation> {
    expected
        .iter()
        .filter(|(id, marker)| !body(text, id).contains(marker.as_str()))
        .map(|(id, marker)| lost_record(id, marker, line_of(text, id)))
        .collect()
}

/// V16 offers NO mechanical direction, and that is the point rather than an
/// omission. Restoring a deleted record needs the record; deciding the option
/// no longer needs one changes intent. Both are judgement, so an agent that
/// hits this must stop, which is exactly what compaction trading a record
/// away for bytes should feel like.
fn lost_record(id: &str, marker: &str, line: usize) -> Violation {
    Violation::new("V16", format!("{id} lost its `{marker}` record"))
        .why("a closed option without its record is obeyable but not auditable")
        .try_(
            Fix::Judgment,
            "restore the record, or record why the option no longer needs one",
        )
        .at(line)
}

/// The 1-based line a declaration sits on, or 0 if it is gone entirely.
fn line_of(text: &str, id: &str) -> usize {
    text.lines()
        .enumerate()
        .find(|(_, l)| at_line_start(l).is_some_and(|i| i.label() == id))
        .map_or(0, |(i, _)| i.saturating_add(1))
}

/// The body of one declaration, up to the next one.
///
/// Scoped rather than searching the whole file: a marker that moved to some
/// other rule would otherwise still count as present, and "the record is
/// somewhere" is not the claim V16 makes.
#[must_use]
pub fn body(text: &str, id: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for line in text.lines() {
        match at_line_start(line) {
            Some(found) if found.label() == id => inside = true,
            Some(_) => inside = false,
            None => {}
        }
        if inside {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Parse a `.spec-records` baseline: `<id>` then whitespace then the marker,
/// which runs to end of line so it may contain spaces. `#` comments and blank
/// lines are skipped.
#[must_use]
pub fn parse_records(text: &str) -> Vec<Record> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once(char::is_whitespace))
        .map(|(id, marker)| (id.to_owned(), marker.trim().to_owned()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal spec with every section, in order, that all five rules
    /// accept. The companion to every planted violation below: a guard that
    /// rejected everything would pass an all-planted suite (V18).
    const REAL: [&str; 27] = [
        "# spec",
        "",
        "## \u{a7}G GOAL",
        "one line.",
        "",
        "## \u{a7}C CONSTRAINTS",
        "- a bullet",
        "",
        "## \u{a7}I INTERFACES",
        "- `cmd foo` -- does a thing",
        "",
        "## \u{a7}V INVARIANTS",
        "V1: **a rule.** cited by T1.",
        "V3: **a gap above is fine.** V1 is cited here too.",
        "",
        "## \u{a7}T TASKS",
        "| id | scope | tasks | done-when |",
        "|----|-------|-------|-----------|",
        "| M1 | first | T1-T2, T4 | done |",
        "T1|x|a task|V1",
        "T2|.|another|V3",
        "T2a|.|rides its base|V1",
        "T4|.|after a gap|V1",
        "",
        "## \u{a7}B BUGS",
        "id|date|cause|fix",
        "B1|2026-08-01|a cause|a fix",
    ];

    fn real() -> String {
        REAL.join("\n")
    }

    /// Swap two whole lines. A `replace` pair cannot do this: the second call
    /// finds the header the first one just wrote and undoes it -- which is
    /// exactly how the first version of this test passed while planting
    /// nothing at all.
    fn swap_lines(text: &str, a: &str, b: &str) -> String {
        text.lines()
            .map(|l| match l {
                _ if l == a => b,
                _ if l == b => a,
                _ => l,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn all(text: &str) -> Vec<Violation> {
        let mut out = sections_ordered(text);
        out.extend(ids_unique(text));
        out.extend(citations_resolve(text));
        out.extend(rows_sorted(text));
        out.extend(tasks_in_one_milestone(text));
        out
    }

    /// The companion for all five rules at once: every real shape passes.
    #[test]
    fn a_well_formed_spec_has_no_violations() {
        assert_eq!(all(&real()), Vec::<Violation>::new());
    }

    fn v11_says(text: &str, want: &str) {
        let got = sections_ordered(text);
        assert!(
            got.iter().any(|v| v.msg.contains(want)),
            "want {want}: {got:?}"
        );
    }

    /// A header lost to an edit. Everything under it is still there and still
    /// reads fine, which is the problem: it is no longer in any section.
    ///
    /// This is what survived the calibration. The rule no longer asks whether
    /// the header is PRESENT -- it asks whether anything was left stranded by
    /// its absence, which is the same defect and nothing more.
    #[test]
    fn v11_rejects_items_whose_header_is_gone() {
        v11_says(&real().replace("## \u{a7}B BUGS", "## BUGS"), "no `##");
    }

    /// The companion, and the whole point of T15: a spec that simply HAS no
    /// bugs yet needs no §B, and 4.1.0 says so. The old rule failed 49 of 51
    /// fleet specs largely on this.
    #[test]
    fn v11_accepts_a_section_that_is_absent_with_nothing_under_it() {
        let text = real();
        let cut = text.split("## \u{a7}B BUGS").next().unwrap_or_default();
        assert_eq!(sections_ordered(cut), Vec::<Violation>::new());
    }

    /// V11 is LABEL-AGNOSTIC: the section exists, whatever it is called.
    /// Only 16 of 51 fleet specs spell these headers exactly as `SECTIONS`
    /// does, and calling the other 35 "missing" was a true rule delivering a
    /// false message. Whether the label carries the canonical word is T17's.
    #[test]
    fn v11_finds_a_section_under_any_label() {
        for label in ["", " Invariants", " \u{2014} Invariants", " INVARIANTS"]
        {
            let text = real().replace(
                "## \u{a7}V INVARIANTS",
                &format!("## \u{a7}V{label}"),
            );
            assert_eq!(
                sections_ordered(&text),
                Vec::<Violation>::new(),
                "label {label:?}"
            );
        }
    }

    /// ...but a HEADING that merely starts with a section letter is not that
    /// section. `## §T55-PLAN` is a real fleet heading, and reading it as §T
    /// would silently satisfy the rule with a document that has no §T at all.
    #[test]
    fn a_heading_that_runs_into_the_letter_is_not_a_section() {
        let text = real().replace("## \u{a7}T TASKS", "## \u{a7}T55-PLAN");
        v11_says(&text, "no `##");
    }

    /// V27, planted with the case that motivated it: six fleet repos use the
    /// canonical letters for other concepts. `§V VERSIONING` holds version
    /// pinning, so `§V.2` means something different there than everywhere
    /// else -- and FORMAT.md's ADDRESSING promises zero ambiguity.
    ///
    /// JUDGMENT, not mechanical: the content has to move to another section,
    /// and no header rewrite can do that. An agent applying this unattended
    /// would file version pinning under INVARIANTS and call it canonical.
    #[test]
    fn v27_rejects_a_letter_used_for_another_concept() {
        for (canonical, wrong) in [
            ("## \u{a7}V INVARIANTS", "## \u{a7}V VERSIONING"),
            ("## \u{a7}T TASKS", "## \u{a7}T TESTING"),
            ("## \u{a7}B BUGS", "## \u{a7}B BUILD"),
        ] {
            let got = labels_canonical(&real().replace(canonical, wrong));
            assert_eq!(got.len(), 1, "{wrong}: {got:?}");
            assert!(
                got.first().is_some_and(|v| !v.is_mechanical()),
                "{wrong} must be judgement -- the content moves: {got:?}"
            );
        }
    }

    /// A SYNONYM is still a violation, but a repairable one: the section
    /// holds what the letter promises, so writing the canonical word and
    /// keeping the old label as a note is lossless and computable.
    #[test]
    fn v27_calls_a_listed_synonym_mechanical() {
        let text =
            real().replace("## \u{a7}I INTERFACES", "## \u{a7}I SURFACES");
        let got = labels_canonical(&text);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got.first().is_some_and(Violation::is_mechanical), "{got:?}");
    }

    /// A BARE header names nothing at all, so appending the canonical word is
    /// the single correct edit -- mechanical for the same reason.
    #[test]
    fn v27_calls_a_bare_header_mechanical() {
        let text = real().replace("## \u{a7}V INVARIANTS", "## \u{a7}V");
        let got = labels_canonical(&text);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(got.first().is_some_and(Violation::is_mechanical), "{got:?}");
    }

    /// The companion, and the reason this rule is worth having: 236 of 261
    /// fleet headers ALREADY carry the word. A qualifier, a dash, a case
    /// change and a parenthetical all pass, so the rule cannot satisfy itself
    /// by rejecting every header that is not byte-identical to SECTIONS.
    #[test]
    fn v27_accepts_every_header_that_names_the_concept() {
        for label in [
            "## \u{a7}B BUGS",
            "## \u{a7}B Bugs",
            "## \u{a7}B \u{2014} Bugs / Known Issues",
            "## \u{a7}B bug log",
        ] {
            let text = real().replace("## \u{a7}B BUGS", label);
            assert_eq!(
                labels_canonical(&text),
                Vec::<Violation>::new(),
                "{label}"
            );
        }
    }

    /// An UNKNOWN letter has no canonical word, so it is not this rule's
    /// business -- same tolerance V11 grants it.
    #[test]
    fn v27_ignores_an_extension_section() {
        let text = format!("{}\n## \u{a7}D DECISIONS\nprose.\n", real());
        assert_eq!(labels_canonical(&text), Vec::<Violation>::new());
    }

    /// One misplaced section is ONE violation, and it names the section that
    /// actually moved.
    ///
    /// A fleet spec carries §R after §B. Walking in canonical order and
    /// comparing positions reported that as THREE violations -- against §V,
    /// §T and §B, none of which had moved -- because one late section makes
    /// every section before it look early. `migrate` reads this to decide
    /// what to move, so blaming the wrong three would move the wrong three.
    #[test]
    fn a_late_section_is_blamed_alone() {
        let text = real().replace(
            "## \u{a7}I INTERFACES",
            "## \u{a7}I INTERFACES\nplaceholder\n",
        );
        let moved = format!("{text}\n## \u{a7}R RESEARCH\nR1|a|b|c\n");
        let got = sections_ordered(&moved);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            got.first().is_some_and(|v| v.msg.contains("\u{a7}R")),
            "must name the section that moved: {got:?}"
        );
    }

    /// 4.1.0's `§R`, accepted: a research row is an ordinary pipe row, so
    /// nothing here needed a rule of its own.
    #[test]
    fn v11_accepts_a_research_section() {
        let with_r = real().replace(
            "## \u{a7}V INVARIANTS",
            "## \u{a7}R RESEARCH\nid|topic|finding|src\nR1|jwt|`jose` wins|url\n\n## \u{a7}V INVARIANTS",
        );
        assert_eq!(sections_ordered(&with_r), Vec::<Violation>::new());
    }

    /// ...and an R row with no §R above it is stranded, exactly as a T or B
    /// row would be. This is the half that proves acceptance is not blindness.
    #[test]
    fn v11_rejects_a_research_row_with_no_section() {
        let orphan = real().replace(
            "## \u{a7}V INVARIANTS",
            "R1|jwt|`jose` wins|url\n\n## \u{a7}V INVARIANTS",
        );
        v11_says(&orphan, "no `##");
    }

    /// §R is OPTIONAL -- present only if `/research` ran -- so its absence
    /// must stay silent. `real()` has none, and this pins that.
    #[test]
    fn a_spec_with_no_research_section_still_passes() {
        assert!(!real().contains("\u{a7}R"), "the fixture must have no §R");
        assert_eq!(sections_ordered(&real()), Vec::<Violation>::new());
    }

    /// V12 and V14 cover §R for free, which was the whole argument for
    /// teaching the grammar one letter instead of writing new rules.
    #[test]
    fn v12_and_v14_reach_research_rows_for_free() {
        let dup = "## \u{a7}R RESEARCH\nR1|a|b|c\nR1|d|e|f\n";
        assert!(ids_unique(dup).iter().any(|v| v.rule == "V12"), "V12");
        let unsorted = "## \u{a7}R RESEARCH\nR2|a|b|c\nR1|d|e|f\n";
        assert!(rows_sorted(unsorted).iter().any(|v| v.rule == "V14"), "V14");
    }

    /// Extension sections are real and in fleet use -- §D, §E, §O, §P, §X.
    /// Only the KNOWN letters are ordered against each other; an unknown one
    /// between them is not the checker's business.
    #[test]
    fn v11_tolerates_an_unknown_section_letter() {
        let with_ext = real().replace(
            "## \u{a7}T TASKS",
            "## \u{a7}D DECISIONS\n\nsome prose.\n\n## \u{a7}T TASKS",
        );
        assert_eq!(sections_ordered(&with_ext), Vec::<Violation>::new());
    }

    /// V39, the shape it is FOR: a spec that spans a directory tree declares
    /// its edges in `§F` and its derived navigation in `§N`, and both sit
    /// between `§G` and `§C` -- the edges are structure, so a reader meets
    /// them before the constraints written in their terms.
    fn federating(text: &str) -> String {
        text.replace(
            "## \u{a7}C CONSTRAINTS",
            "## \u{a7}F FEDERATION\n- `../SPEC.md` is the parent\n\n\
             ## \u{a7}N NAV\n- up: `../SPEC.md`\n\n## \u{a7}C CONSTRAINTS",
        )
    }

    #[test]
    fn v39_accepts_the_federating_pair_in_rank() {
        let text = federating(&real());
        assert_eq!(sections_ordered(&text), Vec::<Violation>::new());
        assert_eq!(labels_canonical(&text), Vec::<Violation>::new());
    }

    /// PLANTED (V18): rank is the half a constant change can get wrong
    /// silently, so each new letter is placed after `§C` and must be blamed
    /// BY NAME -- the `§R`-after-`§B` defect one letter further out.
    #[test]
    fn v39_rejects_a_federating_section_out_of_rank() {
        for (header, body) in [
            ("## \u{a7}F FEDERATION", "- `../SPEC.md` is the parent"),
            ("## \u{a7}N NAV", "- up: `../SPEC.md`"),
        ] {
            let late = real().replace(
                "## \u{a7}I INTERFACES",
                &format!("{header}\n{body}\n\n## \u{a7}I INTERFACES"),
            );
            let got = sections_ordered(&late);
            assert_eq!(got.len(), 1, "{header}: {got:?}");
            assert!(
                got.first().is_some_and(|v| v.msg.contains(header)),
                "{header}: {got:?}"
            );
        }
    }

    /// A minimal spec carrying one federating header, in rank.
    fn one_header(label: &str) -> String {
        format!(
            "## \u{a7}G GOAL\none line.\n\n{label}\n- an edge\n\n\
             ## \u{a7}C CONSTRAINTS\n- a bullet\n"
        )
    }

    /// V27 matches a STEM, so every spelling a real header wears passes.
    /// `Navigation` is why the word is `nav` rather than `navigation`.
    #[test]
    fn v39_accepts_every_spelling_of_the_new_headers() {
        for label in [
            "## \u{a7}F FEDERATION",
            "## \u{a7}F Federation",
            "## \u{a7}F \u{2014} Federation",
            "## \u{a7}N NAV",
            "## \u{a7}N Nav",
            "## \u{a7}N \u{2014} Navigation",
        ] {
            let text = one_header(label);
            let got = [labels_canonical(&text), sections_ordered(&text)];
            assert_eq!(got.concat(), Vec::<Violation>::new(), "{label}");
        }
    }

    /// The ambiguity V39 exists to end: the letter spent on another concept.
    /// It was LEGAL the day before -- V11 passes an unknown letter untouched
    /// and V27 had no word to hold it to -- which is exactly why claiming the
    /// letter after a second reader picked `§F FIXTURES` would have cost a
    /// collision no header rewrite repairs.
    #[test]
    fn v39_rejects_a_new_letter_used_for_another_concept() {
        for wrong in ["## \u{a7}F FIXTURES", "## \u{a7}N NOTES"] {
            let text = format!("{}\n{wrong}\n- prose.\n", real());
            let got = labels_canonical(&text);
            assert_eq!(got.len(), 1, "{wrong}: {got:?}");
            assert!(
                got.first().is_some_and(|v| !v.is_mechanical()),
                "{wrong} must be judgement -- the content moves: {got:?}"
            );
        }
    }

    /// THE COMPANION, and the one that matters most here: a spec carrying
    /// NEITHER section is untouched. Absence is legal (V11), and this rule's
    /// failure mode is LOUDNESS on every repo that declined an optional
    /// feature -- V15's own history, fired on all 48 specs with no milestone
    /// row.
    #[test]
    fn a_spec_carrying_neither_new_section_is_untouched() {
        let text = real();
        assert!(!text.contains("\u{a7}F"), "the fixture must carry no §F");
        assert!(!text.contains("\u{a7}N"), "the fixture must carry no §N");
        assert_eq!(all(&text), Vec::<Violation>::new());
        assert_eq!(labels_canonical(&text), Vec::<Violation>::new());
    }

    /// A spec whose V3 is retired by V1, in the canonical mark.
    fn retiring(tag: &str) -> String {
        real().replace("V3: **a gap above is fine.**", &format!("V3: {tag}"))
    }

    /// V41's companion FIRST: a well-formed mark is SILENT, and a spec that
    /// carries none is untouched. This rule's failure mode is noise on a
    /// legal file, and the fleet writes retirement in prose today -- zero
    /// specs use the bracket form, so a rule that fired on the prose would
    /// be loud on every repo that never opted in.
    #[test]
    fn v41_is_silent_on_a_well_formed_mark_and_on_none() {
        let live = retiring("**retired.** V1 replaced it [superseded by V1]");
        assert_eq!(supersessions_resolve(&live), Vec::<Violation>::new());
        assert_eq!(supersessions_resolve(&real()), Vec::<Violation>::new());
        let prose = retiring("**retired.** superseded by V1, in prose");
        assert_eq!(supersessions_resolve(&prose), Vec::<Violation>::new());
    }

    /// B30, and the shape the test above MISSED: prose that mentions the
    /// words AND carries a bracket later on.
    ///
    /// `V6 was superseded by V7 [B14].` parsed as a mark naming V7, so V41
    /// reported V7 as superseded by ITSELF. The companion above used prose
    /// with no `]` -- the one prose shape that was already safe -- so it
    /// asserted the property while testing the case that could not fail.
    /// The bracket is part of the marker, not punctuation near it.
    #[test]
    fn v41_ignores_the_words_without_an_opening_bracket() {
        for prose in [
            "**a rule.** V1 was superseded by V3 [B14].",
            "**a rule.** see the note on superseded by V1 (below] here",
        ] {
            let text = retiring(prose);
            assert_eq!(
                supersessions_resolve(&text),
                Vec::<Violation>::new(),
                "{prose}"
            );
        }
    }

    /// PLANTED: a rule that retires ITSELF. V13 cannot see this -- the
    /// citation resolves perfectly well -- which is the whole reason this
    /// rule is not folded into it.
    #[test]
    fn v41_rejects_a_rule_that_supersedes_itself() {
        let text = retiring("**retired by nothing.** [superseded by V3]");
        let got = supersessions_resolve(&text);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            got.first().is_some_and(|v| v.msg.contains("itself")),
            "{got:?}"
        );
    }

    /// PLANTED: the mark points at a rule that is ITSELF retired, so a
    /// reader following it lands on law that is also dead. Mechanical to
    /// repair -- retag to whatever superseded the winner -- and mechanical
    /// is what the fix says, because nothing here needs a judgement.
    #[test]
    fn v41_rejects_a_mark_that_points_at_dead_law() {
        let text = real()
            .replace(
                "V1: **a rule.** cited by T1.",
                "V1: **also retired.** [superseded by V3]",
            )
            .replace(
                "V3: **a gap above is fine.** V1 is cited here too.",
                "V3: **retired.** [superseded by V1]",
            );
        let got = supersessions_resolve(&text);
        assert!(got.iter().any(|v| v.msg.contains("retired too")), "{got:?}");
        assert!(got.iter().all(Violation::is_mechanical), "{got:?}");
    }

    /// MANY winners, because a rule that is SPLIT is replaced by several --
    /// measured in the fleet as `superseded by V17, V18`. Each is checked,
    /// so a good winner beside a bad one does not launder it.
    #[test]
    fn v41_reads_every_winner_a_mark_names() {
        let text = retiring("**split.** [superseded by V1, V3]");
        let got = supersessions_resolve(&text);
        assert_eq!(got.len(), 1, "the self-reference alone: {got:?}");
        assert!(
            got.first().is_some_and(|v| v.msg.contains("itself")),
            "{got:?}"
        );
    }

    /// A marker inside BACKTICKS is a literal, reusing V13's own boundary
    /// (V7) -- this document shows the mark as an example, and an example is
    /// not a mark.
    #[test]
    fn v41_ignores_a_mark_shown_as_an_example() {
        let text = retiring("**a rule about marks.** `[superseded by V3]`");
        assert_eq!(supersessions_resolve(&text), Vec::<Violation>::new());
    }

    /// The mark is read on `\u{a7}V` DECLARATIONS only. A task row saying a
    /// task was superseded is ordinary prose, and the fleet writes exactly
    /// that today -- `T22|x|dead: cycle guard superseded by ...`.
    #[test]
    fn v41_says_nothing_about_a_task_row() {
        let text = real()
            .replace("T2|.|another|V3", "T2|.|dead [superseded by T2]|V3");
        assert_eq!(supersessions_resolve(&text), Vec::<Violation>::new());
    }

    /// V42, PLANTED: a literal pipe in the text moves the field boundary,
    /// so the last field stops being citations. The row still LOOKS fine,
    /// which is why nothing caught it for eight rows of our own spec.
    #[test]
    fn v42_rejects_a_row_whose_literal_pipe_is_unescaped() {
        let text = real()
            .replace("T1|x|a task|V1", "T1|x|a task showing a `| M1 |` row|V1");
        let got = rows_escape_pipes(&text);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            got.first().is_some_and(|v| v.msg.contains("not 4")),
            "{got:?}"
        );
        assert!(got.first().is_some_and(Violation::is_mechanical), "{got:?}");
    }

    /// ...and ESCAPED, the same row is fine. This is the companion that
    /// keeps the rule about the ESCAPE rather than about the character:
    /// FORMAT.md permits a literal pipe, it requires it be written `\|`.
    #[test]
    fn v42_accepts_the_same_pipe_once_it_is_escaped() {
        let text = real()
            .replace("T1|x|a task|V1", "T1|x|a task showing a \\| row|V1");
        assert_eq!(rows_escape_pipes(&text), Vec::<Violation>::new());
    }

    /// The OTHER direction, which went unreported until a review asked for
    /// it: a row missing its last field emits an empty `cites`, which reads
    /// exactly like a row that cites nothing. That is B29's
    /// indistinguishability from the near side.
    #[test]
    fn v42_rejects_a_row_with_too_few_fields() {
        let text =
            real().replace("T1|x|a task|V1", "T1|x|a task with no cites");
        let got = rows_escape_pipes(&text);
        assert_eq!(got.len(), 1, "{got:?}");
        assert!(
            got.first()
                .is_some_and(|v| v.msg.contains("3 fields, not 4")),
            "{got:?}"
        );
        assert!(
            got.first().is_some_and(|v| !v.is_mechanical()),
            "adding a missing field is judgement: {got:?}"
        );
    }

    /// The companion that matters most: a well-formed spec is SILENT, and a
    /// row is judged only when it IS a row. A `\u{a7}V` statement carries no
    /// fields, and the milestone table's own rows open with a pipe rather
    /// than an id, so neither is this rule's business.
    #[test]
    fn v42_is_silent_on_rows_that_are_well_formed() {
        assert_eq!(rows_escape_pipes(&real()), Vec::<Violation>::new());
        let statement = "## \u{a7}V INVARIANTS\nV1: **a | b | c.**\n";
        assert_eq!(rows_escape_pipes(statement), Vec::<Violation>::new());
    }

    /// Order is part of the format, not a convention, because every `§S.n`
    /// address is read against it.
    #[test]
    fn v11_rejects_a_misordered_section() {
        let swapped =
            swap_lines(&real(), "## \u{a7}V INVARIANTS", "## \u{a7}T TASKS");
        v11_says(&swapped, "out of order");
    }

    /// V12, planted: the same id declared twice. And the companion that
    /// matters most here -- a GAP is not a violation, or every spec that ever
    /// retired a number would be red.
    #[test]
    fn v12_rejects_a_repeat_but_allows_a_gap() {
        let dup = real().replace("V3: **a gap", "V1: **a gap");
        assert!(
            ids_unique(&dup).iter().any(|v| v.msg.contains("`V1`")),
            "{:?}",
            ids_unique(&dup)
        );
        // real() declares V1 and V3 -- V2 is a gap, and it is clean.
        assert!(ids_unique(&real()).is_empty());
    }

    /// V13, planted: a citation to an invariant that was never declared.
    #[test]
    fn v13_rejects_a_dangling_citation() {
        let dangling = real().replace("T1|x|a task|V1", "T1|x|a task|V99");
        assert!(
            citations_resolve(&dangling)
                .iter()
                .any(|v| v.msg.contains("V99")),
            "{:?}",
            citations_resolve(&dangling)
        );
    }

    /// Everything a finding OFFERS, joined -- the message plus its ranked
    /// directions, which is where a repair lives.
    fn advice(found: &[Violation]) -> String {
        found
            .iter()
            .flat_map(|v| {
                std::iter::once(v.msg.clone())
                    .chain(v.directions.iter().map(|d| d.action.clone()))
            })
            .collect::<Vec<String>>()
            .join(" | ")
    }

    /// B26: on a line that names ANOTHER spec, V13 says the right thing and
    /// then offers two fixes that are both wrong -- point it elsewhere, or
    /// declare a rule that exists in the other file. `\u{a7}F` made that line
    /// an ordinary thing to write, so the branch that added the section owes
    /// the message.
    #[test]
    fn v13_names_the_cross_file_repair_when_the_line_names_a_spec() {
        let text = "## \u{a7}F FEDERATION\n\
                    - down: worker/SPEC.md V2 -- the rule it refines.\n";
        let got = citations_resolve(text);
        let first = advice(&got);
        assert!(first.contains("backticks"), "{first}");
        assert!(first.contains("V19"), "{first}");
    }

    /// The companion, and the one that keeps the advice from becoming noise:
    /// an ORDINARY dangling citation is told none of that. A rule that
    /// offered every repair on every finding would be a rule nobody reads to
    /// the end of.
    #[test]
    fn an_ordinary_dangling_citation_gets_no_cross_file_advice() {
        let text = "## \u{a7}V INVARIANTS\nV1: **a rule.** see V9\n";
        let got = citations_resolve(text);
        let first = advice(&got);
        assert!(!first.contains("backticks"), "{first}");
        assert!(first.contains("V9"), "{first}");
    }

    /// ...and the file name is read OUTSIDE backticks, like everything else
    /// V13 reads (V7). A spec DISCUSSING `worker/SPEC.md` in a code span is
    /// not naming a neighbour.
    #[test]
    fn a_spec_file_inside_backticks_does_not_trigger_the_advice() {
        let text = "## \u{a7}V INVARIANTS\n\
                    V1: **paths like `worker/SPEC.md` are literals.** see V9\n";
        let got = citations_resolve(text);
        let first = advice(&got);
        assert!(!first.contains("backticks"), "{first}");
    }

    /// V14, planted: two rows swapped. The companion is in `real()`, where
    /// `T2a` follows `T2` and precedes `T4` -- the case lexical ordering gets
    /// wrong, since "T2a" sorts before "T4" only by luck of the alphabet.
    #[test]
    fn v14_rejects_an_out_of_order_row() {
        let swapped = real().replace(
            "T1|x|a task|V1\nT2|.|another|V3",
            "T2|.|another|V3\nT1|x|a task|V1",
        );
        assert!(!rows_sorted(&swapped).is_empty());
        assert!(rows_sorted(&real()).is_empty());
    }

    /// V15 fails in three directions, so it is planted three times below. A
    /// guard that caught only the first would look just as green as one that
    /// caught all three -- and the first is the one that already happened.
    fn v15_says(text: &str, want: &str) {
        let got = tasks_in_one_milestone(text);
        assert!(
            got.iter().any(|v| v.msg.contains(want)),
            "want {want}: {got:?}"
        );
    }

    /// The measured failure: 88 tasks belonged to no milestone.
    #[test]
    fn v15_rejects_a_task_in_no_milestone() {
        let orphan = real().replace("| T1-T2, T4 |", "| T1-T2 |");
        v15_says(&orphan, "T4 is in no milestone");
    }

    /// The companion T15 added: a spec with NO milestone rows has not opted
    /// in, so the rule says nothing. It used to say `in no milestone` once
    /// per task -- 48 of 51 fleet specs, every row, for declining an optional
    /// feature. The planted case above proves it still fires where the author
    /// DID opt in, so this cannot be a check that was simply switched off.
    #[test]
    fn v15_is_silent_where_no_milestone_row_exists() {
        let none: String = real()
            .lines()
            .filter(|l| !l.starts_with("| M"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!uses_milestones(&none), "the fixture still has a row");
        assert_eq!(tasks_in_one_milestone(&none), Vec::<Violation>::new());
    }

    /// EXACTLY one, so two claims on one task is a violation too -- otherwise
    /// the rule is satisfied by claiming everything everywhere.
    #[test]
    fn v15_rejects_a_task_claimed_twice() {
        let twice = real().replace(
            "| M1 | first | T1-T2, T4 | done |",
            "| M1 | first | T1-T2, T4 | done |\n| M2 | second | T1 | done |",
        );
        v15_says(&twice, "claimed by two");
    }

    /// And the mirror: a milestone naming work that has no row, which is what
    /// a renamed or deleted task leaves behind.
    #[test]
    fn v15_rejects_a_milestone_claiming_a_missing_row() {
        let phantom = real().replace("| T1-T2, T4 |", "| T1-T2, T4, T9 |");
        v15_says(&phantom, "has no row");
    }

    /// Ranges are the affordance that makes the milestone column cheap, so
    /// they get their own test: `T1-T2, T4` is three tasks, not two tokens.
    #[test]
    fn a_range_expands_and_a_suffixed_row_rides_its_base() {
        assert_eq!(expand_cell("T1-T4, T12"), vec![1, 2, 3, 4, 12]);
        assert_eq!(expand_cell(" T7 "), vec![7]);
        assert_eq!(expand_cell(""), Vec::<u32>::new());
        // `T2a` has a row but is never claimed on its own -- it rides `T2`.
        assert!(tasks_in_one_milestone(&real()).is_empty());
    }

    fn records(pairs: &[(&str, &str)]) -> Vec<Record> {
        pairs
            .iter()
            .map(|(i, m)| ((*i).to_owned(), (*m).to_owned()))
            .collect()
    }

    /// V16, planted: the record is stripped out of the body that carried it.
    #[test]
    fn v16_rejects_a_record_that_was_edited_away() {
        let want = records(&[("V1", "a rule")]);
        assert!(records_survive(&real(), &want).is_empty(), "companion");
        let stripped = real().replace("V1: **a rule.**", "V1: **a thing.**");
        assert!(
            records_survive(&stripped, &want)
                .iter()
                .any(|v| v.msg.contains("V1 lost")),
            "{:?}",
            records_survive(&stripped, &want)
        );
    }

    /// The body is SCOPED to its own declaration. A marker that survived only
    /// by moving to a different rule has not survived: the decision it
    /// documented now hangs off something that never made it.
    #[test]
    fn a_record_that_moved_to_another_rule_does_not_count() {
        let moved = real()
            .replace("V1: **a rule.** cited by T1.", "V1: **moved.**")
            .replace("V3: **a gap", "V3: **a rule.** **a gap");
        let want = records(&[("V1", "a rule")]);
        assert!(!records_survive(&moved, &want).is_empty());
    }

    #[test]
    fn a_records_file_parses_with_comments_and_spaces() {
        let got = parse_records("# a note\n\nT6   DROPPED\nV24  a built pkg\n");
        assert_eq!(got, records(&[("T6", "DROPPED"), ("V24", "a built pkg")]));
    }

    /// V25, planted: a status outside the set. This is the rule the host
    /// checker had and microlith did not (B4), so it gets the same treatment
    /// as the rest -- a plant and a companion.
    #[test]
    fn v25_rejects_a_status_outside_the_set() {
        let bad = real().replace("T1|x|a task|V1", "T1|q|a task|V1");
        let got = statuses_valid(&bad);
        assert!(
            got.iter().any(|v| v.msg.contains("T1 has status `q`")),
            "{got:?}"
        );
        assert!(got.iter().any(|v| v.line > 0), "names the line: {got:?}");
    }

    /// The companion: all three real statuses are accepted, so the guard
    /// cannot pass by rejecting every row.
    #[test]
    fn v25_accepts_every_status_the_format_allows() {
        assert!(statuses_valid(&real()).is_empty(), "the fixture is clean");
        for s in STATUSES {
            let text = format!("## \u{a7}T TASKS\nT1|{s}|a task|V1\n");
            assert!(statuses_valid(&text).is_empty(), "rejected `{s}`");
        }
    }

    /// A milestone row has no status field and must not be read as though it
    /// did -- it opens with `|`, so it is not a task row at all.
    #[test]
    fn a_milestone_row_is_not_checked_for_a_status() {
        let text = "## \u{a7}T TASKS\n| M1 | scope | T1 | done |\n";
        assert!(statuses_valid(text).is_empty());
    }

    /// The classification is pinned, because it is advice about what an
    /// agent may do UNATTENDED and it regressed once already (B5). A
    /// direction is Mechanical only where the tool computes the single
    /// correct edit.
    #[test]
    fn only_computable_directions_are_mechanical() {
        let mech =
            |v: &[Violation]| v.first().is_some_and(Violation::is_mechanical);
        // The tool knows the header text, the order, and the sort key.
        // An ITEM with no header, since T15: bare prose is no longer a
        // violation at all, so a fixture without one proves nothing.
        assert!(mech(&sections_ordered("B1|d|c|f\n")), "V11");
        let unsorted = real().replace(
            "T1|x|a task|V1\nT2|.|another|V3",
            "T2|.|another|V3\nT1|x|a task|V1",
        );
        assert!(mech(&rows_sorted(&unsorted)), "V14");

        // These need a choice the tool cannot make.
        let dup = real().replace("V3: **a gap", "V1: **a gap");
        assert!(!mech(&ids_unique(&dup)), "V12 picks which id survives");
        let dangling = real().replace("T1|x|a task|V1", "T1|x|a task|V99");
        assert!(!mech(&citations_resolve(&dangling)), "V13 guesses intent");
        let bad = real().replace("T1|x|a task|V1", "T1|q|a task|V1");
        assert!(!mech(&statuses_valid(&bad)), "V25 knows the work, not us");
    }

    #[test]
    fn a_citation_is_found_next_to_punctuation() {
        let found = cited("cites (V21,V22) and V13. not Vx or V");
        assert_eq!(found, vec!["V21", "V22", "V13"]);
    }

    /// V13's boundary, planted from the three shapes that actually fired on
    /// this checker's first real run (B3): an example command, the rule's own
    /// illustration, and a qualified cross-project reference.
    #[test]
    fn a_backticked_mention_is_a_literal_not_a_citation() {
        assert!(cited("`grep V47` returned a fragment").is_empty());
        assert!(cited("a dangling `V99` points at nothing").is_empty());
        assert!(cited("name the repo (`itok's V82`)").is_empty());
    }

    /// ...and the boundary must not swallow real citations sharing a line
    /// with backticked text, which is the common case in this very spec.
    #[test]
    fn a_citation_beside_backticks_still_counts() {
        let line = "V5: the cap in `format.rs` is measured (V21), not taste";
        assert_eq!(cited(line), vec!["V5", "V21"]);
    }
}
