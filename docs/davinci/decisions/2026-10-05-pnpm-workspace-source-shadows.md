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
  `target/vize-tests/metrics/check-fixtures-topology/pnpm-workspace-routes` in
  the existing uploaded topology artifact. The event control changes the
  emitted number payload to string and requires the authored callback's
  `toFixed` access to fail, rejecting a silent `any` fallback.

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

Successor `f822dde9d237b8c513fa2aa35f4ee4c5958bcc0b` full Check
`37225958714` launched real native commands: all four initial root/pnpm literal
and typed-consumer checks have clean complete diagnostic vectors. The oracle
then failed its own incorrect assertion that JSON `programs.files` lists
transitive package dependencies; that field records prepared input files,
including collected local imports but excluding route-owned package targets.
The next source separates exact root membership from complete reported source
membership, retains the full report, and stores receipts in the lane's actual
uploaded directory. This partial observation does not establish the remaining
server/direct/default/error/repair controls or whole-PR acceptance.

Source `060889cf86ac7ee51caf85013fe6b3ce9747335a` full Check `37226436491`
passes both typed root/pnpm controls, including all native 1/2-server clean,
mapped number error, emitted-string callback error and repaired full-output
checks. The literal controls reached clean chained and direct checks, then
exposed the oracle's remaining input-membership error: direct relative imports
belong to the prepared inputs, unlike route-owned package targets. Only that
expected input list is corrected next; production is unchanged from `f822`.
Uploaded artifact `11311853935` retains 16 unique raw receipts from 18 actual
CLI executions (clean/repaired rows deduplicated); the successor adds execution
ordinals, working directories and CLI PIDs to retain every individual process.

The bounded baseline branch `test/pnpm-workspace-7834-baseline` now has source
`800ccf6b163f89210dd96a0d29102d52d5989a61`, with complete crates/Davinci/lock
trees identical to main `61c975f8889a002a1b53265d86d0985447ffab63` and only 18
fixture/driver paths changed. Its full Check `37226838354` must demonstrate the
original native failure using the same authored inputs and commands. An
initial sparse-index preparation error produced a rejected dispatch (422,
workflow absent); only that proof ref was corrected with an exact lease, and
no native run, main/PR/queue mutation or baseline claim resulted from it.

Both #7834 and #6982 currently identify `ubugeeei` (GitHub numeric id 71201308)
as their author. The commit includes that verified issue-author Co-author
trailer, without inventing a separate external reporter identity.
