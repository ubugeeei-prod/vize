# Current upstream typecheck validation

Tracking: [#7856](https://github.com/ubugeeei-prod/vize/issues/7856).
This prepares source qualification; actual current CLI execution is pending.
The [paired issue receipt](https://github.com/ubugeeei-prod/vize/issues/7856#issuecomment-5987128899)
records the same authority and execution boundaries.

## Original authority

Use the read-only `tests/_fixtures/_git/vue-benchmarks` gitlink at
`5489aee433cd1054b9d72973457498544da7c467`. Its 154 typecheck cases exactly
match the published case set. The existing Vize replay stays on its separate
`65c6102504b14cd49c0b03305be8dd0b9d208c59` revision and 150 cases.
Neither that adapter nor its expectation tables are changed here.

The September 29 snapshot reports Vize 0.429.1 passing 143/154 plants. Its
eleven failures have zero matching diagnostics; none is a reported code,
range or message-parser mismatch:

- `attrs-aria-data-unknown`
- `attrs-unknown-fallthrough`
- `inherit-attrs-default-unknown`
- `inherit-attrs-false-unknown`
- `unknown-prop-strict`
- `fallthrough-mono-false-bad`
- `fallthrough-multi-bad`
- `fallthrough-multi-false-bad`
- `fallthrough-vif-both-mono-false-bad`
- `fallthrough-vif-mono-multi-bad`
- `fallthrough-vif-static-multi-bad`

These historical results do not prove failures on current source. Current
strict-prop and fallthrough generation already exists; current production
diagnostics, all clean controls and the complete original judge must run
before identifying a remaining defect or changing production.

## Bounded current replay

The new `vue-benchmarks-current-typecheck` helpers import the original
`prepareAllPlants`, `scoreCombinedRun`, diagnostic parser and pin handling
from the fixed upstream. Only the upstream-created work copy is writable.
The copied tsconfig paths, package dependency and NODE_PATH retain upstream's
Vue package transport, changed solely to a dedicated Vue 3.5.43 installation. All 154
original metadata objects, pins, source bytes and parser/scorer dependencies
are archived and hashed. The submodule stays pristine before and after.

Require an explicit source-built Vize binary and native 7.0.2 executable;
there is no global or package binary fallback. Retain hosted checkout/build
logs to bind the binary hash to the exact source SHA. Version output alone
does not prove the build provenance. The dedicated npm provider installation
has a retained package-lock identity and complete file/link digest inventory.

Run six untimed native checks in one authored workspace: original default
text and JSON with complete virtual TS on one/two servers, each under the
original shared config and isolated fallthrough config. Compare complete
ordered one/two-server JSON reports without path/message normalization.
Retain clean-file coverage, every diagnostic string, compiler options and
program root/config membership. Those authored members are not a native
transitive dependency-graph closure proof.

The optional independent reference pins the upstream top-level vue-tsc 3.3.11
and TypeScript 6.0.3, with two text runs against the same configs and sources.
The upstream pnpm lock and actual dedicated npm lock are retained separately;
top-level version equality does not establish equal transitive installations.
Original plant expectations score each engine independently; Vize output
never becomes the oracle. Non-default fallthrough warnings remain warnings.
Every original stdout/stderr byte, status/signal and own process-error data
is retained before parsing or assertions. Failures cannot become clean runs.
The all-pass result additionally requires no warnings, skips or unattributed
diagnostics; a remaining gap exits unsuccessfully and stays unqualified.

Five owned Node-stub controls locally verify failed binary streams, malformed
JSON custody, ENOENT/null streams, signal termination, error causes/cycles/symbols/accessors and
fixture containment. Format/lint and syntax checks pass. They execute no
Vize/native/reference/provider command and supply no current CLI acceptance.

## Delivery boundaries

Use the genuine ordered native Stack: metadata #7860, lint #7910, then
this typecheck probe. This child is based on actual lint head
`16623802f55d30337f89ef67f5102ce22e79fb3a`, which includes metadata
`d7169333f84bb3d8ddfaebcc348b4b83930d6f40` as an ancestor.
Root review precedes the finite hosted execution. Exact-head Actions and
protected suites, actual signed merge and subsequent release qualification
remain necessary. Any actual defect needs its original corpus fixture and
paired issue/canonical record. This evidence has zero migration, timing or
CPU credit; the typecheck 10x target remains unfinished.

The dedicated manual workflow is registered by that actual merge before one
dispatch on exact current `main`. It checks the requested source SHA against
the event ref, workflow SHA, checkout and actual remote main. It builds the
real CLI with the locked Cargo graph and retains source/tree/workflow/ELF
identities. The provider prefix uses committed npm lock version 3 with all
35 dependency entries carrying exact resolved versions and registry SRI;
runtime installation uses `npm ci` without scripts or force. The lock was
resolved as metadata only on macOS, requiring a cross-platform lock-only
force for the Linux native package; no dependency was installed or executed
locally. Original upstream pnpm and actual prefix npm identities stay distinct.
The default native-phase 45-pair/CPU authorities and protected tooling jobs
are unchanged. Root review precedes the sole actual post-registration dispatch.

Fresh source Check `37257533302` rejected five unawaited Node registrations
and two descriptor getter/setter identity reads under type-aware lint. The
controls now await completion, and the serializer obtains those original
functions through their own data descriptors without binding or invoking
them. The failed original raw job log is retained; this successor needs fresh
exact-head Actions. Workflow, provider lock, original judge and runtime
conditions are unchanged, and no historical green or current CLI credit transfers.

The genuine replay onto repaired lint parent `16623802f5` preserves every
typecheck helper, workflow, provider manifest and frozen-lock byte from the
corrected child `749f9333da`. Both original authored commit messages and
verified reporter trailers are retained. Only the canonical record conflicted;
it is reconstructed from the exact incoming parent plus the owned typecheck
clause, preserving all lint schema-custody corrections. Prior source failures
stay historical; fresh source Actions and actual protected prefix delivery
are required before the sole main execution.
