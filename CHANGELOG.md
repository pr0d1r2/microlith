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
| `0.6` | even | that surface stabilized — the fixes the first real users find, and a backlog a consumer can read without re-porting the grammar | published |
| `0.7` | odd | one spec spans a directory tree — `§F` declares the edges a directory owns, `§N` the derived navigation, and the row codec every consumer would otherwise re-implement is exported | **current**, published |
| `0.8` | even | the public gate is trustworthy: platform matrix, MSRV axis, release automation | planned |
| `0.9` | odd | mechanical editing and planning — every mutation through one verified write path | planned |
| `1.0` | even | the contract frozen: the CLI surface, the JSON output, and the library API | planned |

`0.1` through `0.4` were never published to crates.io. They are recorded because
the guarantee was earned, not because there is an artifact to download.

Pre-1.0 SemVer permits a minor to break, and here each rung *is* a behaviour
change, so that permission is used honestly rather than worked around. crates.io
is immutable — yanking hides a version, it does not delete it — so the first
public artifact is `0.5.0-rc.1`, which cargo does not select by default. The
pipeline gets proven before a permanent number is spent.

## [Unreleased]

### Added

- **`mth tasks --format json` names each milestone's ship version** (V53,
  issue #46). A new `milestones` array sits beside `tasks`, one object per
  `| M<n> |` row in file order:

      {"id":"M9","ships":"0.8.0","tasks":["T26","T26a",...],"pending":6}

  Milestone numbers are not release order — in this repo M13 shipped before
  M9 — so a consumer asking "which rows belong to the next release?" had to
  re-read the phrase *ships as `0.8.0`* out of the scope cell itself. Now it
  reads `ships`. The version is the first *ships as* followed by a backticked
  value, carried verbatim, and `null` when the scope names none; ordering
  releases stays the caller's. `tasks` is read off each row's `milestone`
  field, so the two cannot disagree, and `pending` counts every row not yet
  `x`. The array is empty, never absent, when a spec declares no milestones.
  The library exports the same reading as `ships(text)`.

  Additive: a consumer that ignores the key is unaffected (V32).

## [0.7.3] — 2026-09-20

A patch, and unlike the last one it **meets the test `0.7.1` set** — *"a
consumer gets it by `cargo update` and loses nothing"*. Measured rather than
assumed, because `0.7.2` did not meet that test and had to say so.

MEASURED two ways, both with two builds, because one program cannot be two
(B42). Over **869 `SPEC.md` files on disk**, the `0.7.2` binary and this one
were run side by side on `check`, `derive`, `anchors` and `tasks --format
json`: **byte-identical output and identical exit codes on every file, for
every verb**. The corpus sweep, run at both tags over its own deduplicated
denominator, agrees — 381 specs in 73 projects, `clean: 150`, 522 violations,
the same per-rule tallies on both sides.

The one observable difference is the one that was added: `mth bugs SPEC.md`
exits 2 on `0.7.2` (unknown command) and 0 here. That asymmetry is the whole
reason it is a verb rather than a flag.

### Added

- **`mth bugs` enumerates `§B`** — `id`, `date`, `cause` and `fix` for every
  bug record, in id order, in `human` or `json`, with `unread` beside the
  array exactly as `tasks` carries it (T42, issue #41). The rows were already
  parsed: `check` validates their ids and citations and `derive` counts
  them, so nothing new is read here — what was missing was a way for a
  caller to ask, which left a consumer acting on bug records keeping its own
  parser of a grammar this crate owns.

  `§B` is not `§T` with other words, and a caller reusing that shape gets a
  wrong answer: FORMAT.md gives `§B` `id|date|cause|fix` with **no status
  column**, so the second cell is a date, and the fix cell is carried as
  written rather than split into citations it may not be — the corpus holds
  prose there as often as a rule id.

  A verb rather than `tasks --section B`: every verb here ignores a flag it
  does not know, so an older published `mth` answers `--all-sections` with
  `§T` rows and exit 0. An unknown verb exits 2, so a caller's fallback
  fires on absence. That is T29's argument against `derive --format json`,
  on a worse case — here the wrong answer is shaped exactly like the right
  one.

  Internally the four-field pipe row now has **one** reader (`rows`), shared
  by both verbs, so `§R` will need no third parse when it lands.

### Fixed

- **Corrects two figures published from an unrepeatable measurement**
  (B42). `0.7.2`'s entry below says 76 of 419 distinct spec texts, across 18
  projects, go from a clean `check` to a reported violation. Measured with
  the corpus sweep at both tags — which is the method this repo keeps
  tested, for exactly this reason — the figure is **74 of 379**, `clean`
  falling from 223 to 149. The project count has no backing from that
  method at all and is withdrawn.

  The published script deduplicated specs by content only; the sweep also
  drops copies by name, so roughly forty texts it excludes were counted.
  That is B31's defect — two denominators for one population — and B27,
  B28, B31 and B33 are the same shape four times before this one.

  The 0.7.2 entry is left as written. crates.io is immutable, so the text a
  consumer already has cannot be edited; correcting it here is the only
  honest place, and editing history to hide a wrong number would be worse
  than the number.

### Added

- **The sweep counts empty-id furniture rows** and documents the two-build
  comparison it cannot make. #27 was answered with a count of rows whose id
  cell is empty — group labels between tasks — produced by a script that no
  longer exists. That count now lives in `examples/corpus.rs` with its own
  tests: **67 rows in 13 specs** of 379, correcting the 71-in-14 that was
  posted. A question of the form "how many specs change verdict between two
  releases" needs two builds and one program cannot be two, so the module
  documents running the sweep at each tag and diffing `clean:`.

## [0.7.2] — 2026-09-20

A patch by number. **Read this before updating: `check` now fails on specs it
used to pass.**

MEASURED against `0.7.1` over 419 distinct spec texts: **76 of them, across 18
distinct projects, go from a clean `check` to a reported violation.** None goes
the other way. `0.7.1`'s own entry set the test a patch has to meet — *"a
consumer gets it by `cargo update` and loses nothing"* — and this release does
not meet it. The number stays `0.7.2` as a deliberate call; the figure is
recorded here so nobody meets it as a surprise in CI.

The cause is V49, and the change is the point rather than an accident: a `§T`
written as a markdown table is invisible to the id grammar, so the section read
as ABSENT rather than unreadable. B15 recorded 16 specs passing `check` with
their whole task section unseen. Those specs now say so, once per section, and
`mth migrate <path>` converts them — the mechanical fix the finding names.

Three smaller changes to behaviour that already existed:

| invocation | `0.7.1` | `0.7.2` |
|---|---|---|
| `check`/`fmt --check` with several paths | exit 0 or 1 | exit 2 |
| `<verb> --help` | ran the verb | prints help |
| `tasks --format json` | 4 fields per row | adds `unread`, `milestone`, `declares_milestones` |

`fmt --help` inside a project rewrote `SPEC.md` before this release, and
`archive --help` moved a row into the sink; both now explain themselves.

The LIBRARY surface is additive only — `archive_spec`, `archive_report` and
`ARCHIVE` join it, and nothing existing changed shape — so a consumer
compiling against the crate is unaffected.

### Fixed

- **A second path is now REFUSED, not silently dropped** (V46, B37). `mth
  check` and `mth fmt --check` took the first positional path and ignored the
  rest in silence, so `mth check clean.md broken.md` exited 0 with `broken.md`
  never read, and a gate written `mth check *.md` reported green over a set it
  never examined. All six reading verbs shared the defect, since all six
  resolved their path the same way. Extra paths are now a usage error (exit 2)
  naming each one; run `mth` once per path.

  **This changes an exit code a caller may depend on**: a command that passed
  several paths used to exit 0 or 1 on the first of them and now exits 2. That
  verdict was never about the files it was given, so a caller reading it was
  already being misinformed, but the number itself is new.

  Fanning out over every path was rejected rather than deferred: `check` and
  `tasks` each emit a JSON document naming one path, so `--format json` would
  owe a multi-file shape nothing upstream defines. It reopens if FORMAT.md
  settles one, or if a caller turns up that cannot loop.

- **`mth <verb> --help` answers instead of running the verb** (V51, B41).
  Every verb read `--help` as its positional path. Outside a project that
  meant a complaint about the directory when the reader had asked about the
  flag. Inside one it is worse by a category:

  ```
  $ mth fmt --help
  mth: fmt rewrote SPEC.md
  $ mth archive --help
  mth: archive moved 1 row from SPEC.md to SPEC-ARCHIVE.md
  ```

  A write nobody asked for, reached by the one argument that means *not
  yet*. V10 says a tool that rewrites law is one a user must invoke
  deliberately, and typing `--help` is the plainest statement there is of
  not having decided to.

  Two features composed into it: `positional()` skips a flag's value but
  never asked whether the flag takes one, so `--help` was skipped as a flag
  and the path fell through to `SPEC.md`'s default.

  Now every verb answers, rendered from the one command registry (V33), so
  a new verb gets help the day it is listed and no second copy can go stale:

  ```
  $ mth check --help
  usage:
    mth check [--records <file>] [--format human|json] [--verbose] [<path>]

  The structural rules: sections present and ordered, ids unique, citations
  resolve, rows sorted, ...

  exit: 0 ok | 1 drift or violation | 2 usage
  ```

  `-h` works the same, and the flag is read anywhere in the arguments —
  `mth check --records r.txt --help` is still a question. An unknown verb is
  still reported as one, which is the answer a caller needs before any help.

- **A dangling citation in a federated node now names the nodes** (V50, B40).
  A child's invariant cited from its parent read as dangling, and both
  judgments pointed the wrong way: "point it at the rule that was meant"
  already did, and "declare V10" would duplicate an id the child owns, which
  is the one thing V12 forbids.

  B26 had already built the repair and keyed it on the wrong thing.
  `names_a_spec_file` tests the LINE, which is true for the §F row it was
  found on and false for every prose citation in the node — so it fired on
  the one line that needed it least and stayed quiet where the reader had
  nothing to go on:

  ```
  src/SPEC.md:11: microlith/V13: `V10` is cited but never declared
      why: a dangling reference reads as authoritative, so nobody follows it
      judgment: this spec federates with api, db -- if `V10` is a rule of one of them,
                write it in backticks as `api:V10` (V19, §F)
      judgment: point it at the rule that was meant
      judgment: declare V10, if the rule is real but missing
  ```

  The candidates are read from the file that already lists them: §F's `dir`
  column and §N's `path`. It NAMES rather than RESOLVES — which node declares
  the rule needs the other files, and a check over one `&str` should not
  pretend otherwise — so it narrows the hunt and says that is what it is
  doing. A spec that declares no edges is told nothing, because advice
  offered everywhere is advice nobody reads.

  A test asserts the recommended form actually passes. Worth knowing if you
  hit this through another tool: a bare `src/api:V10` does **not** resolve
  here, whatever a wrapper may do with it — backticks are what make a
  qualified id a literal, which is FORMAT-EXTENSIONS.md's own §F sentence.

- **`tasks` and `check` no longer answer silence for rows they cannot read**
  (V49, B39). A spec whose §T is written as a markdown table came back
  `{"tasks":[],"unread":0}`-shaped — an empty array and exit 0 — with the
  rows sitting right there. A consumer enumerating tasks could not tell an
  empty backlog from an unreadable one, and the only verb that knew was
  `migrate --check`, which a reader has no reason to run first.

  `check` now reports it, once per section, naming the count and the one
  action that fixes it:

  ```
  SPEC.md:7: microlith/V49: §T holds 2 declarations in a DIALECT this build cannot read
      why: an unread declaration is not an absent one, and every verb reports it as absent
      mechanical: run `mth migrate <path>` -- the rows convert to the canonical form
  ```

  One finding per section rather than per row, deliberately: B15 measured
  1,227 such rows across 21 fleet specs, and V15's own history is a check
  that fired once per row and was too loud to read.

  `tasks` carries the same number and stays exit 0. The JSON gains an
  `unread` field — additive, so a consumer that ignores it is unaffected —
  and the human count line says `none READ` rather than `none` when rows
  were there.

  Both read `migrate`'s own answer rather than a second opinion about it, so
  a line counted here is exactly a line `migrate` would convert. Giving
  `tasks` a distinct exit code was rejected and recorded: §I promises
  report-only, and a third code would break every caller scripting the
  0/1/2 split.

- **A row `migrate` cannot place is now named, not declined in silence**
  (V47, B38). A spec whose section heading was not recognised *and* whose
  rows were in the bracketed dialect reported clean from `tasks`, `check`
  and `migrate --check` alike — the file with two defects was the one no
  verb could see. The defects masked each other: an unrecognised heading
  means the rows never parse, and rows that never parse are never items, so
  V11's orphan rule had nothing left to fire on.

  `migrate --check` now reports each such row, naming the header that would
  hold it, and exits 1. The row is still not converted — where it belongs is
  judgement (V6) — so naming the section and re-running `migrate` is the
  repair, after which `check` sees the spec for the first time.

  Keying V11 on the heading instead was rejected: FORMAT.md permits ordinary
  `##` headings that open no section, so that rule would fire on prose. It
  reopens if a grammar arrives that says which `##` lines are sections.

### Added

- **`mth tasks --format json` carries each row's milestone** (V52). T41
  exported the partition for a library consumer and rejected this field at
  the time, rightly: a lib caller would have parsed JSON to learn what a
  pure function returns. The CLI caller is the other half, and had no path
  but re-reading the `| M<n> |` grammar — the duplication V7 exists to end.
  T41's row named the trigger ("can follow later w/o breaking this"); it
  fired.

  ```json
  {"id":"T41","status":"x","text":"...","cites":["V6","V7","V15"],"milestone":"M8"}
  ```

  Read by **number**, not by label: a suffixed row rides its base (V14), so
  `T7a` belongs to whichever milestone claims 7. A lookup on the written id
  finds nothing and calls it unclaimed — a wrong answer that reads exactly
  like a right one.

  `null` is two facts a row cannot separate: a spec that declares no
  milestones (V15 opted out) and a row the declared ones left out (V15
  firing). So `declares_milestones` sits beside the array and says which,
  and the row keeps one shape. Not a missing key and not `""` — a key that
  comes and goes makes every reader test before indexing, and an empty
  string is a value somebody has to know is special.

  Both fields are additive, so a consumer ignoring them is unaffected.

- **`mth archive` — the compaction sink** (V48, T39). Moves a finished task's
  TEXT to `SPEC-ARCHIVE.md` and leaves a stub row in its place:

  ```
  $ mth archive --check --records .spec-records SPEC.md
  mth: 28 rows would move to SPEC-ARCHIVE.md (26802 chars): T1, T2, T3, ...
  mth: 5 held back -- T6, T10, T20, T26d, T29 carry a closed-option record (V16)
  ```

  The row stays; only the text leaves. A stub keeps the id, the status and
  the citations, so an id is never reused (V12), a milestone still finds the
  row it claims (V15), and the citation graph `derive` reads is unchanged.
  Lifting the rows out entirely was measured first and rejected: it makes
  every milestone claim a task with no row, and the milestone-at-a-time
  variant that was meant to rescue it qualified three of twelve milestones,
  because one held row blocks a whole claim.

  A row carrying a closed-option record is held back and named — that is the
  one thing compaction may never trade for bytes — so pass `--records`, or
  those rows move too.

  The move is proven before either file is written: every row arrived byte
  for byte, every id stayed, every citation cell was carried. The sink is
  written first, so a failure between the two writes leaves the text in both
  files rather than in neither. A duplicate is a finding somebody fixes; a
  hole is data nobody gets back.

  `--check` reports and exits 0. It is not a gate: V10 names the gates and
  closes the list, and a spec with finished work in it is an ordinary spec.
  When a file is big enough to fold is the caller's threshold, measured with
  a tool that can count tokens — which this crate deliberately cannot.

  New library items: `archive_spec`, `archive_report`, `ARCHIVE`.

### Changed

- **The `SPEC.md` ceiling came down, for the first time in this file's life:
  36,800 → 34,100.** Every previous entry in `.context-limits` argues a
  raise. `0.7`'s raise to 36,800 was taken because the runner had to land
  somewhere that day, not because the file had earned the room, and T39 said
  so at the time. The fold moved 28 of 33 finished rows and took the spec
  from 36,757 to 29,738 o200k; the new ceiling follows the same ~13% headroom
  ratio every previous line used, so a lowering cannot quietly become a
  stricter rule.


## [0.7.1] — 2026-09-19

A patch, so it sits off the version ladder above: `0.7`'s answer to *what can
you rely on at this tag?* is unchanged. Nothing that `0.7.0` shipped changed —
the CLI surface, the JSON output and every existing library item are as they
were. One library item is **added**, which Cargo's 0.x rules allow in a patch:
a consumer on `0.7.0` gets it by `cargo update` and loses nothing.

It ships now, rather than waiting for `0.8`, because a consumer is dogfooding
it: a federated planner filtering `§T` by milestone needs the partition, and
without a release its only path is a second reading of the grammar.

### Added

- `milestones(text)` at the crate root: each `| M<n> |` row's id and the task
  numbers its tasks cell claims, ranges expanded, in file order (T41). A
  planner that filters `§T` by one milestone can call it instead of re-reading
  the milestone grammar V15 already owns.

## [0.7.0] — 2026-09-05

**Odd minor: functional, not for production** (§V.34). This is the rung where
one spec stops sitting alone at a project root and spans a directory tree
instead: `§F` declares the edges a directory owns, `§N` the derived navigation
to its neighbours, and the row codec every federated consumer would otherwise
re-implement is exported rather than re-read.

`0.7` was previously planned as the gate-hardening rung. It was reassigned
because two milestones had claimed one number and only this one was ready —
the gate rung is now `0.8`, and mechanical editing moves to `0.9`. The ordering
that mattered is intact: a verb that writes a consumer's spec still lands after
a gate you can trust, not before it.

### Read this before upgrading — it is a minor because it breaks

`V42` is a new check, and it is not silent on files that pass today. Measured
against this project's own history rather than asserted: the `SPEC.md` that
shipped inside the `0.6.1` `.crate` is green under `0.6.1`'s checker and **red
under this one**, exit 1, on eight rows — `T5`, `T7c`, `T10`, `T15`, `T20`,
`T23`, `T29`, `B15`.

That is the whole reason for the version. The additions below would have been
legal in a patch, because a `0.x` patch may add public API and `^0.6.1`
resolves it. A check that turns a passing spec red may not: a consumer pinned
to `^0.6.1` would have picked this up with no action and watched green CI go
red on a file they had not touched.

Across a wider corpus of 256 distinct specs the effect is **32 rows in 10
specs, and 7 specs that pass `check` today will not after upgrading** — 1.4% of
rows. Low enough to be worth printing, not low enough to arrive unannounced.

Each hit is a real defect rather than a style preference: the row's last field
is not what its author wrote, and any consumer reading that field has been
believing a fragment of prose. The fix is to escape the pipe, or to reword so
the row carries none — we took the second route in our own spec, because `\|`
inside a code span renders its backslash and made the document worse to read.
That trade-off is being raised with the format's author.

### The additions

Every one of them is optional: a spec that uses none is checked exactly as it
was before.

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

- **The row codec is public: `cells`, `escape`, `unescape`.** A pipe row is
  the one construct in this format a consumer cannot avoid re-implementing —
  `§T`, `§B`, `§F` and `§N` are all rows — and until now the splitter, the
  decoder and the encoder were either private or absent, so every consumer
  wrote its own reading of one sentence in `FORMAT.md`. They ship as a set
  because the round trip is the property worth having: `escape` writes what
  `unescape` reads, and `unescape` decodes exactly what `cells` split.

### Changed

- **`derive` no longer calls a retired invariant an orphan.** That report asks
  whether a rule is dead or merely uncited, and a supersession mark is somebody
  answering it.
- **`--help` wraps its verb list.** It never did, and the eighth verb pushed
  the line to 83 columns — past the width every other line in that output is
  held to.

### Fixed

- **`migrate` deleted whitespace beside an escaped pipe.** Converting a
  markdown-table row split it on *every* pipe, including an escaped one, then
  trimmed each fragment — so a row writing `` `\| grep` `` came back as
  `` `\|grep` ``. It is a lossy write under a verb whose losslessness proof
  passed, because that proof asserts every alphanumeric run survives and a
  space is not one. Measured across 673 fleet specs: 116 files migrate
  differently now, 33 distinct texts over 4 projects.

- **`unescape` was not the inverse of the splitter it decodes for.** It
  rewrote `\|` and left `\\` doubled, while the splitter spends both
  characters — so a cell whose author wrote one backslash reached every
  reader with two, `mth tasks --format json` included. A backslash before
  anything else stays literal, as it always did in the splitter, so a path
  in a cell is unharmed.

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
  **streak** is a `0.8` condition and this is not one yet.

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

[Unreleased]: https://github.com/pr0d1r2/microlith/compare/v0.7.3...HEAD
[0.7.3]: https://github.com/pr0d1r2/microlith/releases/tag/v0.7.3
[0.6.1]: https://github.com/pr0d1r2/microlith/releases/tag/v0.6.1
[0.6.0]: https://github.com/pr0d1r2/microlith/releases/tag/v0.6.0
[0.5.0]: https://github.com/pr0d1r2/microlith/releases/tag/v0.5.0
[0.5.0-rc.1]: https://github.com/pr0d1r2/microlith/releases/tag/v0.5.0-rc.1
