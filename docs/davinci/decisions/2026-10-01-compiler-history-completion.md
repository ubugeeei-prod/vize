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
five links remain prepared until then. Next: real shared SFC API registration,
the five prepared SSR and five Vapor cohorts, every remaining semantic
history review and target/dialect fixture gap. Native acceptance stays zero;
the whole-product native compiler is still unavailable. No product route,
output golden, instruction ceiling or old shape/runtime witness changes.
