# SPEC.md FORMAT -- EXTENSIONS

Cavekit's `FORMAT.md` fixes seven sections. This document is the DELTA a
project may add on top of it, and nothing here needs a particular tool to be
true: a spec that follows it is still a cavekit spec, read by hand or by
whatever you already run.

The vendored `FORMAT.md` beside this file is upstream's, unmodified. See
`.format-upstream` for the exact revision it matches. This file never edits
it -- it says what is ADDED, so the difference between the two is a thing a
reader can find rather than a thing they have to notice.

**Every extension section is OPTIONAL.** A spec carrying none of them is
unchanged, and must stay that way: an extension that makes existing specs
fail is a fork, whatever it is called.

## SECTION ORDER

Fixed order, fixed headers, addressable -- the same rule upstream states. A
section may be absent, but is never reordered.

| # | section | defined by |
|---|---------|------------|
| 1 | `## §G GOAL` | cavekit `FORMAT.md` |
| 2 | `## §F FEDERATION` | **this document** |
| 3 | `## §N NAV` | **this document** |
| 4 | `## §C CONSTRAINTS` | cavekit `FORMAT.md` |
| 5 | `## §I INTERFACES` | cavekit `FORMAT.md` |
| 6 | `## §R RESEARCH` | cavekit `FORMAT.md` |
| 7 | `## §V INVARIANTS` | cavekit `FORMAT.md` |
| 8 | `## §T TASKS` | cavekit `FORMAT.md` |
| 9 | `## §B BUGS` | cavekit `FORMAT.md` |

## THE SECTIONS

### `## §F FEDERATION`

The header must carry **`federation`** -- matched as a stem, case-insensitively, with qualifiers free to follow. `## §F FEDERATION`, `## §F Federation` and `## §F — Federation` all name it.

The edges this directory DECLARES. A spec federated over a directory tree is one file per directory, so each one names the tree it belongs to rather than restating it. Bullets, no ids -- an edge is a fact about the file, not an addressable item. A citation that crosses an edge names the file it crosses to, because a bare `§V.2` addresses THIS spec and nothing else.

```
## §F FEDERATION
- up: `../SPEC.md` -- the parent this spec refines.
- down: `worker/SPEC.md`, `store/SPEC.md`.
- `worker/SPEC.md §V.2` -- how a citation crosses.
```

### `## §N NAV`

The header must carry **`nav`** -- matched as a stem, case-insensitively, with qualifiers free to follow. `## §N NAV`, `## §N Nav` and `## §N — Nav` all name it.

The DERIVED half: what a reader follows to reach a neighbour. Written down rather than computed at read time, so a spec read on its own still says where it sits. Independent of the section above -- a leaf declares no edges and still has neighbours -- so neither implies the other.

```
## §N NAV
- parent: [../SPEC.md](../SPEC.md)
- siblings: [../api/SPEC.md](../api/SPEC.md)
```

## THE MARKERS

A marker is an extension that is not a new section but a note ON a line. It
is addressed by its words rather than by a letter, so it needs a canonical
spelling for the same reason a section does: two projects writing one thing
two ways is one thing nothing can find twice.

Markers are written in square brackets at the end of the statement they mark,
and inside backticks they are literal -- an example of a marker is not a
marker, exactly as an example of a citation is not a citation.

### `[superseded by ...]`

Written on a `§V` statement. A rule that has been REPLACED, marked rather than deleted. Deleting it would free the id for reuse and strand every citation that still names it, so the statement stays and the mark says it is no longer in force. More than one replacement may be named, because a rule that is SPLIT is replaced by several. The rules named must be live: pointing at a statement that is itself superseded sends a reader to law that is also dead, so the chain is written to its live end.

```
V3: **the old rule.** [superseded by V9]
V4: **a rule that was split.** [superseded by V10, V11]
```

## ADDRESSING

`§<S>.<n>` addresses item `n` of section `S`, exactly as upstream
defines it -- `§F.2` is the second edge declared.

This is the whole reason a letter must mean ONE thing everywhere. A citation
resolves against the READER's section set, not the writer's, so two projects
spelling one letter for two concepts break every citation that travels
between them -- silently, because both files still parse.

## ADOPTING THESE

Copy this file next to your `FORMAT.md` and write the sections. Nothing else
is required: the letters are ordinary cavekit sections that upstream has no
opinion about yet, and a spec that uses them is legal to every reader that
tolerates an unknown letter -- which upstream's own rule requires.

If upstream adopts a letter, it stops being an extension and leaves this
document. That is the intended end state, not a defeat: this file exists to
record what one copy leads with, and a shorter one means the lead was taken.

## THIS FILE IS GENERATED

Rendered from the same constants the checker reads, and frozen by a test --
so it cannot describe a format the tool does not enforce. Regenerate with
`mth extensions`; never edit it by hand.
