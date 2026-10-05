# Pinned upstream Vue benchmark inputs

Decision for [#7856](https://github.com/ubugeeei-prod/vize/issues/7856), 2026-10-05.

## Scope

Register `tests/_fixtures/_git/vue-benchmarks` as a genuine mode-160000
submodule of `https://github.com/pikax/vue-benchmarks` at upstream main
`5489aee433cd1054b9d72973457498544da7c467`. The GitHub commit API returned
that exact main on 2026-10-05; its publication commit is dated 2026-09-29.
Keep upstream files read-only and hydrate only on Actions when required.

The dedicated `tests/_fixtures/vue-benchmarks-upstream.json` records the
upstream MIT license: preserved `LICENSE`, Git blob
`a7017290772b36ce31d5e94645177b289564983e`, 1,084 bytes. API-decoded original
bytes have SHA-256 `c3081f92362a167af8f17890c6876dcdf06c1dbbd328347e9ff696103a538467`.
This registration copies no upstream source or results into Vize.

## Inventory and custody

The executable structural inventory increases from 146 to 147 gitlinks.
The existing canonical Davinci corpus selects and hydrates every gitlink;
its expected source-inventory count therefore becomes 147, with no hidden
benchmark exclusion. A future canonical run must retain the actual enlarged
source results, including any new failure. Structural reconciliation alone
does not prove a hydrated 147-source runtime sweep.

The benchmark ledger row has no ecosystem/App membership and explicitly
unknown dialect coverage. Keep all 144 ecosystem project registrations,
142 unique ecosystem fixtures, 16 App fixtures, all oracle memberships and
capability evidence unchanged. Keep the 144 registered DOM-output comparisons,
16 old-lane hard-error skips, exact error-reason allowlist and instruction
ceilings unchanged. A new input that changes an outcome fails the existing
gate; do not relabel it as the old 146-source success.

The explicit unknown-fixture count becomes 132, while all 15 partial
classifications and their evidence remain unchanged. The first source's
tooling lane exposed its stale 131-count assertion; retain that failed run.

The historical 146-source DOM runs, file/template counts and Phase 2 records
remain historical. Only the current executable inventory summaries change.
The registration establishes no native migration, default-routing,
fix-history or performance acceptance.

## Published snapshot and current qualification

The pinned published report was generated at 2026-09-29T12:56:26.892Z from
`8a8848276c52956d7e54e262e5846e41fc922288`, workflow run `36570273148`.
It uses `vize` and `@vizejs/native` 0.429.1, Node 22.23.2 on Linux x64,
200 files, five runs and one warmup. Vize main at registration is
`d1a25ec1da2efca98534520ff8ebe84d34708e3d`, version 0.432.0.
The public 0.432.0 package and current source are separate tool identities;
neither has executed this new pinned benchmark in this change.

Independent read-only audits of the original published results identify:

- Lint: both Vize 1T/default cover 200/200 files, but all eleven mandatory
  validity plants fail diagnostic attribution. The two displayed notes are
  a truncated summary. Suite `2026-09-12.2` has hash
  `3a09ab6b2b314ce5be7d60bf17fa06941f895f25fe0e305adf88c0a1b0ee06a2`.
  A text-parser boundary is suspected; actual current machine output must
  distinguish upstream attribution defects from missing Vize diagnostics.
- Compiler: CSS passes 13/17; VDOM production passes 27/33, development
  30/33; Vapor passes 27/33 in both modes. Some Vapor reference cases also
  fail, so fixing Vize alone cannot establish their validity or a rank.
- Typecheck: eleven of 154 original plants have missing diagnostics.
  All eleven historical responses contain zero diagnostics; original
  expected vectors, full projects and settings remain authoritative.

These are historical audit counts, not current-source pass claims.
Compiler/CSS owners and typecheck owners qualify their actual corrections
independently; this metadata PR changes no production behavior.

The existing disposable `vue-benchmarks-replay` keeps its independent
`65c6102504b14cd49c0b03305be8dd0b9d208c59` pin, expectations and mutating
`ensureUpstream` helper unchanged. Never point that helper at this read-only
submodule. The completed older #3283 slice is not reopened.

## Required follow-through

- Qualify complete unchanged validators with authenticated current-source
  CLI/native outputs on Actions; retain exact source/tree/binary, lock,
  fixture, command, tool versions and full raw failures.
- Preserve every original validator, including all eleven lint plants,
  complete typecheck diagnostic vectors and compiler target/mode controls.
  An upstream adapter correction must preserve the same expected diagnostics.
- Carry necessary fixes through exact-head Actions, protected full queue
  checks, actual main and a verified product release. Keep #7856 open after
  this registration until its runtime and delivery acceptance is complete.
- Run comparable timing/RSS only after workload parity and correctness.
  No rank-label edit, skipped validator, lower expectation, synthetic
  composition or measured 10x claim is admitted by this registration.

## Automatic canonical qualification on the current Stack

The existing three-layer native Stack #7919 is #7860 → #7910 → #7918.
Its `d716`/`166238`/`3f2010` source checks remain historical. Literal-main
`4e4c8c977d7052edbb6a2b42808ddc6e7b59f3bf` contains the actual #7974
repair of the original seven DOM and seven SSR controls. It does not
transfer acceptance from the failed 147-source run `37256474108`.
That run's original 228-byte `v-pre` source, complete failure receipts and
unexecuted chained Pug command remain in the
[canonical divergence record](./2026-10-05-canonical-v-pre-divergence.md).

Ordinary PR and merge-group checks previously executed their smaller
default corpus; only scheduled/manual Real Project Matrix hydrated all
147 gitlinks. The necessary integration moves that exact canonical job
into a reusable workflow in a separate move commit. Its original job body
has SHA-256 `308c7e7753ceec8e73650ad4f62c8d0eb588a26052742425d12924455dd1801c`.
The Matrix caller preserves its mode, environment, full job commands,
checkout, pins, 120-minute envelope and every other job. Its default remains
`enforce`; original DOM run/finalizer/receipt/artifact production and
strict SSR → Pug chaining remain unchanged. No 22-shard campaign is added.

The ordinary source planner now selects the same reusable execution for
gitlink/fixture registration, producer and dependency inputs, benchmark
scripts, workflow/actions and the selection/comparison/receipt helpers.
Real Git's NUL-separated diff includes deletions and both rename sides.
Unknown comparison or an empty diff requires execution; malformed context
and paths refuse. There is no branch-name condition or manual dispatch.
The source report first requires an explicit successful plan and an actual
successful corpus result when selected, then delegates every original job
to the unchanged strict aggregator. An irrelevant skipped corpus is
accepted only with an explicit false selection; its failure still refuses.
No global skip allowlist or original queue-only WIT rule changes.

The replay retains every incoming canonical clause at 350 lines, all
original fixture/scorer/provider bytes, the 147 count, 144 comparisons,
16 original error-file skips and unchanged budgets. Semantically equal
flow YAML preserves the source workflow's 350-line ceiling. A first local
integration test exposed an accidental test selector and was corrected
without changing its expected WIT job. The complete focused selection,
strict aggregate, workflow, hydration/refusal and queue controls pass
32/32 under the available Node 25.8.1; this is local source evidence, not
configured hosted/native acceptance. Fresh exact-head Actions, actual
whole147 DOM/SSR/Pug evidence, all protected checks and signed merges of
the complete native Stack remain required. Keep #7856 and current ranking
unfinished; the fixed154 manual workflow may run only after actual
registration on literal main and publication coordination. Whole cold500
42.55 ms/10x remains unfinished.

## First automatic execution and Pug registration

The genuine automatic full147 runs on source `67f62a587d`
([37273858282](https://github.com/ubugeeei-prod/vize/actions/runs/37273858282)),
`7c70a816b6` ([37273856590](https://github.com/ubugeeei-prod/vize/actions/runs/37273856590))
and `edc88d82d4` ([37273855936](https://github.com/ubugeeei-prod/vize/actions/runs/37273855936))
all pass whole DOM and SSR: 42,998 files, 42,625 templates, 42,609 DOM
comparisons/16 original error skips, 42,625 SSR production comparisons and
42,606 SSR oracle comparisons/19 original legacy-error skips, with zero
refusals/rejections/divergences. All three fail the first actual chained Pug
execution because the benchmark's original Pug key lacks a baseline row;
production reach is skipped. Finalization and artifact upload still run.
These positive DOM/SSR results do not make the overall source gate pass.

The fixed revision's complete 330 SFCs contain exactly one inline Pug
template: [the unchanged 182-byte original](https://github.com/pikax/vue-benchmarks/blob/5489aee433cd1054b9d72973457498544da7c467/tests/confirm/fixtures/format/format-pug-template.vue),
Git blob `32527f29d4c219d9a22438cc1225258e0d005086`, SHA-256
`60a8a8cf7ee56f861a60adc95101f6d05faa69a49b2aee6f3f390360d3cd84f9`.
The existing pinned `pug@3.0.4` independently renders the original extracted
template with unchanged options:

```html
<div class="wrapper"><h1 class="title">CONFIRM_PUG_TITLE</h1><ul><li v-for="item in items" :key="item">{{ item }}</li></ul></div>
```

Its SHA-256 is `124e9a0eb552a1a1656ec89cac9ee20aa9b44c4f3a05f880de7c490c87d5eb81`.
Add only this revision/path/digest row; retain every old 498 row byte and
pin the exact 499-row census. The existing reference oracle separately
includes this read-only gitlink, checking complete 330-SFC/one-Pug scope and
exact original source bytes without adding ecosystem/App/capability
membership. No observed Vize digest or record mode chooses the baseline.
The independent reference test passes 2/2 using existing cached pinned
packages, with no installation; an initial missing compiler search path is
retained as a failed local receipt. Current Rust new-row outcome remains
unmeasured: fresh hosted fidelity/soundness/lowering and complete
DOM/SSR/Vapor result equality against the HTML twin are required, including
all old Pug cases. No normalizer, skip, provider option or budget changes.

Raw original job streams and complete reference template/HTML/official Vue
3.5.35 JavaScript DOM/SSR outputs remain under
`/tmp/vize-stack7919-current-main-review/pug-registration` and its parent.
`pinned-oracle.json` SHA-256 is
`9b08eb7510e72ac3fb5380c95e1f736e11c5416f0c1dbf244eebbc817aa4b619`.
The pinned archive is 5,997,798 bytes, SHA-256
`45c2db7e46640641f62b021cd0acced8feac468d681b7036f373987d78ed1793`;
GitHub's decoded original blob equals its whole archive source. These
JavaScript outputs do not prove current Rust outcomes. Fresh source,
whole147/Pug, protected checks and all three signed merges remain pending;
finite release admission is held. Ranking, fixed154 campaign and the cold500
42.55 ms/10x target remain unfinished.

### Extracted Rust action custody

Fresh bottom source `480626106e` preserves the complete original Matrix job,
but generated Zizmor check `111680122522` rejects the inherited Rust-action
commit at the new reusable path: `6bed0761…` has no history in the referenced
repository. Pin only that action to the actual official `stable` commit
[`89b12181…`](https://github.com/dtolnay/rust-toolchain/commit/89b12181fb390509a0842a86cc55eeb8eb928c1d).
The full corpus inputs, Rust stable role, steps, outcomes, budgets and artifact
contract stay unchanged. Two earlier child executions were cancelled during
the atomic parent/base update; strict aggregation correctly rejected their
missing plan. Those cancellations and the original security alert remain
historical, with fresh complete source/147/Pug checks required for every layer.
