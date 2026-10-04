# Preserve workspace sources and native package identity across route refresh

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

Workspace package copies retain already-registered bare-package bindings only
at their selected real in-package installation scopes. A real pnpm install can
outrank a hoisted sibling shadow and otherwise lead native resolution to raw
`.vue` sources. The resolver's selected `package_link_root` is rebased only
when it lies inside the owner's physical package and ends with the selected
`node_modules/<package-name>` entry. External/shared installs keep their old
scope; non-workspace copies and private `#` routes keep their existing handling.

For workspace targets, that rebased entry links to the nearest planned authored
incoming shadow visible from this exact package copy. The physical package root
and raw manifest must both match. A nearer conflicting identity, ambiguous
ownership or self/ancestor cycle declines reuse and retains the bound install's
own source topology. A same-name package or a globally smallest path is not an
identity proof. The alias owner pins the target's existing authored source/Vue
companions and raw manifest, independently of another owner's refresh order.
The authored symlinks and generated source bytes remain unchanged; native
`preserveSymlinks` still controls whether separate visible links have separate
module identity.

Aliases reuse the existing `PackageNodeModulesLink` lifecycle. An alias's
`package.json` claim participates only in link selection and is never an
expected file, file revision or incremental file candidate. Cold and affected
warm paths retain the planned internal target without canonicalizing through a
stale on-disk alias. Target directories exist before link creation, including on
Windows. Existing package-root/importer owner indexes dirty reverse dependents
when a source, selected link or shared scope changes. Each owner's pinned known
physical manifests also retain their dependency bridges; bare dependencies need
not be nested routes of its top-level binding. Conflicting internal alias targets
fail explicitly, including editor snapshot unions, rather than selecting a path
by ordering. Both target and destination ancestry are checked before cold
prune/runtime writes or incremental link/file mutation, including Windows
junction detection. Root-first preparation may retire only an exact previously
committed cache link whose observed target still matches that receipt, using the
existing symlink-safe removal; unknown or retargeted links and non-directories
remain declined. A new CLI process has no such prior link receipt, so migration
of uncertain old cold-cache bridges is not claimed and remains an adoption
review limitation. Preserved internal union aliases contribute only transient
link-selection claims; a raw parent/descendant link overlapping an alias is
rejected before mutation. This adds bounded existing topology metadata, with no source parse,
resolver policy, pipeline stage or full-project source scan.

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
  across route replacement and reconciliation. Persistent source edits, new
  relative files and shared-to-divergent-to-shared real-link retargets compare
  complete cold/warm bytes, maps and committed links. Warm disk bytes and all
  actual read-link/canonical targets are captured before any cold materialize
  can repair them; a second no-op delta must consider zero candidates. A synthetic secondary
  dependency available only under the divergent package proves its physical
  dependency bridge remains present. Negative controls reject stale
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
- Independent private-branded class controls require one physical package to
  retain identity across source directories and A's direct/B's transitive
  imports. A genuinely different package must still produce exactly one mapped
  private-brand TS2322, then repair restores full reports, inputs and raw links.
  Explicit `preserveSymlinks: true` controls check the native distinction for a
  pnpm in-package link versus a root shared install. These synthetic secondary
  controls are not additional reporter input or provider-package attribution.

The literal package manifest retains the reporter's Vue 3.5.38 and TypeScript
5.9.3 requests. The Actions oracle uses the repository-installed Vue types and
pinned Corsa; this is not a claim to have installed those historical versions.
The unrelated Pinia TS2322 count mentioned in #7834 is outside this repair.
No CPU, startup, throughput or 10x result is claimed.

## Validation status and remaining work

Base is actual main `61c975f8889a002a1b53265d86d0985447ffab63`. The focused
configured Vite+ formatter/type-aware lint reports zero warnings, and the
six manifest controls pass. No local Cargo or native execution has been used.
The successor is rebased onto actual main
`688da7cc620af5a9aec82152b54d8052c64c52bc`, preserving its incoming production
changes and decision text. The source transport regression and actual CLI/native
gates require exact-head Actions. PR Rust runs disable native fixtures, so their success alone cannot
establish the CLI claim; the existing manual full Check's Vue-parity lane is
required before safety acceptance. The original failing
baseline and historical successes are recorded below. Exact successor source,
native identity, persistent ownership, full-suite and protected queue/actual
merge receipts remain TODO. The root's
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

Baseline `800ccf6` full Check `37226838354` now establishes the original
native failure: root links produce the `./util` TS2307, pnpm links also produce
the `./Btn.vue` TS2307, and the typed consumer has the related missing-export
or implicit-any diagnostics. Artifact `11312004122` retains four raw native
reports. Source `079375f1eed8adf0e02865db24dd8a73d11fa526` full Check
`37227103854` passes all 26 literal/typed native executions; artifact
`11312313688` retains every process separately. The four corresponding clean
head reports have identical input hashes, raw links, program options,
canonical virtual TS and native binary bytes to the failing baseline. PR
source Check `37227109000` and the manual full Rust suite succeed.

Safety review added one concrete module-identity control: a private-branded
class made in `B/src/x` must remain assignable to the same package's class
consumed in `B/src/y`. Test-only source
`a4acf51d81df3e61b3fcbced2b14cd4f0986e1e6`, full Check `37228684283`,
confirms that the historical `f822` per-importer copies violate #4153. Both
root/pnpm variants produce three false TS2345 diagnostics at the same authored
`B/src/index.ts:3:10` because their private `brand` declarations have separate
materialized module identities. Artifact `11312707356` retains the raw native
reports. The narrowed real-install successor
`885aab2c5bb11f47ff3b11778d059acebc4daf76` retains the original 26 clean/error/
repair executions and passes the root-layout private-class checks. Its full
Check `37229352503` nevertheless exposes one false pnpm TS2322 at
`packages/a/src/index.ts:3:7`: A's direct C shadow and B's nested C copy still
create different private `brand` identities for the same physical package.
Artifact `11313303163` retains all 29 executed raw reports. Its PR source Check
`37229354947` and full Rust/source coverage also fail the new transport control:
that test used the bare local dependency walker without reconciling newly
reached bare-import bindings. The successor follows complete existing provider
orchestration and keeps the assertion that the native target stays inside the
virtual project; it does not weaken the target assertion or attribute this test
setup omission to production.

The owned workspace aliases and lifetime rules above supersede `885`'s duplicate
nested copies. The required successor matrix is 38 actual CLI/native processes:
all original 26, four shared-class root/pnpm one/two-server checks, four genuine
divergent-package clean/error/error/repair checks, and four explicit
`preserveSymlinks` root/pnpm checks. The pure transport suite must also compare
full bytes/maps/committed links after persistent updates, preserve unchanged raw
sentinels for stale target/root-scope and retargeted links, retire only a known
committed bridge, and reject editor-union exact/ancestor alias conflicts. A
separate union retains aliases owned only by a prior snapshot. These controls are prepared, not yet runtime qualification.
No old native success transfers to this successor; the draft/safety hold,
unrelated Pinia limit and lack of any performance claim remain in force.
