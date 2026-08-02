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
- **Raising a ceiling to turn a gate green.** `format.rs` caps a line, set from
  the measured maximum, and `tests/dogfood.rs` fails if that slack erodes.
  `.context-limits` declares a size ceiling for `SPEC.md` on top of that.
  Raising either is a reviewed decision with its reason in the commit, not a
  reflex — the spec is re-read on every session that touches this repo, so every
  byte is a recurring cost.

## Commits

Explain **why**, not what — the diff already says what. Where a change was
derived from a measurement, give the number; this repo's history is full of
"MEASURED: 2 of 51 passed" for a reason. Keep separate topics in separate
commits, and keep a spec change apart from the code that follows it.

## Submitting a change

Changes reach `main` through **pull requests** — nothing is pushed to `main`
directly.

1. Branch off `main`. Fork first if you do not have push access.
2. Make the change, following the loop above. Keep a spec change in its own
   commit, separate from the code that follows it.
3. Push. The `pre-push` hook runs the full gate, so a red PR should be rare —
   and if the hook skipped loudly because you were outside the dev shell, that
   is exactly when it will not be.
4. Open the PR. CI runs the same 21 steps your hook just ran, plus
   `nix build .#default`.
5. One topic per PR. Two unrelated fixes are two pull requests.

CI is not a second opinion about correctness — it runs the identical definition
from [`hk.pkl`](../hk.pkl). What review adds is a reader. The gate catches what is
mechanically wrong; a person catches what is merely a bad idea.

## Reporting a bug

A `§B` row in `SPEC.md` records the cause, the fix, and the rule that now catches
it. If you are reporting rather than fixing, an issue with a reproducing input is
plenty — the spec row is our job.

## Going deeper

[`INTEGRATION.md`](INTEGRATION.md) is the guardrails in full: what runs at
commit, at push and on CI, why the steps are chained the way they are, and how to
reproduce any verdict with plain cargo. [`AGENTS.md`](../AGENTS.md) is the working
guide — how to add a check, the size ceiling, and what deliberately does not
belong in this crate. [`FORMAT.md`](../FORMAT.md) is the cavekit format this tool
reads, vendored verbatim.

## License

By contributing you agree that your work is licensed under the
[MIT License](../LICENSE), the same terms covering the rest of the project.
