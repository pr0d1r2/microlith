# nanokit

Enforce the [cavekit](FORMAT.md) `SPEC.md` format: lossless minify, structural check, derived reports. CPU only -- no model, no network, no judgement.

## Why

The format had **two** hand-maintained implementations and no home. They disagreed, both gates stayed green, and 88 tasks belonged to no milestone before anything noticed. One implementation, or the rule is decoration.

`FORMAT.md` is normative and lives here, so a tool that cites it can read it.

## Install

```sh
cargo install nanokit
```

## Use

```sh
nanokit fmt --check SPEC.md    # gate: report drift, exit 1, never write
nanokit fmt SPEC.md            # rewrite: one line per statement
nanokit check SPEC.md          # gate: the structural rules
nanokit derive SPEC.md         # report: sizes, citations, orphans
nanokit anchors SPEC.md        # report: the §S.n address of every item
```

Exit codes are the same everywhere: `0` clean, `1` a violation the command gates on, `2` a usage error.

### `fmt` -- one line per statement

`fmt` joins hard wraps so each statement is one line, and refuses any line over the cap.

**Why:** `grep` and every string-anchored edit are line-oriented, so a hard wrap defeats both *silently*. Measured on a real spec: 101 of 101 invariants spanned more than one line, so `grep V47` returned a fragment and a phrase crossing a wrap returned nothing at all -- while an edit whose anchor had been rewrapped did nothing and the test suite stayed green.

The cap is the opposite failure: base64, a minified blob, a pasted transcript. A line is the diff unit, so an edit anywhere in a long line re-sends the whole thing twice.

### `check` -- the structural rules

Sections present and in order, ids unique, citations resolving, rows sorted, every task in exactly one milestone, and every task status one of `.` `~` `x`.

Each violation says where it is and what to do about it:

```
SPEC.md:V13:22: `V14` is cited but never declared
    why: a dangling reference reads as authoritative, so nobody follows it
    mechanical: point it at the rule that was meant
    judgment: declare V14, if the rule is real but missing
```

A **mechanical** direction is deterministic and reversible, so an agent may apply it unattended. A **judgment** direction accepts a regression or changes intent -- an agent that applies one blindly has silenced the guardrail rather than fixed the defect. `--format json` emits the same anatomy with `kind` as data, so an agent never has to parse prose.

`--records <file>` adds the rejected-option check. Its baseline lives with the spec being checked rather than in nanokit, because "this record still exists" is a claim about edits over time and no single file can answer it.

### `derive` and `anchors` -- report only

These answer questions rather than passing judgement, so they **exit 0 even with findings**. An invariant nothing cites might be a dead rule or a missing citation, and only a reader can say which; a gate would answer by fiat.

`derive` reports statement sizes, how often each invariant is cited, and which are cited by nothing. `anchors` lists the `§S.n` address of every item alongside the id it currently resolves to -- addresses are ordinal, so retiring an id shifts every address below it, and printing both makes that visible instead of surprising.

## Use it as a library

The binary is a thin shell. Every rule is a pure function over `&str`, so a consumer calls the rule instead of porting it:

```rust
let violations = nanokit::check_spec(&text, &records);
for v in &violations {
    println!("{}: {} ({})", v.rule, v.msg, v.line);
}
```

## Guarantees

- **Lossless, provably.** Normalizing all whitespace in the input and the output must yield identical strings -- asserted *before* any write. A formatter that can drop a fact is one nobody may run unattended.
- **Idempotent.** `fmt(fmt(x)) == fmt(x)`, tested, because a formatter that changes its mind cannot be a `--check` gate.
- **Deterministic.** No inference, no network, and **zero dependencies**. Every operation is a pure function of the text.
- **Self-guarding.** nanokit's own `SPEC.md` is the first file it formats, caps and checks, using the binary being built. A rule this crate cannot pass is a rule it cannot ship.
- **Every guard is proven by a planted violation**, never by reading it, plus a companion proving it accepts every real shape -- so no check can pass by rejecting everything.

## Status

`0.1.0`, unpublished. All four commands are built and gate this repo's own spec.

`SPEC.md` records what remains, what was deliberately dropped, and every bug found so far with the rule that now catches it.

## License

MIT.
