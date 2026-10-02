# Compiler history completion audit

Tracks [#6880](https://github.com/ubugeeei-prod/vize/issues/6880) and the
shared product contract in [#6891](https://github.com/ubugeeei-prod/vize/issues/6891).

## Closure correction

PR #7308 closed #6880 on 2026-10-01 at 08:16:47 UTC after its SSR model and
production Vapor changes. That merge did not meet the issue's whole-history
fixture scope. The preceding issue comment and committed records explicitly
retain 15 published distinct fix links / 23 inputs, unreconciled history,
unfinished shared registration and zero native acceptance. The compiler
product switch remains gated until the actual full scope passes. The issue was
[reopened on 2026-10-01](https://github.com/ubugeeei-prod/vize/issues/6880#issuecomment-5929525145).
Preserve #7308's valid regression fixtures without counting its partial changes
as whole-history completion.

## Reproducible full audit index

[`compiler-fix-history.tsv`](../plan/compiler-fix-history.tsv) lists every
conventional non-merge fix subject under the issue's eight named crate paths
at `9aaa1fe458a09e0d0c6604dc8835ccf7c737d943`. Regenerate or compare it with:

```sh
vp node tools/commands/fixtures/compiler-fix-history.ts --write
vp exec oxfmt docs/davinci/plan/compiler-fix-history-index.json
vp node tools/commands/fixtures/compiler-fix-history.ts --check
```

The private hash helper accepts `string | Uint8Array` for standalone
type-aware linting. Node Buffers already extend Uint8Array; this annotation
adds no runtime parsing or changed audit/capture bytes.
The tool requires a complete local graph and never fetches implicitly. Its
full-history traversal has 1,321 touching commits / 661 conventional fix
subjects including merges, and 626 non-merge fix subjects. Ordinary path
simplification gives the previously reported 1,208 / 631, but the same 626
non-merge fixes. The last available pre-issue main `0662b996b` gives 624
non-merge fixes; 63 nearby commit tips did not recover the issue's unpinned
1,152 / 609 snapshot. No original snapshot revision is invented.

Two stable patch-equivalent pairs are recorded as review candidates. Their
624 unique patches are not a semantic denominator. Changed witness paths do
not establish coverage, and no row is automatically marked covered, excluded,
superseded or duplicate. Each retains explicit pending semantic/target review.
The receipt binds the index digest and original history graph command.

The 15 published fix links include #428 `0de7787ae`, a two-parent merge.
Its first-parent compiler-path delta has the same stable patch ID as its
source commit `14a4ff375`. This explicit, checked mapping supplies the
non-merge row's original fixture reference without rewriting old provenance.
Together with the five prepared SFC cases, 20 rows have bounded fixture links;
neither the remaining row count nor those links establish semantic completion.

## Five preserved SFC Results

The first executable wave restores five prepared original input/option/full
public Result references: imported component before a same-name prop #4538,
scoped CSS v-bind #583, dynamic loop ref_for #4536, computed component unref
#1986 and custom directive child patch #4514. Their original public entrypoint
and filename/scope/TypeScript/separate-template options remain unchanged.
All 20 files match archive `026214e44` and source `ede1ea23d`; original and
repeated raw observation bytes agree. The CSS Result retains all 77 CSS bytes.
All five maps are null and diagnostics empty; populated forms remain uncovered.

The bounded archive retains a compact source/tree/build receipt, actual selected
Cargo artifact, historical observer source and repeated observations/stderr.
The original full source receipt and raw build log remain in the private audit
archive; their recorded hashes do not claim those files are committed here.
That historical source commit is not yet a remote provenance reference.
Read-only archive checks do not certify fresh current-head execution.

Publication requires exact-head Actions for the ordinary five-case product
test, unchanged all-100 gates, protected queue and actual main merge. These
five links remain prepared until then. The dependent target layer preserves
five SSR and five Vapor cases and registers their original fix links through
this manual index API; 30 linked rows still all await semantic review. Next:
exact-head execution of the shared API registration, every remaining semantic
history review and target/dialect fixture gap. Native acceptance stays zero;
the whole-product native compiler is still unavailable. No product route,
output golden, instruction ceiling or old shape/runtime witness changes.

## Shared public compiler API registration

The SFC, SSR and Vapor manifests in `tests/_fixtures/differential/compiler/`
reference the same five original inputs/options per target and complete
observed public outputs in the parent layers' product fixture directories.
They use the existing shared manifest/result v1 contract and retain every
planned case ID and `dom`, `ssr` or `vapor` coordinate. Fixtures are not copied
or regenerated from current output. Both parent fixture layers are actual
dependencies of this registration.

The three `*_fix_history_observer` examples invoke the original public
entrypoints with their original options, including SFC filename/scope/
TypeScript/script-output context and Vapor's mounted-case identifier prefix.
They record all seven SFC Result fields, all four SSR fields (`code`, `preamble`,
`map`, `diagnostics`) and all four Vapor fields (`code`, `templates`, `map`,
`errorMessages`), plus source bytes and exhaustive actual option snapshots.
Examples include inputs only, never expected Results. The observer-build
provider genuinely merged in #7362 binds each real Cargo-selected executable
and probe transcript to the committed source tree, lockfile, toolchain and
observer source; this consumer adds no competing build framework.

The adapter captures two fresh full stdout observations, requires exact repeat
bytes and empty stderr, compares the complete public Result with its historical
reference, then reobserves the source-bound executable when validating a report.
JSON object ordering is transport-only; module/CSS bytes, nulls, scalar values,
array ordering, maps, diagnostics, bindings, macro artifacts, SSR preamble and
Vapor template bytes are preserved.
Missing builds, missing/duplicate cases, altered options or failed processes
retain all five failed planned rows per target. Native rows stay `unsupported`, comparison
stays `not-compared` and whole-product native acceptance remains zero.
The Actions wrapper catches build and contract-probe failures, writes those
failed rows with the original error, then fails the test. Each actual first,
repeat and final revalidation attempt is saved before comparison; a final
receipt or observation failure saves a failed report without native credit.
The prevalidation report is explicitly retained as unverified evidence.

Eight pure adapter contracts pass locally. Actual source-built execution of
all three observers remains pending exact-head Actions. The existing merge-only
tooling execution law builds each observer, requires five matching references
per target, and retains zero handled/equivalent native rows. Build logs,
selected artifacts, receipts and repeat bytes
are Actions artifacts under `target/differential/compiler-api/`, not committed
whole-repository evidence ledgers. The immutable original sources are now
published as [SSR `c2451a0fb`](https://github.com/ubugeeei-prod/vize/commit/c2451a0fb39bf7390c51630cfe28ae0c1b06a870)
and [Vapor `b11eb54d8`](https://github.com/ubugeeei-prod/vize/commit/b11eb54d83b3d8ed6333e2709b10112b45cd84b8)
through the provenance heads recorded in the target companion. Original tree,
parent, raw commit and `+0900` identities matched before those references were
created; their timestamps were not normalized or substituted. Fresh-clone
captures can inspect the actual original objects. The private SFC `ede1ea23d` ancestry
contains an excluded research ledger and must remain unpublished. These
historical pins and fresh Actions are separate evidence. Whole-history
semantic review, remaining target/dialect gaps and the product-switch gate
remain unfinished; #6880 stays open and native compiler credit stays zero.
