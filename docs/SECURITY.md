# Security policy

## Reporting a vulnerability

Report privately, not in a public issue.

- Preferred: [GitHub private vulnerability
  reporting](https://github.com/pr0d1r2/microlith/security/advisories/new)
- Or email **pr0d1r2@gmail.com** with `microlith security` in the subject.

Please include what you ran, what happened, and the input that triggered it. A
reproducing `SPEC.md` is worth more than a description of one.

Expect an acknowledgement within a week. If a report is valid, the fix and the
advisory go out together, and you are credited unless you ask otherwise.

## Supported versions

Pre-1.0, only the latest published version is supported. There are no backports
to earlier minors — see [`CHANGELOG.md`](../CHANGELOG.md) for what each rung means.

## What the attack surface actually is

Worth stating plainly, because it is unusually small and that changes what is
worth reporting:

- **Zero runtime dependencies.** No transitive supply chain to compromise.
- **No network.** Nothing is fetched, resolved or phoned home at any point.
- **No `unsafe`.** `unsafe_code = "forbid"` in `Cargo.toml`, so it cannot be
  reintroduced locally — memory-safety bugs are not representable here.
- **No code execution.** `mth` reads a markdown file and writes a markdown
  file. It does not evaluate, shell out, or load plugins.

So the realistic classes are:

1. **A write that loses or corrupts content.** `fmt` and `migrate` rewrite your
   spec in place. They assert losslessness before writing, but a proof with a
   gap is a real vulnerability in the only sense that matters here — your file.
2. **A crash or hang on hostile input.** Panics are denied by lint, but a
   pathological input reaching an unwrap-free path and still aborting, or an
   input causing unbounded time or memory, is worth reporting.
3. **A check that silently passes.** A gate that reports clean on a file it
   could not parse is the failure mode this project was built to prevent, and
   is treated as a defect of the same seriousness.

If you found something in category 1, say so first — those get priority.

## What is out of scope

- Findings against `FORMAT.md`. It is [vendored from
  cavekit](THIRD-PARTY-NOTICES.md) verbatim; report those upstream.
- The absence of a feature, or a rule you disagree with. That is
  [`SPEC.md`](../SPEC.md) and an ordinary issue.
