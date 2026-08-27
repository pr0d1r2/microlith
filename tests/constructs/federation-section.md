# SPEC

## §G GOAL
one line, prose -- what the code in THIS directory must do.

## §F FEDERATION
The edges this directory declares, as bullets with no ids. A federated spec is one file per directory, so each one names the tree it belongs to rather than restating it.

- up: `../SPEC.md` -- the parent this spec refines.
- down: `worker/SPEC.md`, `store/SPEC.md` -- the children it constrains.
- a citation crossing an edge is written `worker/SPEC.md §V.2`, because a bare `§V.2` addresses THIS file and nothing else.

## §N NAV
The derived half: what a reader follows to reach a neighbour. Written down rather than computed at read time, so a spec read on its own still says where it sits.

- parent: [../SPEC.md](../SPEC.md)
- siblings: [../api/SPEC.md](../api/SPEC.md)

## §C CONSTRAINTS
- the edges above are DECLARED, never inferred from the directory layout.

## §V INVARIANTS
V1: **a rule.** cited by T1.

## §T TASKS
| id | scope | tasks | done-when |
|----|-------|-------|-----------|
| M1 | first | T1 | done |
T1|x|a task|V1
