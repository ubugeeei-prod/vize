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
