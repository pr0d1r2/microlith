# cavespec

Mechanical operations on a caveman [`SPEC.md`](FORMAT.md): lossless minify, structural check, derived reports. CPU only -- no model, no network, no judgement.

## Why

The format had **two** hand-maintained implementations and no home. They disagreed, both gates stayed green, and 88 tasks belonged to no milestone before anything noticed. One implementation, or the rule is decoration.

[`FORMAT.md`](FORMAT.md) is [cavekit](https://github.com/JuliusBrussee/cavekit)'s, vendored here **verbatim** so a tool that cites the format can read it. What cavespec *enforces* lives in its own `SPEC.md` under §V, and that set is a superset -- the losslessness proof, idempotence, the line cap, citation resolution, row order and the survival of rejected-option records have no counterpart upstream. The format is an input, not the identity.

## Install

```sh
cargo install cavespec
```

## Use

Every verb, its synopsis and its exit codes are in [Commands](#commands) below -- generated from the binary, so the list cannot go stale. What follows is why each one exists.

The path defaults to `SPEC.md` -- the one file [FORMAT.md](FORMAT.md) says every command reads -- so run from a project root and omit it. Pass a path for anything else.

Silence is success. Every command is quiet when there is nothing to report; `--verbose` opts out of that. What it prints differs by command: `fmt` and `check` are mute when clean, so it confirms what was examined; `derive` and `anchors` already speak, so it deepens -- per-statement sizes biggest first, and full text instead of a truncated gist.

### `fmt` -- one line per statement

`fmt` joins hard wraps so each statement is one line, and refuses any line over the cap.

**Why:** `grep` and every string-anchored edit are line-oriented, so a hard wrap defeats both *silently*. Measured on a real spec: 101 of 101 invariants spanned more than one line, so `grep V47` returned a fragment and a phrase crossing a wrap returned nothing at all -- while an edit whose anchor had been rewrapped did nothing and the test suite stayed green.

The cap is the opposite failure: base64, a minified blob, a pasted transcript. A line is the diff unit, so an edit anywhere in a long line re-sends the whole thing twice.

### `check` -- the structural rules

Sections present and in order, ids unique, citations resolving, rows sorted, every task in exactly one milestone, and every task status one of `.` `~` `x`.

Each violation says where it is and what to do about it:

```
SPEC.md:22: cavespec/V13: `V14` is cited but never declared
    why: a dangling reference reads as authoritative, so nobody follows it
    mechanical: point it at the rule that was meant
    judgment: declare V14, if the rule is real but missing
```

`file:line:` first, so an editor and `grep -n` both jump to it. The rule id is **qualified** and sits beside the message, never inside the coordinates: every consumer is a caveman spec numbering from `V1`, so a bare `V13` printed against *your* `SPEC.md` names one of your rules instead of one of cavespec's.

A **mechanical** direction is deterministic and reversible, so an agent may apply it unattended. A **judgment** direction accepts a regression or changes intent -- an agent that applies one blindly has silenced the guardrail rather than fixed the defect. `--format json` emits the same anatomy with `kind` as data, so an agent never has to parse prose.

`--records <file>` adds the rejected-option check. Its baseline lives with the spec being checked rather than in cavespec, because "this record still exists" is a claim about edits over time and no single file can answer it.

### `derive` and `anchors` -- report only

These answer questions rather than passing judgement, so they **exit 0 even with findings**. An invariant nothing cites might be a dead rule or a missing citation, and only a reader can say which; a gate would answer by fiat.

`derive` reports statement sizes, how often each invariant is cited, and which are cited by nothing. `anchors` lists the `§S.n` address of every item alongside the id it currently resolves to -- addresses are ordinal, so retiring an id shifts every address below it, and printing both makes that visible instead of surprising.

<!-- BEGIN cavespec docs -->
## Commands

Every verb, its synopsis and what it does. Regenerate with `cavespec docs`.

### `fmt`

```text
fmt [--check] [--verbose] [<path>]
```

One line per statement: joins hard wraps and enforces the line cap. Rewrites the file; `--check` reports drift and exits 1 instead. The transform is proven whitespace-only before any write. `--verbose` confirms what was examined, with the longest line against the cap.

### `check`

```text
check [--records <file>] [--format human|json] [--verbose] [<path>]
```

The structural rules: sections present and ordered, ids unique, citations resolve, rows sorted, every task in exactly one milestone, every status one of `.` `~` `x`. Each violation carries a line and a ranked fix, marked mechanical (safe to apply unattended) or judgment (needs a human). `--records` adds the rejected-option check, whose baseline the caller owns because survival is a claim about edits rather than about the file. `--verbose` confirms what was examined, and says when the records check did not run.

### `migrate`

```text
migrate [--check] [--verbose] [<path>]
```

Section headers to canonical 4.1.0. A case or punctuation difference is rewritten silently; a label carrying real text is rewritten with the original kept beneath it, so nothing is discarded. Every alphanumeric run of the original is proven to survive before any write. A letter used for a DIFFERENT concept is never touched -- annotating one keeps the characters and inverts the meaning -- so those are reported and exit 1. `--check` reports without writing.

### `derive`

```text
derive [--verbose] [<path>]
```

Sizes, the citation graph, invariants cited by nothing, and statements said twice. Report-only: exits 0 even with findings, because an orphan is a question for a reader, not a build failure. `--verbose` adds every statement's size, biggest first: what to cut.

### `anchors`

```text
anchors [--verbose] [<path>]
```

The section address of every item, with the id it resolves to and whether the two have drifted apart. Report-only. `--verbose` prints each item in full, not a 60-char gist.

### `docs`

```text
docs
```

Print this command reference as markdown -- the source for README's generated block, kept in sync by a test. Report-only: it writes to stdout, never to the README.

## Exit codes

| code | meaning |
|------|---------|
| `0` | clean |
| `1` | drift, or a violation the command gates on |
| `2` | usage error |
<!-- END cavespec docs -->

The command reference above is **generated**: `cavespec docs` prints it, and a test fails if this block drifts from the code. Edit the registry in `src/docs.rs`, never the block by hand.

## Use it as a library

The binary is a thin shell. Every rule is a pure function over `&str`, so a consumer calls the rule instead of porting it:

```rust
let violations = cavespec::check_spec(&text, &records);
for v in &violations {
    println!("{}: {} ({})", v.rule, v.msg, v.line);
}
```

## Guarantees

- **Lossless, provably.** Normalizing all whitespace in the input and the output must yield identical strings -- asserted *before* any write. A formatter that can drop a fact is one nobody may run unattended.
- **Idempotent.** `fmt(fmt(x)) == fmt(x)`, tested, because a formatter that changes its mind cannot be a `--check` gate.
- **Deterministic.** No inference, no network, and **zero dependencies**. Every operation is a pure function of the text.
- **Self-guarding.** cavespec's own `SPEC.md` is the first file it formats, caps and checks, using the binary being built. A rule this crate cannot pass is a rule it cannot ship.
- **Every guard is proven by a planted violation**, never by reading it, plus a companion proving it accepts every real shape -- so no check can pass by rejecting everything.

## Status

`0.4.0`, unpublished. A minor version here is a level of **guarantee**, not a feature count: `0.4` means the rules hold on real-world markdown rather than only on this repo's own file. `SPEC.md` §V.30 carries the ladder; `0.5` is the public rung.

All five commands are built and gate this repo's own spec.

`SPEC.md` records what remains, what was deliberately dropped, and every bug found so far with the rule that now catches it.

## License

MIT.
