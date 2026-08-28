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
| `0.5` | odd | public and usable, deliberately **partial** — a consumer may depend on it and delete their ported copy | published |
| `0.6` | even | that surface stabilized — the fixes the first real users find, and a backlog a consumer can read without re-porting the grammar | **current**, published |
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

The format grows a published delta. `FORMAT.md` is cavekit's and ships
verbatim; what this build enforces on top of it now has a document of its own
rather than living in `SPEC.md` prose and three constants.

These are **additions** to the public surface, so the next release carrying
them is a minor rather than a patch. Every one of them is optional: a spec that
uses none is checked exactly as it was before.

### Added

- **`§F FEDERATION` and `§N NAV` are known section letters**, ranked between
  `§G` and `§C`. Together they let one spec span a directory tree — `§F`
  declares the edges a directory owns, `§N` the derived navigation to its
  neighbours — instead of one spec sitting alone at a project root. Absence
  stays legal, so a spec carrying neither is untouched.
- **`[superseded by V<n>]`**, a marker that retires an invariant without
  deleting it. Deleting one would free its id for reuse and strand every
  citation that still names it, so the statement stays and the mark says it is
  no longer in force. More than one replacement may be named, because a rule
  that is split is replaced by several. `check` reports a mark that names the
  rule itself, or one that points at a rule which is also retired.
- **`mth extensions`**, and `microlith::format_extensions()` beside it. Prints
  the sections and markers this build adds to the vendored format, as markdown
  — the source for `FORMAT-EXTENSIONS.md`, kept in sync by a test. Report-only,
  like `mth docs`.
- **`FORMAT-EXTENSIONS.md` ships in the `.crate`.** It is written for a reader
  who does *not* have this tool: copy it next to your own `FORMAT.md` and the
  extensions are adoptable by hand. Every entry carries what was measured
  before it was claimed — how many specs use it, and what claiming it costs —
  against a stated denominator, because a letter is only free until somebody
  else spends it.

### Changed

- **`derive` no longer calls a retired invariant an orphan.** That report asks
  whether a rule is dead or merely uncited, and a supersession mark is somebody
  answering it.
- **`--help` wraps its verb list.** It never did, and the eighth verb pushed
  the line to 83 columns — past the width every other line in that output is
  held to.

### Fixed

- **`mth anchors` could not address `§F` or `§N` items at all.** It decided
  what was addressable from a hand-listed set of letters, so the two new
  sections were known everywhere else and invisible there. It matters more
  than its size suggests: neither section carries ids, so the ordinal is the
  only way to cite an edge — `§F.2` is the whole point of the letters, and
  the verb that produces that address did not know they existed. The set is
  derived now, so the next letter is addressable the day it is known.

- **`cites` could silently hold prose instead of ids.** `id|status|text|cites`
  is positional, so an unescaped `|` in the text does not break a row — it
  moves the field boundary, and the last field stops being citations. Six of
  *this* repo's own rows shipped a sentence there, and `check` was green on
  all of them, because every other rule reads the text rather than the
  fields. If you consume `mth tasks --format json`, its `cites` are now
  trustworthy in a way they were not before.

### This one will fire on specs that pass today

**`V42` is a new check, and it is not silent on existing files.** It reports
a row whose literal `|` is unescaped, which FORMAT.md has always required
(*"literal `|` → escape as `\|`"*) and which nothing enforced until now.

Measured across 256 distinct specs before it was written: **32 rows in 11
specs, and 7 specs that pass `check` today will not after upgrading.** That
is 1.4% of rows — low enough that we judged it worth printing, but it is not
zero and you may meet it.

Each hit is a real defect rather than a style preference: the row's last
field is not what its author wrote, and any consumer reading that field has
been believing a fragment of prose. The fix is to escape the pipe, or to
reword so the row carries none — we took the second route in our own spec,
because `\|` inside a code span renders its backslash and made the document
worse to read. That trade-off is being raised with the format's author.

## [0.6.1] — 2026-08-14

A patch, so it sits off the version ladder above: `0.6`'s answer to *what can
you rely on at this tag?* is unchanged. The three contracts — the CLI surface,
the JSON output and the library API — are byte-for-byte what `0.6.0` shipped.

The published crate is **not** identical, though, and it would be wrong to say
so: `rust-version` lives in the manifest that ships inside the `.crate`, so
lowering it to **1.95** changes what a consumer resolves. It changes it in the
one direction that cannot break them — a lower floor admits more toolchains
than before, never fewer — which is why it is a patch and not a rung.

### Changed

- **MSRV lowered, 1.96 → 1.95.** Nothing here ever needed 1.96 — the highest
  feature `src/` uses is `Option::is_none_or` (1.82). The old floor was a
  measurement of whatever the pinned nixpkgs revision happened to carry, and
  the flake now follows the fleet's one nixpkgs authority (`nixos-26.05`,
  rustc 1.95.0) instead of naming a revision of its own. Lowering a floor only
  widens who can build the crate, so this needs nothing from consumers.
- The dev shell takes `hk` from [nix-hk](https://github.com/pr0d1r2/nix-hk)
  rather than from nixpkgs, which packages no `hk` at all on 26.05. Affects
  contributors, not consumers: the published crate is unchanged by it.

### Fixed

- The gate's `semver` step reported *"the pub API broke against v0.6.0"* when
  `cargo-semver-checks` had in fact failed to **build** the baseline — a
  verdict about code the run never compared, pointing the reader at an API
  that had not changed. It now reads the baseline tag's declared floor, and
  when this toolchain sits below it says so and exits 0, re-arming at the
  first tag it can build. Contributor-facing (§V.36, §B.23).
- The `nixpkgs` README badge was a number typed into the generator that
  exists to stop numbers being typed. It said `26.11` from birth and is now
  read from `flake.lock`.

### Added

- Releases run through [`cargo-release`](https://github.com/crate-ci/cargo-release),
  configured by `release.toml` — the gate is a pre-release hook, so a red gate
  aborts a publish rather than being something a releaser is trusted to have
  run first. Contributor-facing; nothing about the crate changes.
- The `package` gate step now also records what the `.crate` ships, in
  `.crate-files`. A file that starts or stops shipping shows up as a diff line
  in the pull request that caused it, instead of at publish time.

## [0.6.0] — 2026-08-08

**Even minor: stable** (§V.34). `0.5` was published as deliberately partial; this
is where that surface settles. The rung's question — *what can you rely on at
this tag?* — is answered: the backlog is readable by a consumer without
re-porting the grammar, and the first defect a real user found is fixed.

The fix below is the reason this is not merely an addition. A literal `|` in a
table cell has been misread since the first checker, silently, and any consumer
reading task text or citations through `0.5.0` was reading truncated cells.

### Added

- `mth tasks` — enumerates `§T`: every row's id, status, text and citations, in
  id order, with `--format json` for a caller that parses it. It **enumerates
  and does not select**: which rows are pending is mechanical, which one to work
  next is judgement and stays with the caller (§V.6).
- `microlith::tasks_report`, `tasks_report_verbose` and `tasks_json` — the same
  enumeration for a caller that links the library instead of running the binary.

### Fixed

- a literal `|` in a table cell, which FORMAT.md says to write `\|`, was read as
  a field boundary. Every cell after it shifted by one, so a task's text was
  truncated and its citations were looked for in the wrong column. Three rows of
  this repo's own `SPEC.md` are written that way. One splitter now serves the
  status rule, the milestone rule and the new verb.

## [0.5.0] — 2026-08-02

Promoted from `0.5.0-rc.1` unchanged — no code differs between them. The
candidate did its job and found nothing, which is the outcome it was published
to establish rather than a formality skipped:

- the published tarball was **downloaded from crates.io** and its own test suite
  run against it — 185 tests, 0 failures. The dogfood tests read `SPEC.md` and
  `.spec-records`, so this proves the shipped `.crate` carries what it needs,
  not merely that the repo does.
- `cargo install microlith --version 0.5.0-rc.1 --locked` produced a working
  `mth` binary from the registry.
- docs.rs built it clean. That build runs in a different, network-less sandbox
  and is the most common reason a first publish needs a second attempt.

`0.5` is **odd**: public and usable, deliberately **partial** (§V.34). Depend on
it for a trial. `0.6` is where the surface settles, and it carries the fixes the
first real users find — which is why it is the next rung rather than a
maintenance afterthought.

### Known limits

- The public API was trimmed to the verbs (§V.32) and has **not yet been
  exercised by an outside consumer**. Adding a `pub` item is a non-breaking
  minor change, so a gap here costs a version rather than a redesign.
- `ci.yml` has run green on every commit since the repo went public, but a green
  **streak** is a `0.7` condition and this is not one yet.

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

[Unreleased]: https://github.com/pr0d1r2/microlith/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/pr0d1r2/microlith/releases/tag/v0.6.1
[0.6.0]: https://github.com/pr0d1r2/microlith/releases/tag/v0.6.0
[0.5.0]: https://github.com/pr0d1r2/microlith/releases/tag/v0.5.0
[0.5.0-rc.1]: https://github.com/pr0d1r2/microlith/releases/tag/v0.5.0-rc.1
