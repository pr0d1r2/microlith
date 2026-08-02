# Working here

For agents and humans. Read this before changing anything; read `SPEC.md`
for what must hold and what to build next.

## Start here: the gate

`hk.pkl` defines every gate op once; `.github/workflows/ci.yml` supplies
only the runner and the toolchain (V22). Entering the dev shell — `direnv
allow`, or `nix develop` — installs the `pre-commit` and `pre-push` hooks
and rewrites them on every entry, so the hook you have is always the one
`flake.nix` describes.

**The hooks are the gate of record** (V22), and `ci.yml` calls that same
definition rather than restating it — so a laptop and a runner cannot
disagree about what passes. `all` is literally `fast` plus two steps.
Worth knowing while reading a green local run: CI has still executed zero
times, because the GitHub remote does not exist yet.

The hook is also where you — or the agent loop — are told what to fix, so
every step's failure text is written to be acted on rather than merely
read.

[`INTEGRATION.md`](INTEGRATION.md) has the whole flow: which steps run at
each stage, which **files** each stage examines, and why the cargo steps
are chained.

Outside that shell the hooks **skip loudly**: one line on stderr, exit 0
(V23). A gate exists to stop bad commits, not to stop git. But nothing
stands behind them, so that line means genuinely ungated rather than
merely local — enter the shell (B19).

The next unstarted task is **T8**, publish, in `SPEC.md`. T7 is `~` and
blocked on it.

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

The hook runner is a convenience, never a dependency of the crate: no
verdict here rests on `hk`'s own logic.

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
