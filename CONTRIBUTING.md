# Contributing

Thanks for looking. This project has an unusual working loop, and knowing it up
front will save you a rejected patch.

**The short version: the spec changes first, then the code.** `SPEC.md` is not
documentation of what was built — it is the statement of what must hold, and the
code is downstream of it.

## Get set up

Everything is pinned in a Nix flake, so the toolchain you get is the toolchain
the gate gets:

```sh
nix develop          # or: direnv allow
```

Entering the shell installs the `pre-commit` and `pre-push` hooks and rewrites
them on every entry. **The hooks are the gate of record.** Outside the shell
they skip loudly — one line on stderr, exit 0 — which means genuinely ungated,
not merely local. Enter the shell.

You do not need the hook runner to check your own work. Every gate step is a
plain cargo command:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- fmt --check SPEC.md     # the dogfood: we format our own law
cargo run -- check --records .spec-records SPEC.md
```

No verdict in this repo rests on `hk`'s own logic.

## The loop

1. **Read `SPEC.md`.** §V is what must hold; §T is what is left to build; §B is
   every bug found so far with the rule that now catches it. If what you want to
   change is a rule, it is a spec change and belongs in its own commit.
2. **Write the rule and its runner together.** A rule with no checker reads as
   law and gates nothing. They land in the same commit.
3. **Prove the guard by planting the violation it exists to catch** — plus a
   companion proving it accepts every real shape. A guard that has never
   rejected anything is indistinguishable from one that cannot.
4. **Put the rule in the library**, as a pure function over `&str`, so a consumer
   can call it rather than re-port it. That is the entire reason this crate
   exists: the format previously had two hand-maintained implementations that
   disagreed while both of their test suites stayed green.

## The one hard rule

**Never bypass.** `--no-verify`, lowering a threshold, deleting a test, or adding
`#[allow]` to silence clippy are all ways of shipping the defect with the alarm
switched off. Fix the cause.

If you think the check itself is wrong, that is a legitimate position — but it is
a spec change. Say so in `SPEC.md`, in its own commit, with the reasoning.

## Things that will get a patch turned down

- **Inference.** Every operation here is CPU-derivable and deterministic. A
  rewrite that needs judgement belongs to a human or an agent calling `/spec`,
  not hidden inside a formatter.
- **Dependencies.** Zero today. Keep it that way unless a rule genuinely cannot
  be computed locally.
- **A second definition of something that already exists.** The first question
  about any new rule is *where does it live* — here, once.
- **Raising a ceiling to turn a gate green.** `.context-limits` caps `SPEC.md`
  and `format.rs` caps a line; both are set from measurement with real slack, and
  `tests/dogfood.rs` fails if the slack erodes. Raising either is a reviewed
  decision with its reason in the commit, not a reflex.

## Commits

Explain **why**, not what — the diff already says what. Where a change was
derived from a measurement, give the number; this repo's history is full of
"MEASURED: 2 of 51 passed" for a reason. Keep separate topics in separate
commits, and keep a spec change apart from the code that follows it.

## Reporting a bug

A `§B` row in `SPEC.md` records the cause, the fix, and the rule that now catches
it. If you are reporting rather than fixing, an issue with a reproducing input is
plenty — the spec row is our job.

## Going deeper

[`AGENTS.md`](AGENTS.md) is the full working guide, for humans and agents alike:
the gate's structure, how to add a check, the size ceiling, and what deliberately
does not belong in this crate. [`FORMAT.md`](FORMAT.md) is the cavekit format
this tool reads, vendored verbatim.

## License

By contributing you agree that your work is licensed under the
[MIT License](LICENSE), the same terms covering the rest of the project.
