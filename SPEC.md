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

V21: **a threshold is MEASURED HERE, and a FLOOR ratchets while a CEILING breathes.** Copying a number from another project is the classic failure: the first consumer's gate floors coverage at 98, and this crate's own measures 94.68% (format.rs 100 · lib.rs 94.44 · main.rs 89.91) ∴ a copied floor would have failed this repo on commit ONE. The two directions take OPPOSITE slack & are ⊥ one discipline: a size CEILING wants ~12% headroom ∵ a ceiling a hair above current turns every legitimate addition into a raise; a coverage FLOOR wants ~1 point ∵ it is a RATCHET, and a floor far below current gates nothing. NAME the gap rather than hiding it: `main()` and the I/O arms are structurally unreachable from lib tests, & only a process-boundary test moves them.
V22: **ONE gate definition, many callers; the workflow holds ORCHESTRATION only.** `hk.pkl` owns WHICH command, WHICH flags, WHICH order & the coverage floor; `ci.yml` owns the runner, the toolchain & the checkout ∵ that half has no local equivalent to disagree with. ∴ pre-commit, pre-push & CI reach ONE copy and the ops CANNOT drift. ELIMINATION beats policing -- a drift guard only fires AFTER someone has written the second copy. The gate of record stays REPRODUCIBLE with plain cargo: every step is a command a contributor can paste with nothing but rustup ∴ the hook runner is a CONVENIENCE, ⊥ a dependency of the crate, and no verdict may rest on its own logic.
V23: **a hook DEGRADES when its runner is absent; it ⊥ BLOCKS.** A gate exists to stop bad commits, ⊥ to stop git. Runner missing -- a fresh clone, outside the dev shell, a container -- ⇒ ONE line to stderr and exit 0: LOUD about being skipped, ∵ a SILENTLY skipped gate is indistinguishable from a passing one. Hard-failing instead makes the repo unusable for anyone who has ⊥ installed the toolchain, the wrong failure mode for a convenience when the gate of record is CI (V22). ∴ the hook command is written BY HAND: an installer writes a command assuming its own binary is on PATH, and outside the shell it is not, so every git command in the repo fails rather than merely running ungated.

## §T TASKS

| id | scope | tasks | done-when |
|----|-------|-------|-----------|
| M1 | the formatter -- the capability that exists nowhere else | T1-T3 | `nanokit fmt --check SPEC.md` green on this repo, losslessness asserted before every write |
| M2 | the gate -- this repo defends itself before it grows | T6 | `hk check --all` exit 0, hooks installed by shell entry, coverage floor measured HERE |
| M3 | the checker -- one implementation of the format rules | T4-T5 | `nanokit check` enforces V11-V16, each proven by a planted violation |
| M4 | adoption -- consumers stop carrying copies | T7-T8 | published, the first consumer depends on the registry crate and deletes its ported copy |

T1|x|`fmt` core: unwrap hard wraps, pairwise continuation rule, line cap; lossless proof asserted BEFORE the write; idempotence tested|V1,V2,V3,V4,V5
T2|x|`fmt --check`: report drift & exit 1, ⊥ rewrite; the `rustfmt`/`nixfmt` grammar so it needs no teaching|V10,V2
T3|x|`.context-limits` at 4k from commit one + `tests/dogfood.rs`: this SPEC.md is formatted, under the cap, and carries real slack -- asserted against the library being built|V9
T4|.|`check` verb: V11-V16 as pure fns over `&str`, each with a PLANTED violation and a companion accepting every real shape. Port from the consumer's `tests/spec_integrity.rs`, restating the rules HERE so the pointer is not load-bearing once that copy is deleted. Verify: `nanokit check SPEC.md` exit 0 on this repo and exit 1 on each planted case|V7,V11,V12,V13,V14,V15,V16,V18
T5|.|`derive` + `anchors`: invariant sizes · citation counts · orphan invariants (uncited, non-working) · `§S.n` addressing. Report-only, CPU only. Sizing is now trivial ∵ one line per statement means an invariant IS a line (V3)|V6,V10
T6|x|the gate, RUN & green: `hk check --all` exit 0 · `nix build` · a fresh shell installs `pre-commit`+`pre-push` · hk absent ⇒ ONE stderr line, exit 0 (proven by PLANTING an empty PATH, V18). Coverage re-measured AFTER the gate existed -- 94.68% lines, unmoved by doctest/rustdoc ∴ floor 94 stands as a measurement, ⊥ a prediction (V21). `pkl/Config.pkl` is vendored VERBATIM ∴ exempt from every formatter AND from `typos`: upstream misspells a word in a doc comment & the fix would FORK the bytes, turning the next re-sync into a conflict. `actionlint` landed WITH the live workflow, ⊥ before it. DROPPED as consumer-shaped, ⊥ forgotten: the cassette/network axis · `--no-default-features` (no features here) · `package` (nothing published until T8) · `shellcheck` (no shell scripts -- a linter with nothing to lint is a rule with no runner) · `fetch-depth: 0` (no test reads `HEAD~n`)|V21,V22,V23,V10,V18
T7|.|absorb the host checker & the consumer's `tools/specfmt`; both are DELETED in the same commit that replaces them, ⊥ left as a third copy|V7,V17
T8|.|publish, then the consumer depends on the registry crate and drops its ported guards -- in that order (V20). Smaller and zero-dep ∴ this is also the publish REHEARSAL for the larger consumer's own release|V20,V7

## §B BUGS

id|date|cause|fix
B1|2026-08-01|the gate's FIRST real run rejected the comment that exempted the vendored tree from it: the comment QUOTED upstream's misspelling as evidence, and `typos` scans `hk.pkl` too ∴ the exemption's justification became a new violation, one file further out. A rule that names its own trigger literally REINTRODUCES it|name the defect DESCRIPTIVELY (`a misspelling of "precedence" at Config.pkl:201`), ⊥ by quoting it, & say in the comment WHY it is not quoted -- otherwise the next reader restores the quote as a courtesy. Found by running the gate, ⊥ by reading it (V18)
B2|2026-08-01|INSERTING V4 (the pairwise rule) renumbered V4-V9 upward, and every citation IN THE CODE kept its old number ∴ 14 doc comments pointed one rule off -- the cap cited as the continuation rule, `ONE implementation` cited as `CPU only`. They still RESOLVE, so ⊥ a dangling-reference check can see them, & each reads as deliberate. Retiring old V10 (`nanokit DOGFOODS itself`) in the same edit orphaned its TWO runners: `tests/dogfood.rs` & hk's `nanokit` step enforced a rule with no text -- the inverse of V17|APPEND, ⊥ insert: a new rule takes the NEXT free number so no existing citation moves (V12 already says this for reuse; the same argument forbids insertion). Retargeted all 14 & restored the dogfood rule as V24. Found by AUDIT, ⊥ by a gate -- `check` (T4) enforces V13 over SPEC.md, & a citation living in a `.rs` doc comment is outside every checker this crate plans ∴ code citations stay a REVIEW obligation, named here so that is a known limit rather than an assumed cover
