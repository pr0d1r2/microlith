# Working here

For agents and humans. Read this before changing anything; read `SPEC.md`
for what must hold and what to build next.

## Start here: what is half-done

`hk.pkl` and `.github/workflows/ci.yml` are **annotated drafts that have
never run**. They are on disk instead of absent so the next session starts
from a draft rather than a blank page, and they are **inert by
construction**:

- no git hooks are installed, so `hk` is never invoked on a commit
- the workflow triggers on `workflow_dispatch` only, so it cannot fire

Each carries a banner naming exactly what remains. That is the open half of
**T6** in `SPEC.md`, and it is the next thing to build. Expect
`hk check --all` to fail the first time it is run: fix the causes, and set
the coverage floor from a fresh measurement rather than from the placeholder
number in the file.

Nothing else in the repo is half-done. `fmt` works, is tested, and gates
this repo's own spec.

## The rule

**Never bypass.** `--no-verify`, lowering a threshold, deleting a test, or
adding `#[allow]` to silence clippy are all ways of shipping the defect
with the alarm switched off. Fix the cause. If the check itself is wrong,
that is a spec change: say so in `SPEC.md`, in its own commit.

## Reproduce a verdict

Everything is a plain cargo command. No runner is required to check your
work:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -- fmt --check SPEC.md     # the dogfood: we format our own law
```

## What this crate is

One implementation of the cavekit `SPEC.md` format. It exists because that
format had two hand-maintained implementations that disagreed while both
gates stayed green. So the first question about any new rule is **where
does it live** — here, once, called by consumers (V7).

`FORMAT.md` in this repo is normative. If a rule is in your head and not in
`FORMAT.md` or `SPEC.md`, it is not a rule yet.

## Adding a check

Three things, or it is not done:

1. The rule as a **pure function over `&str`** in the lib, so a consumer
   can call it instead of re-porting it (V7).
2. A **planted violation** proving it rejects, plus a companion proving it
   accepts every real shape (V18). A guard that never rejected anything is
   indistinguishable from one that cannot.
3. The rule and its checker in the **same commit** (V17). A rule with no
   runner reads as law and gates nothing; a checker that lands on a file
   violating it is red at birth.

## The size ceiling

`.context-limits` caps `SPEC.md`. It is set with real slack and it is not a
formality: a spec is re-read on every session that touches this repo, so
every byte is a recurring cost. Raising the ceiling is a reviewed decision
with its reason in the commit — not a reflex when the gate turns red.

Same for the line cap in `format.rs`. It is set from the measured maximum,
and `tests/dogfood.rs` fails if the slack erodes.

## What does not belong here

- **Inference.** Every operation is CPU-derivable and deterministic (V6).
  A rewrite needing judgement is a `/spec` call by a human or an agent, not
  a hidden model call inside a formatter.
- **Network.** Zero dependencies today. Keep it that way unless a rule
  genuinely cannot be computed locally.
- **Method.** How a change was derived belongs in its commit message; the
  spec says what must hold.

## Cross-project references

Both this repo and `itok`, its first consumer, are public. Naming the other
is fine — but always **qualified** (`itok's V82`, never a bare `V82`, which
reads as ours) and always **see-also**: every rule here carries its own
evidence, so nobody has to fetch another repo to understand it (V19).
