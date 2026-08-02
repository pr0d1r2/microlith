# microlith (mth)

[![CI](https://github.com/pr0d1r2/microlith/actions/workflows/ci.yml/badge.svg)](https://github.com/pr0d1r2/microlith/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/microlith.svg)](https://crates.io/crates/microlith)
[![docs.rs](https://docs.rs/microlith/badge.svg)](https://docs.rs/microlith)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![edition 2021](https://img.shields.io/badge/edition-2021-000000?logo=rust&logoColor=white)](Cargo.toml)
[![MSRV 1.82](https://img.shields.io/badge/MSRV-1.82-000000?logo=rust&logoColor=white)](Cargo.toml)
[![dependencies 0](https://img.shields.io/badge/dependencies-0-brightgreen)](Cargo.toml)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-brightgreen)](Cargo.toml)
[![gate hk](https://img.shields.io/badge/gate-hk-6E4AFF)](hk.pkl)
[![coverage floor](https://img.shields.io/badge/coverage_floor-%E2%89%A594%25-brightgreen)](hk.pkl)

[![nix flake](https://img.shields.io/badge/nix-flake-5277C3?logo=nixos&logoColor=white)](flake.nix)
[![nixpkgs 26.11](https://img.shields.io/badge/nixpkgs-26.11-5277C3?logo=nixos&logoColor=white)](flake.lock)
[![platforms](https://img.shields.io/badge/platforms-linux_(x86__64,_aarch64)_%C2%B7_macos_(aarch64)-5277C3?logo=nixos&logoColor=white)](flake.nix)

[![built with Claude Code](https://img.shields.io/badge/built_with-Claude_Code-D97757)](https://claude.com/claude-code)
[![built with Opus 5](https://img.shields.io/badge/built_with-Opus_5-D97757)](https://www.anthropic.com/claude)
[![built with SDD](https://img.shields.io/badge/built_with-spec--driven_development-D97757)](SPEC.md)

> ### Built by an LLM, deliberately and in the open
>
> This repository — code, spec, tests and prose — was written by [Claude Code](https://claude.com/claude-code) running Anthropic's **Claude Opus 5**. Most commits carry a `Co-Authored-By: Claude Opus 5` trailer, and every commit is signed. A human owns every decision, reviews every diff, and is accountable for what ships.
>
> **The method is spec-driven development.** [`SPEC.md`](SPEC.md) is the law rather than a description written afterwards: it holds the invariants that must stay true, the tasks that remain, and a record of every bug found so far paired with the rule that now catches it. A rule and its checker land in the *same commit*, because a rule with no runner gates nothing. Every guard is proven by planting the violation it exists to catch, plus a companion proving it still accepts every real shape.
>
> **Integration and guardrails run in git hooks, not in CI.** Entering the dev shell (`nix develop`, or `direnv allow`) installs `pre-commit` and `pre-push`, which run [hk](https://github.com/jdx/hk) against one definition of the gate in [`hk.pkl`](hk.pkl). Pre-commit takes the fast set — format, clippy, tests, and the tool checking its own spec. Pre-push adds the expensive axes: doctests, rustdoc, and `cargo llvm-cov` against a **94% line-coverage floor** — a ratchet, not a target; at `0.4.0` the suite sits at 98.56% lines over 176 tests. [`ci.yml`](.github/workflows/ci.yml) is a second caller of that same definition, never a second copy of it.
>
> Deeper: [`AGENTS.md`](AGENTS.md) is the working guide · [`CONTRIBUTING.md`](CONTRIBUTING.md) is the loop · [`SPEC.md`](SPEC.md) is what must hold and what remains · [`FORMAT.md`](FORMAT.md) is the format itself.

Mechanical operations on a [caveman](https://github.com/JuliusBrussee/caveman) [`SPEC.md`](FORMAT.md): lossless minify, structural check, derived reports. CPU only -- no model, no network, no judgement.

## Why

Some projects keep a single `SPEC.md` at the root: what the software must do, what must stay true, and what is left to build. The AI coding agent reads it before it works and updates it afterwards, so that file -- not the chat log -- is the durable memory. The practice is called **spec-driven development**.

[cavekit](https://github.com/JuliusBrussee/cavekit) defines a format for that file. [caveman](https://github.com/JuliusBrussee/caveman) is the compressed notation it is written in, trading English for symbols so the same meaning costs fewer tokens. A spec in that format looks roughly like this:

```text
## §V INVARIANTS
V1: every write is proven lossless BEFORE it happens.
V2: formatting twice changes nothing the second time.

## §T TASKS
| id | scope    | tasks | done-when            |
|----|----------|-------|----------------------|
| M1 | the core | T1-T2 | V1 holds under test  |
T1|x|prove the transform keeps every word|V1
T2|.|make the formatter idempotent|V2
```

Statements carry ids (`V1`, `T2`). Rows cite other ids (the trailing `|V1`). Tasks claim a milestone (`M1`) and a status (`x` done, `.` todo). Sections come in a fixed order. None of that is enforced by anything on its own -- it is a convention held up by whoever is editing.

**The main reason this tool exists: an agent should not spend tokens on work a CPU does for free.**

Most of what happens to such a file is mechanical. Joining a statement that got hard-wrapped back onto one line. Checking that a rule citing `V2` really does declare `V2` somewhere. Keeping rows in id order. Confirming every task claims exactly one milestone. An agent doing that by hand reads the whole file, reasons about it, and writes it back -- tokens spent, latency added, and a judgement call invited where none was needed. It can also simply get it wrong. Every one of those operations is a pure function of the text, so `mth` does them on the CPU instead: no model, no network, deterministic, free.

**The second reason: one standard, enforced the same way every time.**

Before this crate the format had **two** hand-maintained implementations and no home. They disagreed with each other, both of their test suites stayed green, and 88 tasks belonged to no milestone at all before anyone noticed. A rule each tool interprets for itself is not a rule. One implementation, or the rule is decoration.

[`FORMAT.md`](FORMAT.md) is cavekit's, vendored here **verbatim** so a tool that cites the format can read it. What microlith *enforces* lives in its own `SPEC.md` under §V, and that set is a superset -- the proof that no write loses a word, the guarantee that formatting twice changes nothing, the line cap, citations that must resolve, rows that must stay ordered, and a check that a decision you once rejected has not quietly crept back in. None of those have a counterpart upstream. The format is an input, not the identity.

## Install

```sh
cargo install microlith
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
SPEC.md:22: microlith/V13: `V14` is cited but never declared
    why: a dangling reference reads as authoritative, so nobody follows it
    mechanical: point it at the rule that was meant
    judgment: declare V14, if the rule is real but missing
```

`file:line:` first, so an editor and `grep -n` both jump to it. The rule id is **qualified** and sits beside the message, never inside the coordinates: every consumer is a caveman spec numbering from `V1`, so a bare `V13` printed against *your* `SPEC.md` names one of your rules instead of one of microlith's.

A **mechanical** direction is deterministic and reversible, so an agent may apply it unattended. A **judgment** direction accepts a regression or changes intent -- an agent that applies one blindly has silenced the guardrail rather than fixed the defect. `--format json` emits the same anatomy with `kind` as data, so an agent never has to parse prose.

`--records <file>` adds the rejected-option check. Its baseline lives with the spec being checked rather than in microlith, because "this record still exists" is a claim about edits over time and no single file can answer it.

### `derive` and `anchors` -- report only

These answer questions rather than passing judgement, so they **exit 0 even with findings**. An invariant nothing cites might be a dead rule or a missing citation, and only a reader can say which; a gate would answer by fiat.

`derive` reports statement sizes, how often each invariant is cited, and which are cited by nothing. `anchors` lists the `§S.n` address of every item alongside the id it currently resolves to -- addresses are ordinal, so retiring an id shifts every address below it, and printing both makes that visible instead of surprising.

<!-- BEGIN mth docs -->
## Commands

Every verb, its synopsis and what it does. Regenerate with `mth docs`.

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
<!-- END mth docs -->

The command reference above is **generated**: `mth docs` prints it, and a test fails if this block drifts from the code. Edit the registry in `src/docs.rs`, never the block by hand.

## Use it as a library

The binary is a thin shell. Every rule is a pure function over `&str`, so a consumer calls the rule instead of porting it:

```rust
let violations = microlith::check_spec(&text, &records);
for v in &violations {
    println!("{}: {} ({})", v.rule, v.msg, v.line);
}
```

## Guarantees

- **Lossless, provably.** Normalizing all whitespace in the input and the output must yield identical strings -- asserted *before* any write. A formatter that can drop a fact is one nobody may run unattended.
- **Idempotent.** `fmt(fmt(x)) == fmt(x)`, tested, because a formatter that changes its mind cannot be a `--check` gate.
- **Deterministic.** No inference, no network, and **zero dependencies**. Every operation is a pure function of the text.
- **Self-guarding.** microlith's own `SPEC.md` is the first file it formats, caps and checks, using the binary being built. A rule this crate cannot pass is a rule it cannot ship.
- **Every guard is proven by a planted violation**, never by reading it, plus a companion proving it accepts every real shape -- so no check can pass by rejecting everything.

## Status

`0.4.0`, unpublished. A minor version here is a level of **guarantee**, not a feature count: `0.4` means the rules hold on real-world markdown rather than only on this repo's own file. `SPEC.md` §V.30 carries the ladder, and [`CHANGELOG.md`](CHANGELOG.md) renders it as a table you can read without opening the spec.

**An even minor is stable; an odd minor is functional but not for production** -- the Linux 2.x and GNOME convention, and the parity describes the *release*, not the work in it. So `0.5` is the first public rung and says so in the number: usable, worth depending on for a trial, deliberately **partial** -- what ships works, and not everything ships yet. `0.6` is where that surface settles and becomes production-ready.

All six commands are built and gate this repo's own spec.

`SPEC.md` records what remains, what was deliberately dropped, and every bug found so far with the rule that now catches it.

## The name

A [microlith](https://en.wikipedia.org/wiki/Microlith) is a small stone blade -- rarely more than a few centimetres -- knapped in the Mesolithic. Three things about it are the whole reason for the name.

**It was never used alone.** Microliths were hafted in rows into arrow shafts, spear tips and sickles, several to a tool. Each was one part of a job that no single piece could do by itself. `mth` is not the thing that develops your software; it is one small hard edge that other tools get assembled around. It does the mechanical operations on `SPEC.md` and nothing else -- no inference, no network, no judgement -- so an agent, a git hook, a CI job or a person can haft it into whatever they are actually building. The library exists for exactly that reason: a consumer calls the rule instead of carrying their own copy of it.

**It was basic, and powerful, and took almost no training.** That is the part worth stealing. Microlithic technology spread because a standard small blade did not need a master knapper -- ordinary people could make them, use them, and repair a tool by replacing one blade rather than remaking the whole thing. So: six verbs, no configuration file, no flags you must learn before the tool is useful, and silence when there is nothing to report. `mth fmt` and `mth check` are the entire learning curve. Power here means the guarantees underneath -- lossless, idempotent, deterministic -- not surface area you have to study.

**It was standardised.** Interchangeable by design, which is what made replacement cheap. Same bet here: every rule is a pure function over `&str` with one definition, so there is one implementation to agree with rather than several to reconcile.

The theme is honest, too. The notation is [caveman](https://github.com/JuliusBrussee/caveman), the toolkit is [cavekit](https://github.com/JuliusBrussee/cavekit), and a microlith is the thing you would actually be holding.

## Contributing

The spec changes first, then the code -- [`CONTRIBUTING.md`](CONTRIBUTING.md) explains the loop and how to get set up. [`AGENTS.md`](AGENTS.md) is the deeper working guide, for humans and agents alike.

## License

[MIT](LICENSE).
