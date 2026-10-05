# Governing config directory for type libraries

Issue: [#7825](https://github.com/ubugeeei-prod/vize/issues/7825).

The CLI legitimately widens its common project root when a selected source
imports a sibling directory. The flattened virtual tsconfig previously stayed
at that common root, losing the governing config directory used by TypeScript
to resolve explicit `types` and implicit `node_modules/@types` libraries. A
missing library then produced a global TS2688 and masked real source errors.

Place the generated config beside the mirrored governing config directory.
Keep original `types` names, and reanchor existing mirror-relative `include`,
`paths`, `typeRoots` and output-source paths to the new location. Normal imports
retain their original directory chain; app packages are not hoisted into the
common source root. CLI execution, shards, declaration emission and cold/warm
generated-file ownership use the same config path. Config diagnostics remain
attributed to the authored config.

The original reporter's five input files are retained byte-for-byte in
`tests/_fixtures/differential/typechecker/outside-import-types/input.json`.
The independently authored expectation is the real TS2322 at `src/b.ts:3:14`.
Required source-built CLI tests compare all nonempty per-file diagnostic
vectors and direct official TypeScript 7.0.2 CLI output. Additional fixtures
check a scoped direct type package and its global declarations, and ensure
an app-only import stays unavailable to a sibling source. A unit test preserves
all transformed path values and original type-library names.

Fresh exact-head Actions, unchanged instruction ceilings, protected full suites,
actual merge and published consumer verification are required. The release
owner's publication hold applies until its first-cut qualification finishes.
This repairs the existing product without native-stage acceptance credit.

Independent review of b738 found the option-probe disk reader still used the old
root path and normal PR Rust runs disable native execution. The correction
updates that authoritative reader, adds a nested unsanitized-config no-probe
law, and requires the new source CLI tests in the existing native-phase job.
Complete raw stdout/stderr/status, all input files, official package/version,
binary hashes and source identity are retained as Actions artifacts. Earlier
PR green is insufficient; fresh corrected-head runtime acceptance is required.

The correction review also moves the new no-probe law inside its existing
`cfg(test)` module; its complete nested input and no-probe assertion stay
unchanged. Fresh strict source checks and native capture are still pending.

The first required runtime execution passed the exact original normal/sharded
and direct-scoped cases. Its sibling control reports both authored errors but
orders the app diagnostic before the imported source, so the full native stdout
expectation now follows that ordering without changing any code, input or
diagnostic. Refresh the generated consumer surface inventory for the added
source module; require fresh complete three-case execution before acceptance.

The second native run reaches the sibling CLI comparison and matches both full
diagnostic vectors. The CLI's established convention renders files outside the
governing app root as absolute paths, so assert the exact authored sibling path
from the isolated case root. Native stdout remains config-relative. No runtime
source, input, diagnostic or output convention is changed by this test repair.

At 3a114 the three required CLI/native vectors pass, but current strict Rust
CI exposes an existing editor test reading the old mirror-root config path.
Use its actual nested `packages/web/tsconfig.json` and assert the internal
project carries that precise path. The native Vue/script/check-server session
previously reconstructed a root config path too; pass the already-generated
config explicitly while preserving the common mirror root and public URI
ownership. No public document layout changes. Required Actions now execute
the exact five-file original through the native script editor with one fully
authored TS2322 response, and require an entirely clean native monorepo alias
control. Capture full editor output, all original bytes and the same official
binary identity; fresh source and runtime gates replace the red 3a head.

At cfd208 all three full CLI/native cases and the original five-file native
script-editor single-TS2322 response pass. The raw monorepo alias response
retains four severity-4 TS6196 suggestions on generated, unused type aliases,
so the previous wholly empty raw expectation was incorrect. Assert every
field of the explicit complete four-suggestion response, retaining their
generated source/config ownership and all original inputs. No diagnostics
are filtered, and any resolution error or extra suggestion still fails.
Refresh current-head source/native receipts before readiness.
