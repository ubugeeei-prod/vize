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
