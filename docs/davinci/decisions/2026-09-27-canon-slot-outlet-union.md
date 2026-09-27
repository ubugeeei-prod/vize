# Canon inferred slot payload carrier

Tracked in [#6922](https://github.com/ubugeeei-prod/vize/pull/6922) and the
typechecker fixture history [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

- Preserve normal object-literal string widening for static attributes on all
  inferred outlets, including repeated names. Literal discriminants require
  explicit authored bound expressions such as `:kind="'first' as const"`;
  declared checking literals are unchanged.
- Only repeated static names, or multiple outlets whose dynamic names may
  overlap, activate the union carrier. Other components keep existing emitted
  text without new aliases. Collection computes this flag once per component.
- Generated outlet maps form a union directly. Each lexical child scope returns
  the same carrier, and `__VizeInferredSlots` gathers every payload for each name
  into one callable signature. There is no fixed overload sampling bound.
  Missing/optional payload keys stay optional; all common required keys retain
  their value union. Single payloads retain their original object type.
- Different names return an intersection of one-key slot objects. A flat mapped
  object fails the upstream strict `IfEquals` oracle even when assignable in both
  directions. The one-key intersection preserves that existing public contract.
- External authored overloaded slots retain the exact main-line parent helper:
  inference uses the last signature. TODO: design a separate external authored
  overload contract; this repair does not introduce or promise a sixteen limit.
- Keep owner history through `38abc3820bce8d663e5f83672efef24d30746a34`
  as actual merge ancestry, including its correction removing all implicit
  static `as const`. The old preparation branch at `e7042ac05` remains preserved.
- Restore three Canon snapshots and the owner's static-slot-name Maestro snapshot
  to original main bytes. Their authored inputs contain no `<slot>` outlets,
  so the new inferred helper is not emitted; the external helper is exactly the
  main source again. No new snapshot hashes are invented. Fresh Rust snapshot
  execution remains required in Actions.
- Test registration moves remain separate commits; source budgets are unchanged.
  The [registered CLI corpus](../../../tests/fixtures/typechecker/slot-outlet-union/README.md)
  preserves the original four inputs byte-exactly and adds seventeen/eighteen
  positive controls, first/last negative controls, optional-key, single and strict repeated-static string checks.
- The new T1 tooling test validates the existing source-build receipt against the
  exact HEAD, binary digest, version and CI recipe. It compiles the actual CLI's
  virtual documents with TypeScript and pins complete normalized diagnostic
  file/line/column/code/message records for all eight added negative controls.
  Its explicit runtime catalog entry defers T0 and preserves required full T1.
- Required Rust CLI runs must find real Vue (root/tests/playground/examples/Nuxt
  plus its `@vue` namespace); missing Vue fails when TSGO is required. Only an
  explicitly optional local run may skip. The original root-only discovery could
  pass in 0.00s without executing the oracle, so that result earns no runtime proof.
- Local TypeScript 6.0.3 carrier-fragment probes accept seventeen/eighteen and
  nested first/last cases, preserve the upstream single-slot strict comparison,
  and reject every wrong first/last value, bad discriminant and optional boolean.
  These are semantic prototypes using emitted helper bytes, not a fresh binary
  run. The full added diagnostic oracle records its observed scope honestly.
- Fresh Actions must validate actual source-built CLI/TSGO execution, the three
  original Vue diagnostics' complete position/message/code, generated TS,
  snapshots and the full queue. Capture raw observations under
  `target/differential/slot-outlet-union.json`. The T1 test also writes exact build
  identity, process status and the full normalized diagnostic vector to the
  `VIZE_SLOT_CLI_OBSERVATION` Actions log before checking any oracle or count.
  A failed parse or assertion additionally logs raw stdout/stderr as base64 under
  `VIZE_SLOT_CLI_RAW`; no assertion or receipt check is relaxed. Finish the original three complete
  Vue oracle records from that observation before merge. Native typechecker
  comparison is unavailable; no native acceptance is claimed. No local Cargo
  build is run under the disk constraint. Issue evidence accompanies publication.

## Installed Vue SFC reference observation

The official installed `vue-tsc` 3.3.11 (`gitHead`
`f521b39b402ebaaa9ee70faf5a60a895544ed9e9`), TypeScript 6.0.3 and Vue
3.6.0-beta.10 were executed in isolated temporary projects. Strict upstream
`IfEquals`/`exactType` controls accept `string` for both single and repeated
static string attributes and reject literal-union expectations. Full slot
functions preserve all seventeen/eighteen explicitly bound literal overloads;
first and last calls pass and wrong kind/value calls report full `TS2769`
diagnostics. `Parameters` and parent-template binding still observe the last
overload. The direct generated carrier addresses this limitation; it does not
claim exact overloaded-function shape parity with that reference.

The original four Options API inputs produce no diagnostics in this installed
Vue toolchain and their slot payloads are `any`. This supplies no byte-exact
original diagnostic parity credit. A script-setup equivalent reports the side
`string` to `'pc' | 'sp'` error plus missing keys and bad forwarded number, but
is a separate observation. The unchanged original four inputs remain registered
in the required source-built CLI oracle. The CLI's expected three original
identities include `Parent.vue` lines 4 and 7 and `Wrong.vue` line 4; their full
records must come from fresh candidate Actions, rather than guessed messages.
