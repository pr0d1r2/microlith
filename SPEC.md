# nanokit — the cavekit SPEC format, enforced

Self-contained spec. `nanokit` owns the cavekit SPEC format and the CPU operations over it. It exists because that format had TWO hand-maintained implementations and no home: a checker in a host workspace that does not travel, and a ported copy in a consumer's test suite. They disagreed, both gates stayed green, and 88 tasks belonged to no milestone before anything noticed. One implementation, or the rule is decoration.

Lineage: the format is cavekit's, and every measurement below was taken in `itok`, the first consumer, which is public alongside this repo. A SEE-ALSO, ⊥ load-bearing (V19).

## §G GOAL

Keep a `SPEC.md` CORRECT, SMALL & ADDRESSABLE: enforce the format, minify it losslessly, derive its anchors -- CPU only, no model.

## §C CONSTRAINTS

- Rust. One bin: `nanokit`, plus the lib every consumer calls. MIT licensed.
- CPU ONLY. ⊥ inference, ⊥ network, ⊥ a model. Deterministic, or it does ⊥ ship.
- `FORMAT.md` is NORMATIVE & lives HERE. Every consumer reads THIS copy -- it was homeless (a plugin dir), which is half of why the two implementations drifted.
- Zero host-internal deps ∴ any repo adopts it as an ORDINARY dependency.
- The spec CAPS ITSELF from commit one, ⊥ after the growth has happened.

## §I INTERFACE

- `nanokit fmt [--check] <path>` — lossless minify: unwrap hard wraps, enforce the line cap. Default REWRITES; `--check` reports & exits 1 on drift (`rustfmt`'s grammar).
- `nanokit check <path>` — the structural rules: sections present & ordered (V11) · ids unique (V12) · citations resolve (V13) · rows sorted (V14) · every task in exactly one milestone (V15) · rejected-option records survive (V16).
- `nanokit anchors <path>` — derive the `§S.n` addressing FORMAT.md defines.
- `nanokit derive <path>` — sizes · citation graph · orphan invariants. Report-only.
- Exit: 0 ok · 1 drift or violation · 2 usage.

## §V INVARIANTS

V1: **`fmt` is LOSSLESS, and PROVABLY so.** Normalizing all whitespace in the input and in the output ! yield IDENTICAL strings, asserted BEFORE the write is allowed. A formatter that CAN drop a fact is one nobody may run unattended, & "I read the diff" is ⊥ a proof: an assertion from inspection has a measured record of being wrong, while a claim backed by a RUN needs no such gate. MEASURED: this exact assertion is what made a 1167→276-line transform safe to run on a 92kB law file.
V2: **`fmt` is IDEMPOTENT.** `fmt(fmt(x)) == fmt(x)`, tested, ∵ a formatter that keeps changing its mind cannot be a `--check` gate: CI would fail on the file the last run produced.
V3: **one line per STATEMENT; hard wrap is ⊥.** grep & every string-anchored edit are LINE-oriented ∴ a wrap defeats both SILENTLY. MEASURED: 101 of 101 invariants in a real spec spanned >1 line ∴ `grep V47` returned a FRAGMENT, a phrase crossing a wrap returned NOTHING, and an edit whose anchor had been rewrapped did nothing while the suite stayed green. Cost agrees: unwrapping saved 842 tokens against +38% on a one-word diff hunk (the line is the DIFF unit ∴ a paragraph re-sends TWICE), break-even ~7 edits per session.
V4: **the CONTINUATION rule is PAIRWISE, ⊥ per-line.** A wrap is a non-blank line FOLLOWING a non-blank line that opens nothing of its own. A blank-separated paragraph & the text under a header carry NO marker & are whole statements ∴ a per-line predicate rejects them -- which is exactly how the first version of this rule failed, on a spec's own intro paragraph.
V5: **the line CAP is the OPPOSITE failure & is set from MEASURED data.** Unbounded lines are base64 · minified blobs · a pasted transcript. Set the cap ABOVE the measured post-`fmt` maximum with real slack (~12%): a ceiling a hair above current turns every legitimate addition into a raise, & a threshold raised reflexively trains the reflex instead of catching the problem.
V6: **CPU ONLY -- inference stays a SKILL.** Every operation here is derivable: sizes, the citation graph, orphans, anchors, the unwrap. That makes them deterministic, free, offline & testable. A rewrite that needs judgement is a `/spec` call by a human or an agent, ⊥ a hidden model call inside a formatter.
V7: **ONE implementation; a consumer CALLS it, ⊥ re-ports it.** Two hand-maintained copies of one rule set is the defect this crate exists to end. A copy is permitted ONLY as a VERBATIM vendored file, re-synced on upgrade & marked as vendored.
V8: **`FORMAT.md` is normative & ships HERE.** A format cited by every command and living in nobody's repo is unreadable exactly when it is needed -- the drift that produced 88 orphan tasks was compounded by a dead reference nobody could follow, so a guess got made instead of a read.
V9: **the spec CAPS ITSELF, from commit ONE.** `.context-limits` carries a ceiling before the file grows into it. EVIDENCE: the first consumer set its first ceiling at 19,542 -- the size the file ALREADY was -- then raised it four times, to 27,500. A ceiling that arrives after the growth RATIFIES it rather than catching it.
V10: **report-only by DEFAULT; the gates are named & closed.** `fmt --check` and `check` gate; `derive`, `anchors` and bare `fmt` do not. A tool that rewrites law is one a user ! invoke deliberately.
V11: **sections are PRESENT & ORDERED**, per FORMAT.md: §G · §C · §I · §V · §T · §B. A lost header silently unnames every item under it.
V12: **ids are UNIQUE & NEVER REUSED; a GAP is fine.** A skipped number costs nothing; a REUSED one silently redirects every citation that pointed at the old meaning.
V13: **every citation RESOLVES.** A dangling `V99` is a pointer into nothing & it reads as authoritative -- the most expensive kind of wrong, ∵ nobody checks a reference that looks deliberate.
V14: **rows appear in SORTED id order, and a suffixed id RIDES its base** (`T30a` sorts after `T30`, ⊥ lexically). MEASURED: four rows sat out of order for weeks ∵ rows get appended wherever is convenient, and an out-of-order block RENDERS identically to a sorted one ∴ nothing but a check sees it.
V15: **every task belongs to EXACTLY ONE milestone**, read from the THIRD pipe field of an `| M<n> |` row, with RANGES expanded (`T1-T4, T12`). Named by the original checker as the rule most often broken and invisible without a check; it was right -- 88 tasks belonged to none while two gates stayed green. Ranges are the format's own affordance & the reason the mapping is cheap to maintain: not knowing about them is why the column once got judged a burden and deleted.
V16: **considered-and-REJECTED records survive.** An option recorded with its rejection & the TRIGGER that would reopen it is what makes a decision AUDITABLE rather than merely obeyable. Compaction ! never trade these for bytes. Checked by NAMED records, ⊥ a word count: an arbitrary threshold either fires on nothing or fires on prose edits that changed no decision.
V17: **a rule and its CHECKER land in ONE commit.** A rule with no runner is a LIE that reads as law -- worse than no rule, ∵ it provides false assurance. And a checker landing on a file that violates it is RED AT BIRTH ∴ the rewrite ships with it.
V18: **a guard is proven by PLANTING a violation**, ⊥ by reading it. A guard that has never rejected anything is indistinguishable from one that CANNOT. Every check here ships with its planted counter-example AND a companion that accepts every real shape, so it cannot pass by rejecting everything.
V19: **a cross-project reference is QUALIFIED & SEE-ALSO.** Name the repo (`itok's V82`), never a bare `V82` -- this spec owns that namespace, so an unqualified number reads as OURS & breaks. And every rule stands on evidence RESTATED here: a reader ! never fetch another repo to understand a rule, even when that repo is public. Lineage lives in git history & the commit trail.
V20: **PUBLISH before DEPEND.** A consumer takes this crate from a REGISTRY, ⊥ a relative path: a path dep dangles the moment either repo is extracted or moved, which is the exact wart it would be adopted to remove. Until published, a consumer keeps its own copy & uses `nanokit` as a SECOND OPINION -- two implementations by design, with drift visible, beats one implementation that cannot be built.

## §T TASKS

| id | scope | tasks | done-when |
|----|-------|-------|-----------|
| M1 | the formatter -- the capability that exists nowhere else | T1-T3 | `nanokit fmt --check SPEC.md` green on this repo, losslessness asserted before every write |
| M2 | the checker -- one implementation of the format rules | T4-T5 | `nanokit check` enforces V11-V16, each proven by a planted violation |
| M3 | adoption -- consumers stop carrying copies | T6-T8 | gate green, published, the first consumer depends on the registry crate and deletes its ported copy |

T1|x|`fmt` core: unwrap hard wraps, pairwise continuation rule, line cap; lossless proof asserted BEFORE the write; idempotence tested|V1,V2,V3,V4,V5
T2|x|`fmt --check`: report drift & exit 1, ⊥ rewrite; the `rustfmt`/`nixfmt` grammar so it needs no teaching|V10,V2
T3|x|`.context-limits` at 4k from commit one + `tests/dogfood.rs`: this SPEC.md is formatted, under the cap, and carries real slack -- asserted against the library being built|V9
T4|.|`check` verb: V11-V16 as pure fns over `&str`, each with a PLANTED violation and a companion accepting every real shape. Port from the consumer's `tests/spec_integrity.rs`, restating the rules HERE so the pointer is not load-bearing once that copy is deleted. Verify: `nanokit check SPEC.md` exit 0 on this repo and exit 1 on each planted case|V7,V11,V12,V13,V14,V15,V16,V18
T5|.|`derive` + `anchors`: invariant sizes · citation counts · orphan invariants (uncited, non-working) · `§S.n` addressing. Report-only, CPU only. Sizing is now trivial ∵ one line per statement means an invariant IS a line (V3)|V6,V10
T6|.|the traveling gate: `hk.pkl` + vendored `pkl/` + `flake.nix` + `.github/workflows/ci.yml`, copied from the first consumer's proven skeleton -- fmt · clippy · nextest · doctest · coverage floor, orchestration ONLY in the workflow so the ops cannot drift. Born standalone ∴ no extraction rehearsal is owed. Verify: `hk check --all` exit 0|V17
T7|.|absorb the host checker & the consumer's `tools/specfmt`; both are DELETED in the same commit that replaces them, ⊥ left as a third copy|V7,V17
T8|.|publish, then the consumer depends on the registry crate and drops its ported guards -- in that order (V20). Smaller and zero-dep ∴ this is also the publish REHEARSAL for the larger consumer's own release|V20,V7

## §B BUGS

id|date|cause|fix
