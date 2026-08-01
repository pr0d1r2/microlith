# nanokit

Enforce the [cavekit](FORMAT.md) `SPEC.md` format: lossless minify, structural check, anchors. CPU only -- no model, no network, no judgement.

## Why

The format had **two** hand-maintained implementations and no home. They disagreed, both gates stayed green, and 88 tasks belonged to no milestone before anything noticed. One implementation, or the rule is decoration.

`FORMAT.md` is normative and lives here, so a tool that cites it can read it.

## Install

```sh
cargo install nanokit
```

## Use

```sh
nanokit fmt --check SPEC.md   # gate: report drift, exit 1, never write
nanokit fmt SPEC.md           # rewrite: one line per statement
```

`fmt` joins hard wraps so each statement is one line, and refuses any line over the cap.

**Why one line per statement:** `grep` and every string-anchored edit are line-oriented, so a hard wrap defeats both *silently*. Measured on a real spec: 101 of 101 invariants spanned more than one line, so `grep V47` returned a fragment and a phrase crossing a wrap returned nothing at all -- while an edit whose anchor had been rewrapped did nothing and the test suite stayed green.

The cap is the opposite failure: base64, a minified blob, a pasted transcript. A line is the diff unit, so an edit anywhere in a long line re-sends the whole thing twice.

## Guarantees

- **Lossless, provably.** Normalizing all whitespace in the input and the output must yield identical strings -- asserted *before* any write. A formatter that can drop a fact is one nobody may run unattended.
- **Idempotent.** `fmt(fmt(x)) == fmt(x)`, tested, because a formatter that changes its mind cannot be a `--check` gate.
- **Deterministic.** No inference, no network. Every operation is a pure function of the text.
- **Self-guarding.** nanokit's own `SPEC.md` is the first file it formats, caps and checks.

## Status

`0.1.0` -- `fmt` is built. `check`, `anchors` and `derive` are specified in `SPEC.md` and not yet built; the binary says so rather than pretending otherwise.

## License

MIT.
