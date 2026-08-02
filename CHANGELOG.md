# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Version ladder

A minor version here is a level of **guarantee**, not a feature count. Each rung
answers one question: *what can you rely on at this tag?* `SPEC.md` §V.30 is the
source of this ladder; the table below is its public rendering, so a consumer
arriving from crates.io can read the number without opening our spec.

**An even minor is stable; an odd minor is functional but not for production**
(§V.34) — the Linux 2.x and GNOME convention. The parity describes the
*release*, not the work that went into it.

| version | parity | what you can rely on | status |
|---------|--------|----------------------|--------|
| `0.1` | odd | it builds reproducibly anywhere | reached |
| `0.2` | even | it cannot regress locally — the gate runs in git hooks | reached |
| `0.3` | odd | one implementation of the format rules, not two that disagree | reached |
| `0.4` | even | those rules hold on real-world markdown, not only on this repo's own file | reached |
| `0.5` | odd | public and usable, deliberately **partial** — a consumer may depend on it and delete their ported copy | **current** |
| `0.6` | even | that surface stabilized — the fixes the first real users find | planned |
| `0.7` | odd | the public gate is trustworthy: platform matrix, MSRV axis, release automation | planned |
| `0.8` | even | mechanical editing and planning — every mutation through one verified write path | planned |
| `1.0` | even | the contract frozen: the CLI surface, the JSON output, and the library API | planned |

`0.1` through `0.4` were never published to crates.io. They are recorded because
the guarantee was earned, not because there is an artifact to download.

Pre-1.0 SemVer permits a minor to break, and here each rung *is* a behaviour
change, so that permission is used honestly rather than worked around. crates.io
is immutable — yanking hides a version, it does not delete it — so the first
public artifact is `0.5.0-rc.1`, which cargo does not select by default. The
pipeline gets proven before a permanent number is spent.

## [Unreleased]

Nothing yet.

## [0.5.0-rc.1] — 2026-08-02

The first public artifact. A **release candidate** on purpose: cargo does not
select a pre-release by default, so the publish pipeline is proven before a
permanent number is spent on an immutable registry. `0.5.0` follows once this
one installs from crates.io and the consumer builds against it.

`0.5` is **odd**, and the number is saying so: public and usable, deliberately
partial. Depend on it for a trial; `0.6` is where the surface settles.

### Added

- **`fmt`** — one line per statement: joins hard wraps, enforces the line cap.
  The transform is proven whitespace-only before any write.
- **`check`** — the structural rules: sections present and ordered, ids unique,
  citations resolve, rows sorted, every task in exactly one milestone, every
  status one of `.` `~` `x`. Each violation carries a line and a ranked fix,
  marked *mechanical* (safe to apply unattended) or *judgment* (needs a human).
  `--format json` emits the same anatomy as data.
- **`migrate`** — section headers to canonical cavekit 4.1.0. Every alphanumeric
  run of the original is proven to survive before any write; a letter used for a
  different concept is reported rather than rewritten.
- **`derive`** — statement sizes, the citation graph, invariants cited by
  nothing, and statements said twice. Report-only, exits 0 even with findings.
- **`anchors`** — the `§S.n` address of every item beside the id it resolves to,
  and whether the two have drifted. Report-only.
- **`docs`** — prints the command reference as markdown; a test freezes it
  against README so the documentation cannot go stale silently.
- **Library API** — every rule is a pure function over `&str`, so a consumer
  calls the rule instead of porting it. `microlith::check_spec` and
  `microlith::format_spec` are the entry points.

### Guarantees

- **Lossless, provably.** Normalizing all whitespace in input and output must
  yield identical strings, asserted *before* any write.
- **Idempotent.** `fmt(fmt(x)) == fmt(x)`, tested.
- **Deterministic.** No inference, no network, and **zero dependencies**.
- **Self-guarding.** This repo's own `SPEC.md` is the first file it formats,
  caps and checks, using the binary being built.
- **Every guard is proven by a planted violation**, with a companion proving it
  accepts every real shape — so no check can pass by rejecting everything.

[Unreleased]: https://github.com/pr0d1r2/microlith/compare/v0.5.0-rc.1...HEAD
[0.5.0-rc.1]: https://github.com/pr0d1r2/microlith/releases/tag/v0.5.0-rc.1
