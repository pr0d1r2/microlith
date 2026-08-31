# SPEC

## §G GOAL
one line, prose -- what the code in THIS directory must do.

## §F FEDERATION

dir|owns|⊥owns|tokens
worker|the queue consumer and its retry policy|the schema, which is `store`'s|-
store|the schema and every migration|anything that reads the queue|-

## §N NAV

rel|path|lens
up|-|-
self|.|-

## §C CONSTRAINTS
- the edges above are DECLARED, never inferred from the directory layout.

## §V INVARIANTS
V1: **a rule.** cited by T1.

## §T TASKS
| id | scope | tasks | done-when |
|----|-------|-------|-----------|
| M1 | first | T1 | done |
T1|x|a task|V1
