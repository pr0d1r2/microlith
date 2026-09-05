# Built by an LLM, deliberately and in the open

This repository — code, spec, tests and prose — was written by
[Claude Code](https://claude.com/claude-code) running Anthropic's **Claude
Opus 5**. Most commits carry a `Co-Authored-By: Claude Opus 5` trailer, and
every commit is signed. A human owns every decision, reviews every diff, and is
accountable for what ships.

That is the disclaimer. The rest of this file is why it is stated as a design
note rather than as an apology, and what a reader can check for themselves.

## Why say it at all

Two reasons, and only the first is the obvious one.

A model writes plausible code, and plausible is not correct. A reader who does
not know how a repository was produced cannot calibrate how hard to look at it.
Saying so is the minimum.

The second reason is specific to a tool like this one. `mth` exists to make a
claim about a document mechanical — that its sections are ordered, its ids
unique, its citations resolvable — so that "the spec says so" becomes a
statement about a file that was parsed rather than one that was skimmed. A
repository built by a model, arguing that claims should be checked rather than
asserted, has to hold itself to that first. Everything below is an attempt to
make the provenance checkable instead of merely disclosed.

## The method is spec-driven development

[`SPEC.md`](../SPEC.md) is the law rather than a description written afterwards:
it holds the invariants that must stay true, the tasks that remain, and a record
of every bug found so far paired with the rule that now catches it.

A rule and its checker land in the *same commit*, because a rule with no runner
gates nothing. Every guard is proven by planting the violation it exists to
catch, plus a companion proving it still accepts every real shape — a guard
tested only against the bad case is a guard nobody has shown to be selective.

## The guardrails are git hooks that also run on CI

Entering the dev shell (`nix develop`, or `direnv allow`) installs `pre-commit`
and `pre-push`, which run [hk](https://github.com/jdx/hk) against one definition
of the gate in [`hk.pkl`](../hk.pkl).

Pre-commit takes the fast set — format, clippy, tests, and the tool checking its
own spec. Pre-push adds what is too costly for every commit: doctests, rustdoc,
`cargo llvm-cov` against a line-coverage floor that ratchets rather than sits,
and the two network steps — `cargo deny` for advisories and `cargo
semver-checks` against the last release tag.

The floor and the measured percentage are badges in the README, generated from
the run itself rather than typed there, because a number typed into prose is
true the day it is written and quietly wrong after.

[`ci.yml`](../.github/workflows/ci.yml) calls that same definition, so a laptop
and a runner cannot disagree. [`INTEGRATION.md`](INTEGRATION.md) has the full
flow.

## What a reader should actually check

In the order it matters:

1. **Does the gate run for you?** `nix develop` then `hk check`. If a claim on
   this page is false, that is where it shows.
2. **Does the bug record look real or curated?** Every entry pairs a defect
   with the rule that now catches it. A log with no unflattering entries is a
   log somebody edited.
3. **Do the invariants have runners?** An invariant nothing executes is exactly
   the failure this tool exists to detect, and would be embarrassing here in
   particular.
4. **Is the tool run against its own spec?** It is a gate step. A tool that
   checks other people's documents and not its own has not been run in anger.

## Accountability

The human named in [`LICENSE`](../LICENSE) is responsible for this code,
including the parts a model wrote and the parts nobody caught. "The LLM wrote
it" is an explanation of provenance, never a transfer of responsibility.

Bug reports are welcome and unflattering ones are more useful — see
[`SECURITY.md`](SECURITY.md) for the ones that should not be public, and
[`CONTRIBUTING.md`](CONTRIBUTING.md) for everything else.

## Deeper

[`AGENTS.md`](../AGENTS.md) is the working guide ·
[`CONTRIBUTING.md`](CONTRIBUTING.md) is the loop ·
[`SPEC.md`](../SPEC.md) is what must hold and what remains ·
[`FORMAT.md`](../FORMAT.md) is the format itself ·
[`FORMAT-EXTENSIONS.md`](../FORMAT-EXTENSIONS.md) is what this copy adds to it.
