# Integration

How a change gets from an edit to `main`, what checks it at each point, and why
the order is what it is.

The short version: **the gate is a set of git hooks that also run on CI.** Every
check runs on your machine before the push, and CI runs the same steps from the
same definition — not a parallel pipeline that can disagree with your laptop.

## One definition, three callers

[`hk.pkl`](../hk.pkl) defines every check exactly once. Three things call it, and
none of them redefines anything:

```text
                        hk.pkl
              one definition, 31 steps
                          |
        +-----------------+-----------------+
        |                 |                 |
   pre-commit         pre-push           ci.yml
   28 steps           all: 31 steps      all: 31 steps
   (fast + 1 local)
```

`all` is not a second list. In `hk.pkl` it is literally the fast set plus four:

```pkl
local all = (fast) {
  ["rustdoc"]         { ... }
  ["deny-advisories"] { ... }
  ["semver"]          { ... }
  ["coverage"]        { ... }
}
```

Adding a cheap check means editing `fast`, and pre-push and CI inherit it. There
is no second copy to forget.

## The path a change takes

```text
  edit
    |
    v
  git commit ---> pre-commit  (28 steps)       ---fails---> fix, retry
    |                                                           |
    | passes                                                    |
    v                                                           |
  commit lands <------------------------------------------------+
    |
    v
  git push   ---> pre-push    (all, 31 steps)  ---fails---> fix, retry
    |
    | passes
    v
  branch pushed
    |
    v
  pull request ---> ci.yml    (all, 31 steps -- same definition)
    |                          + nix build .#default
    | green, and reviewed
    v
  merged to main
```

**pre-commit** runs the fast set with `fix = true`, so formatters rewrite rather
than merely complain. It adds one step the other two callers do not have:
`no-commit-to-branch`, which refuses a commit made on `main`. It cannot live in
`fast` — `all` is `fast` plus four, and CI runs `hk check --all` **on `main`**
after every merge, so a branch guard in `fast` would fail every CI run on the
branch it protects. The rule is about *where you are committing*, and CI never
commits. It also sets `stash = "git"`, which is correctness rather
than speed: without it a partially staged file (`git add -p`) would be judged as
it looks in the worktree, giving you a verdict about code you are not
committing.

**pre-push** runs everything, adding the four steps too costly for every
commit: `rustdoc`, `coverage`, and the two that need the **network** —
`deny-advisories` fetches the RustSec database, and `semver` diffs the public
API against the last release tag. Everything in `fast` stays offline, because a
hook you are tempted to bypass is worse than no hook.

**CI** runs `hk check --all --check --no-fail-fast`, then `nix build .#default`
to prove the package builds reproducibly from the tracked lock alone. It is
already wired to fire on `pull_request` as well as on pushes to `main`.

**Changes reach `main` through pull requests, and this is enforced.** GitHub
refuses a direct push to `main`: a pull request is required, all three `gate`
matrix jobs are required status checks, the branch must be up to date, force
pushes and deletions are off, and **admins are not exempt**. Proven, not
configured and assumed:

```text
remote: error: GH006: Protected branch update failed for refs/heads/main.
remote: - Changes must be made through a pull request.
remote: - 3 of 3 required status checks are expected.
```

The local `no-commit-to-branch` hook changes no outcome — it moves that refusal
earlier, to before you have built a commit you then have to move. The server
rule is the one that defends the branch, because V23 makes every hook here skip
outside the dev shell. Requiring a PR does not add a check either — CI runs the
same 31 steps your pre-push hook just ran — it adds a *reader*. The gate
catches what is mechanically wrong; a reviewer catches what is merely a bad
idea, and those are different failures.

## What runs on which files

Two things vary by stage, not one. Which **steps** run is the `fast`/`all` split
above. Which **files** they see is separate:

| stage | steps | files examined |
|---|---|---|
| `pre-commit` | 28 | **staged files only** (hk's default) |
| `pre-push` | `all` — 31 | **everything in the push**, computed from the ref range git hands the hook: `Fetching files between refs/remotes/<remote>/main and HEAD` |
| CI | `all` — 31 | **every file in the repo** (`hk check --all`) |

`fast` is a strict subset of `all`, so every step that gated your commit gates
your push again — over a wider set of files.

## The steps that guard claims, not code

Five steps here do something different from linting: they check that a sentence
written somewhere else in this repo is still **true**.

| step | the claim it enforces | where the claim lives |
|---|---|---|
| `deny` | zero dependencies | `README.md`, `Cargo.toml`, `AGENTS.md` |
| `package` | the `.crate` ships the files the test suite reads, and its file set has not moved | `Cargo.toml`'s `must-package`, `.crate-files` |
| `semver` | the version number means what V30 says it means | `SPEC.md` §V.30, §V.34 |
| `readme-badges` | the coverage percentage, MSRV, edition and platform list | `README.md` badges |
| `format-upstream` | `FORMAT.md` matches the upstream version it records | `.format-upstream` |

Each exists because the claim was already being made and nothing was checking
it. A number or a guarantee stated in prose is true the day it is written and
silently wrong afterwards — which is V17 in one sentence: a rule with no runner
gates nothing.

`README.md` is in the `coverage` step's glob for the same reason `SPEC.md` is in
`test`'s: that step reads it. The coverage percentage in the README badge is
**checked against the measurement that just ran**, so it cannot quietly go stale
— the same freeze the `mth docs` block gets, rather than a third-party coverage
service and the account, token and network dependency that would come with it.

hk offers a `--pr` flag, *"check only files changed in the current PR/branch"*,
and CI deliberately does **not** use it. A changed-files scope cannot see damage
to files nobody touched: widen a glob, rename a step, let a fixture rot, and
nothing in the diff points at the breakage. Pre-push proves *your change* is
clean; CI proves *the repo* is.

**Scope and glob multiply.** A step runs only when a file in scope also matches
its `glob`. This repo has been bitten by exactly that: `SPEC.md` was missing from
the `test` step's glob, so a spec-only commit matched nothing and skipped the
entire suite — including the dogfood assertions whose whole subject is
`SPEC.md`. Four commits landed that way before a hand-run `hk check --all` found
a slack breach the hook had never looked for. Hence `SPEC.md` and
`.spec-records` now sit in the globs of `test`, `microlith`, `microlith-check`
and `coverage`.

## Why the steps are chained

Cargo takes a lock on the target directory, so two cargo jobs launched in
parallel do not run in parallel — the second blocks on *"Blocking waiting for
file lock on build directory"*, which reads as a hang. The chain makes that
serialization explicit and leaves hk free to run everything else concurrently:

```text
  serialized by depends -- cargo locks the target dir:

    fmt --> clippy --> test --> doctest --> microlith --+--> microlith-check
                                            fmt own     |    check own spec
                                            spec        |
                                                        +--> rustdoc --> coverage --> semver
                                                                         floor 98%   vs last tag

  no depends, so these run concurrently:

    trailing-whitespace   final-newline       line-endings
    smart-quotes          no-bom              no-merge-conflict
    no-private-key        ripsecrets          no-large-files
    no-case-conflict      no-broken-symlinks  actionlint
    typos                 links               taplo
    nixfmt                deny                package
    readme-badges
```

Ordering is cheapest-first on purpose. `fail_fast = true` locally, so the first
failure is the one you see and it arrives quickly. CI inverts this with
`--no-fail-fast`: there a round trip costs minutes, so a complete list beats an
early one.

The last two cargo steps are **the dogfood** — microlith formats and checks its
own `SPEC.md` using the binary being built. A rule this crate cannot pass is a
rule it cannot ship.

## Where spec-driven development fits

The spec is not documentation sitting beside the gate — it is an **input to
it**. Four steps list `SPEC.md` and `.spec-records` in their globs, and two of
them do nothing but check the spec itself.

```text
   SPEC.md  -->  rule + runner  -->  planted violation  -->  companion
   the law       same commit         proves it rejects       proves it accepts
      ^                                                            |
      |                                                            v
      +---------------------  dogfood  <------------------------  gate
```

That last edge is the point: `microlith` and `microlith-check` run the binary
being built against this repo's own `SPEC.md`. A rule this crate cannot pass is
a rule it cannot ship.

Minified, the loop the gate enforces:

```text
spec first          ⊥ code first ∴ the rule exists before the thing it governs
rule + runner       SAME commit (V17) ∵ a rule with no runner gates nothing
planted violation   the guard goes RED on purpose (V18) ⊥ merely reviewed
+ companion         & still accepts every real shape ∴ ⊥ passing by rejecting all
dogfood             our own SPEC.md is the first file we format & check (V24)
one definition      a rule lives HERE, once, called by consumers (V7)
never bypass        a check that is wrong is a SPEC change, ⊥ a --no-verify
```

Read [`SPEC.md`](../SPEC.md) for the full set. [`CONTRIBUTING.md`](CONTRIBUTING.md)
walks the loop for a first-time change.

## Getting the hooks

Entering the dev shell installs them and rewrites them every time, so the hook
you have is always the one [`flake.nix`](../flake.nix) describes:

```sh
nix develop          # or: direnv allow
```

They are installed **by hand** by the shellHook rather than by `hk install`,
which buys one specific behaviour: outside the shell, the hook finds no `hk` on
PATH and **skips loudly** — one line on stderr, exit 0.

```sh
hk: not on PATH -- SKIPPING the pre-commit gate. Enter the dev shell
```

A gate exists to stop bad commits, not to stop git. But that line means
genuinely ungated rather than merely local, so enter the shell.

## Reproducing any verdict without hk

No verdict rests on hk's own logic. Every step is a plain cargo command you can
paste into a shell:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo nextest run                                    # or: cargo test
cargo test --doc
cargo run -- fmt --check SPEC.md                     # the dogfood
cargo run -- check --records .spec-records SPEC.md
cargo doc --no-deps                                  # RUSTDOCFLAGS="-D warnings"
cargo llvm-cov nextest --fail-under-lines 98
cargo deny check bans licenses sources               # offline: the zero-dep guarantee
cargo deny check advisories                          # network: RustSec
cargo semver-checks check-release --baseline-rev v0.5.0
cargo package --list                                 # what the .crate would ship
```

hk decides *when* things run. It never hides *what* runs.

## Two rules that shape all of this

**Never bypass.** `--no-verify`, lowering a threshold, deleting a test, or
adding `#[allow]` to silence clippy all ship the defect with the alarm switched
off. If the check itself is wrong, that is a spec change — say so in
[`SPEC.md`](../SPEC.md), in its own commit.

**A step's glob must name every input its tests read**, not just the language it
is written in. `tests/dogfood.rs` reads `SPEC.md` and `.spec-records`, so every
step that runs it lists them — see *What runs on which files* above for what
happened before they did.

## Known gaps

Listed rather than silently absent, because a gap you can read is not the same
failure as a gap you cannot.

| gap | where it goes |
|---|---|
| a green **streak** before `0.7.0` — one green run is not a streak | T26 |
| build caching (nix store + cargo target) | T26 |
| release automation and publish | T8 |

`shellcheck` is **deliberately** absent: there are no shell scripts here yet,
and a linter with nothing to lint is the same shape as a rule with no runner.

## Deeper

[`hk.pkl`](../hk.pkl) is the definition and is heavily commented — every step says
why it exists. [`AGENTS.md`](../AGENTS.md) is the working guide,
[`CONTRIBUTING.md`](CONTRIBUTING.md) the arrival path, and [`SPEC.md`](../SPEC.md)
holds the invariants this all enforces (V17, V22, V23, V24).
