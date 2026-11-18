# TSRX node identity: keep Vize's conditionals and loops

Decided 2026-11-18 for
[#13142](https://github.com/ubugeeei-prod/vize/issues/13142),
[#13176](https://github.com/ubugeeei-prod/vize/issues/13176) and
[#13180](https://github.com/ubugeeei-prod/vize/issues/13180): option A.

## Decision

The TSRX Vue profile keeps Vize's own `v-if`/`v-for` node identity. A
conditional whose test re-evaluates to the same branch keeps that branch's
nodes, a loop keeps its item keying (with or without `@empty`), and a static
branch body is created per render. Vize renders no text beside a component or
fragment child inside an element. The pinned reference (`@tsrx/vue` 0.3.5 +
`vue-jsx-vapor` 3.2.26) re-creates nodes on each getter re-run, hoists static
bodies to module-level nodes shared across instances, and leaves a `" "`
placeholder text node beside such a child. Vize does not adopt any of these.
This is what the code already does; nothing changes in output.

## Why

Stable identity keeps DOM state such as focus, selection and typed input
when a same-branch re-evaluation or a list update leaves the content in place.
It matches Vue template semantics, so a TSRX component behaves like the same
component written as a Vue template. The reference behaviors are artifacts of
its runtime, not TSRX semantics, and the specification leaves node identity to
the host. A hoisted node shared by two mounted instances cannot be reproduced
without that cross-instance bug.

## Permanent expected differences

These rows in `tests/_fixtures/tsrx/mount/mount.json` render the same DOM as
the reference except as noted, and must keep differing:

- `loops/nested`: no placeholder text after the loop (#13142).
- `nested/recount`, `and/recount`, `map/update`: nodes kept on a same-branch
  re-evaluation or an array change (#13176).
- `empty/update`, `empty/toggle`, `branches/blocks`: loop keying kept, and the
  empty or static branch created per render (#13180).
