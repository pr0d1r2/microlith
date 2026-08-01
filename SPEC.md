# nanokit — the cavekit SPEC format, enforced

Self-contained spec. `nanokit` owns the cavekit SPEC format and the CPU operations over it. It exists because that format had TWO hand-maintained implementations and no home: `host-checker` in a host workspace that does not travel, and a ported copy in itok's test suite. They disagreed, both gates stayed green, and 88 tasks belonged to no milestone (itok's B12). One implementation, or the rule is decoration.

## §G GOAL

Keep a `SPEC.md` CORRECT, SMALL & ADDRESSABLE: enforce the format, minify it losslessly, derive its anchors -- CPU only, no model.

## §C CONSTRAINTS

- Rust. One bin: `nanokit`, plus the lib every consumer calls. MIT licensed.
- CPU ONLY. ⊥ inference, ⊥ network, ⊥ a model. Deterministic, or it does ⊥ ship.
- `FORMAT.md` is NORMATIVE & lives HERE. Every consumer reads THIS copy -- it was homeless (a plugin dir), which is half of why B12 happened.
- Zero host-internal deps ∴ any repo adopts it as an ORDINARY dependency.
- The spec CAPS ITSELF from commit one, ⊥ after the growth has happened.

## §I INTERFACE

- `nanokit fmt [--check] <path>` — lossless minify: unwrap hard wraps, enforce the line cap. Default REWRITES; `--check` reports & exits 1 on drift (`rustfmt`'s grammar).
- `nanokit check <path>` — structure: sections present & ordered · ids unique · citations resolve · task rows sorted · every task in exactly one milestone.
- `nanokit anchors <path>` — derive the `§S.n` addressing FORMAT.md defines.
- `nanokit derive <path>` — sizes · citation graph · orphan invariants. Report-only.
- Exit: 0 ok · 1 drift or violation · 2 usage.

## §V INVARIANTS

V1: **`fmt` is LOSSLESS, and PROVABLY so.** Normalizing all whitespace in the input and in the output ! yield IDENTICAL strings, asserted BEFORE the write is allowed. A formatter that CAN drop a fact is one nobody may run unattended, & "I read the diff" is ⊥ a proof (itok's V82: an assertion from inspection has a measured record of being wrong). MEASURED: this exact assertion is what made itok's 1167→276-line transform safe to run on its own law file.
V2: **`fmt` is IDEMPOTENT.** `fmt(fmt(x)) == fmt(x)`, tested, ∵ a formatter that keeps changing its mind cannot be a `--check` gate: CI would fail on a file the last run produced.
V3: **one line per STATEMENT; hard wrap is ⊥.** grep & every string-anchored edit are LINE-oriented ∴ a wrap defeats both SILENTLY. MEASURED on itok: 101 of 101 invariants spanned >1 line ∴ `grep V47` returned a FRAGMENT, a phrase crossing a wrap returned NOTHING, and an edit whose anchor had been rewrapped did nothing while the suite stayed green. A blank-separated paragraph & text under a header are STATEMENTS too -- the property is PAIRWISE (a non-blank line following a non-blank line, opening nothing of its own), ⊥ per-line, which is how the first version of this guard rejected a legitimate intro paragraph.
V4: **the line CAP is the OPPOSITE failure & is set from MEASURED data.** Unbounded lines are base64 · minified blobs · a pasted transcript, and the line is the DIFF unit ∴ an edit anywhere in a long line re-sends the whole thing TWICE. Set the cap ABOVE the measured post-`fmt` maximum with real slack (~12%) -- a ceiling a hair above current turns every legitimate addition into a raise, & a threshold raised reflexively trains the reflex instead of catching the problem.
V5: **CPU ONLY -- inference stays a SKILL.** Every operation here is derivable: sizes, the citation graph, orphans, anchors, the unwrap. That makes them deterministic, free, offline & testable. A rewrite that needs judgement is a `/spec` call by a human or an agent, ⊥ a hidden model call inside a formatter.
V6: **ONE implementation; a consumer CALLS it, ⊥ re-ports it.** Two hand-maintained copies of one rule set is the defect this crate exists to end -- MEASURED: itok's ported guards and `host-checker` disagreed, and the rule they disagreed about (every task in exactly one milestone) was violated 88 times with both gates green. A copy is permitted ONLY as a VERBATIM vendored file, re-synced on upgrade & marked as vendored.
V7: **`FORMAT.md` is normative & ships HERE.** A format cited by every command and living in nobody's repo is unreadable exactly when it is needed: itok's B12 names "FORMAT.md is ABSENT from both repos while every spec command cites it" as the reason a guess was made instead of a read.
V8: **the spec CAPS ITSELF, from commit ONE.** `.context-limits` carries a ceiling before the file grows into it. EVIDENCE: itok set its first ceiling at 19,542 -- the size the file ALREADY was -- and then raised it four times to 27,500. A ceiling that arrives after the growth ratifies it rather than catching it.
V9: **report-only by DEFAULT; the gates are named & closed.** `fmt --check` and `check` gate; `derive`, `anchors` and bare `fmt` do not. A tool that rewrites law is one a user ! invoke deliberately.
V10: **nanokit DOGFOODS itself.** Its own `SPEC.md` is the first file it formats, caps & checks ∴ a rule it cannot pass is a rule it may not ship. The gate runs `nanokit fmt --check SPEC.md && nanokit check SPEC.md` against the binary being built.

## §T TASKS

| id | scope | tasks | done-when |
|----|-------|-------|-----------|
| M1 | the formatter -- the capability that exists nowhere else | T1-T3 | `nanokit fmt --check SPEC.md` green on this repo, losslessness asserted before every write |
| M2 | the checker -- one implementation of the format rules | T4-T5 | `nanokit check` reproduces every rule itok and host-checker each carried separately |
| M3 | adoption -- consumers stop carrying copies | T6-T8 | published, itok depends on it with no path dep, host-checker and tools/specfmt deleted |

T1|.|`fmt` core: unwrap hard wraps to one line per statement, pairwise continuation rule, line cap; lossless proof asserted BEFORE the write; idempotence tested|V1,V2,V3,V4
T2|.|`fmt --check`: report drift & exit 1, ⊥ rewrite; the `rustfmt`/`nixfmt` grammar so it needs no teaching|V9,V2
T3|.|`.context-limits` + the dogfood gate: this SPEC.md capped at 4k from commit one, `fmt --check` + `check` run against the binary being built|V8,V10
T4|.|`check`: port the structural guards from itok's `tests/spec_integrity.rs` -- sections ordered · ids unique · citations resolve · rows sorted · every task in exactly one milestone. ONE implementation, and each rule proven by planting a violation|V6,V7
T5|.|`derive` + `anchors`: sizes · citation graph · orphan invariants · `§S.n` addressing. Report-only, CPU only|V5,V9
T6|.|the traveling gate: `hk.pkl` + `flake.nix` + `ci.yml` + baselines, copied from itok's proven skeleton -- born standalone ∴ no extraction rehearsal is owed|V10
T7|.|absorb `host-checker` (host crate) & itok's `tools/specfmt`; both are DELETED in the same commit that replaces them, ⊥ left as a third copy|V6
T8|.|publish so a consumer depends on a REGISTRY crate, ⊥ a path dep that dangles after extraction; only then does itok drop its ported copies|V6

## §B BUGS

id|date|cause|fix
