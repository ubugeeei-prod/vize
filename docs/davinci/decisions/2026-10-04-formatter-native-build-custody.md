# Source-built public formatter addon custody (#6882 / #6830)

The public NAPI formatter history needs evidence that the addon actually came
from the current source build. A local module hash and a live package-test
preparation owner alone cannot prove Cargo emitted those bytes.

## Decision

Capture the existing `build:native:test` Linux Actions debug build, preserving
its complete command, exit/signal/error, stdout and stderr before any admission.
The pinned NAPI CLI passes `--message-format=json-render-diagnostics --locked`
to that same Cargo build. Require exactly one successful `build-finished` and
one genuine `vize_vitrine` cdylib emission with `napi,legacy`, the actual current
crate source path and one shared-library filename. Its bytes must equal the
fresh generated addon. Freeze the same bytes for later independent observers.

Bind the receipt to clean committed HEAD/tree, source subtrees, Cargo/PNPM locks,
build environment, actual Rust/Cargo/Node versions and installed NAPI CLI bytes.
Validation rechecks source, toolchain, full streams, emitted and frozen bytes.
Reject untracked relevant source, stale/dirty receipts, ambiguous generated
addons and substituted artifacts. This uses the existing mandatory JS-package
build and preserves non-Linux, local and release behavior. No additional build,
pipeline stage or level serialization is introduced.

Upload the process streams, frozen module and whole build receipt even when the
existing hosted build/test job fails. Two admission laws require typed complete
Cargo success and preserve failed process frames; their synthetic rows grant
zero execution or native credit.

## Delivery and remaining work

The recovered source was preserved separately and replayed onto actual main
`ead89a7592e653a8ed585f4f3fed88fac71b3551`, retaining incoming workflow, native
providers, immutable formatter manifests and goldens. No local Rust build or
package installation was run. Actual source-built hosted build, exact-head
Actions, unchanged protected suites/instruction ceilings and actual merge remain
required. Failed campaigns remain scoped to their original heads.

A dependent nine-case pack will require twenty-five independent actual public
JS calls, whole input/options/results/errors and fixed-point chains while the
existing preparation owner is alive. That pack is not yet accepted. Public Vite
resolved-configuration/real CLI integration, schema/WASM history, applicable
Glyph measurements, genuine printer-error runtime and #6882 remain unfinished.
No native handled/equivalent/paired or default replacement credit is granted.

The first source2493203d1 campaign failed the zero-warning JS gate because
the two Node test registrations returned unconsumed promises. Mark their
registration results with `void`, preserving both callbacks, complete vectors,
assertions and production build custody. No warning waiver or runtime credit
is added; corrected-source Actions and protected acceptance remain required.

The first child244fbce25 hosted campaign genuinely emitted and froze the addon
(artifact11283519449), but stopped before public calls: the build used twelve
Cargo jobs while the later JS observer legitimately had no Cargo jobs variable.
Build environment is its own actor's custody. Capture and require identical
before/after values in the actual builder, while observers retain their separate
actual environment. Recheck unchanged source, full streams, toolchain and exact
emitted/generated/frozen bytes across actors; do not infer runtime Cargo settings
from the earlier build. A new admission law rejects within-builder mutation and
malformed/unknown fields. The failed original packet remains source-scoped and
grants no public-call acceptance. Fresh both-head Actions remain required.

The repaired parent `9c76c3fb` source campaign preserved builder/observer custody
but failed the unchanged source-length gate: `pr-source-checks.yml` grew from
348 to 356 lines. Move only its existing JS package command and always-upload
history step into the local `test-js-packages-with-history` composite, restoring
the owning workflow to 348 lines. The selected-JS invocation remains `always()`
so prior build failures can upload raw evidence; tests additionally require the
outer `job.status` to be success, and upload remains unconditional within the
invoked composite. Runner 2.337.0 scopes composite `success()` to action status
([official implementation](https://github.com/actions/runner/blob/v2.337.0/src/Runner.Worker/Expressions/SuccessFunction.cs)),
so that function alone cannot retain the prior outer-job success guard.

The source witnesses follow the real local action and retain the entire original
JS command, declaration/type/UI tail, selection, upload settings and fail-closed
required aggregates. No build, job, gate, instruction budget or default path is
added or removed. The failed original log stays source-qualified; new parent
and child Actions plus protected candidate execution and actual merge remain
required. Earlier child `6cabd2b2`'s 25 genuine calls qualify only that source.

## Protected replay output backpressure

Parent `b417f772` passed its exact-source Actions, but protected candidate
`0b5014cf88422ae49bc7ac60f4142f1dfbfeca34` failed Check37153620863's
check-js job111292373083. Cargo reported successful compilation in 52.55s;
the wrapper then truncated its captured JSON replay mid-record and reported
`Failed to spawn process: Resource temporarily unavailable (os error 11)`.
The owner removed the known-red entry while preserving healthy peer candidates.
Its original complete failed log remains retained. Its actual all-100 three-run,
ceiling/ratchet/hold success and artifact11284693314 qualify only that failed
candidate and supply no actual merge or successor acceptance.

The pinned installed Vite+ 0.1.24 references Vite Task `5833b37`. Its
[pipe drain](https://github.com/voidzero-dev/vite-task/blob/5833b37/crates/vite_task/src/session/execute/pipe.rs)
uses synchronous write_all/flush; the
[execution wrapper](https://github.com/voidzero-dev/vite-task/blob/5833b37/crates/vite_task/src/session/execute/mod.rs)
maps drain failures to the same Spawn message and cancels the child. This
supports an output-backpressure diagnosis; there was no PID/thread measurement.
The upstream [corrected diagnosis](https://github.com/voidzero-dev/vite-task/issues/506#issuecomment-5021666986)
and [merged blocking-stdio repair](https://github.com/voidzero-dev/vite-plus/pull/2173)
identify Node's shared nonblocking output descriptors as the cause of this
failure under slow consumers.

Change only Check's existing debug invocation to
`vp run --no-cache --filter './npm/native' build:debug`, retaining its entire
build/capture script and following `vp run --workspace-root check:ci`. The pinned
interleaved reporter inherits stdio for uncached spawned scripts, bypassing the
defective cached drain and ensuring a real fresh measurement build. No dependency
upgrade, resource/performance cap, additional build or gate waiver is introduced.

A pure Node law invokes the actual installed catalog-matching VP with script
caching enabled in an isolated workspace and the explicit uncached command.
Paused/slow consumers verify all 1,212,416 stdout and 671,744 stderr bytes,
including genuine reporter framing, for both exit0 and exit1. Both invocations
must actually run once; truncation, substituted output, failure masking and
unexpected termination fail the law. It runs no Cargo or package installation
and grants no native formatter credit. Fresh parent/child source Actions,
source-built whole packets, protected full suites/all-100 and actual merge
remain required; the old candidate is never reused as proof.
