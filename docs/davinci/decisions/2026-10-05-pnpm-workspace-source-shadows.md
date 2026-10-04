# Preserve discovered workspace package sources across route refresh

Issue: [#7834](https://github.com/ubugeeei-prod/vize/issues/7834), reopening
the practical regression reported in [#6982](https://github.com/ubugeeei-prod/vize/issues/6982).

The reported package chain is `@x/a → @x/b → @x/c → ./util → ./Btn.vue`.
All five authored files exist. Root workspace links and legitimate pnpm
per-package links must resolve through materialized Vue companions. The
previous #7121 regression used manually assembled routes and did not exercise
the original real relative-link topology or subsequent route replacement.

## Decision

Retain the existing physical-package source index across an atomic replacement
of its final route owners. Entry points listed in package exports are only a
subset of the relative source files already registered for that package.
Removing and reinserting the same route previously deleted the whole index,
then restored only the exported entry points. This can omit a barrel's `util.ts`
and the Vue generated companions from nested package shadows.

The retention operation moves metadata for affected roots only. It restores an
entry only when the physical root still has route owners and the current
original-source index still points at that live virtual file. Deleted sources
and roots whose import edges disappeared remain removed. The entire importer
reconciliation wave and the one-key refresh both use this boundary. There is
no new source parse, whole-project source scan, resolver policy, generated-byte
transformation or source-map interpretation.

Workspace package copies also retain every already-registered bare-package
binding at its exact authored importer-relative directory. A nested real pnpm
installation can otherwise outrank a hoisted sibling package shadow and lead
native resolution out to raw `.vue` files. The new nested shadow uses the
existing resolver's binding and package name; distinct importing directories
keep their physical package identities. Non-workspace package copies and
private `#` routes retain their existing handling. The ancestor manifest guard
still bounds cycles. This adds materialized package topology, not a new parse
or resolution stage, and does not change the original symlinks.

The existing `package_routes.rs` remains at its grandfathered 357 lines.
New helpers and fixtures remain below 350 lines. Package shadows, manifests,
Vue companions and installation links retain their existing ownership rules.

## Required proof

- The literal original five-source reproduction is retained in
  `tests/fixtures/typechecker/pnpm-workspace-routes`. `links.json` records the
  exact root links and pnpm `../../../b`/`../../../c` links. Tests create those
  links and do not flatten them or substitute tsconfig aliases.
- The transport regression checks complete registered membership, materialized
  shadow bytes, the nearest native package-directory walk and the complete ordered source/code/mapping/semantic-link rows
  across route replacement and reconciliation. Negative controls reject stale
  deleted-source and removed-root resurrection.
- The registered full Vue-parity CLI oracle checks root and pnpm layouts with
  one and two native servers, explicit chained entry, direct leaf and default
  collection. A separate typed Vue/event consumer rejects `any`, plants one
  mapped TS2322, and checks the complete ordered diagnostic vector and repaired
  virtual TS/report, original bytes and raw symlink targets. Raw command
  streams/status, source/CLI/native identities, input hashes and full reports are retained under
  `target/vize-tests/metrics/pnpm-workspace-routes` in the existing parity lane.

The literal package manifest retains the reporter's Vue 3.5.38 and TypeScript
5.9.3 requests. The Actions oracle uses the repository-installed Vue types and
pinned Corsa; this is not a claim to have installed those historical versions.
The unrelated Pinia TS2322 count mentioned in #7834 is outside this repair.
No CPU, startup, throughput or 10x result is claimed.

## Validation status and remaining work

Base is actual main `61c975f8889a002a1b53265d86d0985447ffab63`. The focused
configured Vite+ formatter/type-aware lint reports zero warnings, and the
six manifest controls pass. No local Cargo or native execution has been used.
The source transport regression and actual CLI/native gates require exact-head
Actions. PR Rust runs disable native fixtures, so their success alone cannot
establish the CLI claim; the existing manual full Check's Vue-parity lane is
required before safety acceptance. Exact hosted results, the original failing
baseline and protected queue/actual merge receipts remain TODO. The root's
safety hold forbids entering the queue until that evidence is reviewed.

Provisional source `e9643d724a067cb33158dd9d8e61a9283352e627` started full
Check `37225449810` and PR Check `37225424550`. The first transport source
compiles, but its source-inventory shard needed regeneration and the new
assertion oracle needed the explicit snapshot declaration. The parity lane
failed before executing the native command because its cached Vite+ wrapper
does not forward `VIZE_TEST_BIN`; the successor binds the lane's actual
`target/ci/vize` build directly. These are retained failed receipts, not
evidence of original-backend failure or a completed repair.

Both #7834 and #6982 currently identify `ubugeeei` (GitHub numeric id 71201308)
as their author. The commit includes that verified issue-author Co-author
trailer, without inventing a separate external reporter identity.
