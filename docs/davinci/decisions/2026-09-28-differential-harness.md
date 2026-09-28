# Shared differential result contract (#6891)

The formatter CLI was the first registered executable adapter. A narrow
compiler SSR adapter is prepared below. The shared
`tests/differential/harness.mjs` seam reads a product's `vize.differential.manifest`
v1, binds the raw manifest SHA256, checks unique planned case IDs and target
coordinates, and validates the `vize.differential.result` v1 envelope. Each
product adapter owns its CLI/API invocation, immutable input and build
receipts, process observations, comparator and result accounting. The existing
formatter adapter now uses this seam without changing its byte comparison,
three-pass idempotence or unsupported native result.

One planned result row exists for every `(case ID, target)` pair. A single-target
adapter may omit `row.target` for compatibility with the existing formatter
report. Multi-target adapters must put the target on each row. Missing,
duplicate or unplanned coordinates fail validation; an unavailable executable
must still produce a failed legacy row for every planned coordinate. The
`summary.plannedCases` count is the number of authored cases; product-specific
target denominators count rows. `dialectCoverage` remains optional on each
case, with absent coverage treated as unknown by downstream reporting.

Native acceptance requires an observed, source/build-bound whole-product
transcript. A completed native row carries `provenance` with
`scope: "whole-product"`, the exact `sourceRevision`, a validated build receipt
SHA256 and exactly the adapter's required set of unique `contributions`. Each
contribution records `stage`, `implementation`, `factOrigin` and explicit
`fallback`. A missing, unknown, legacy or fallback contribution cannot count
as native handled. The product adapter must also verify its runtime
observation. Matching an old reference in the legacy lane never grants native
credit. The formatter reports native unsupported for all five current cases.

The compiler product gate in #6853 will initially report its Croquis SFC parse
contribution as legacy-backed. Other real product adapters, product-specific
comparators, T1/T2 complete execution, and native acceptance integration
remain open work under #6891 and #6853. This contract alone closes none of
those product gaps.

## Compiler SSR adapter preparation (#6891)

The first compiler manifest plans one `compiler/sfc/ssr-slot-scope` case at
target `ssr`. The authored SFC and full emitted module reference are copied
byte-for-byte from the existing `ssr_slot_scope.rs` regression at
`ecef0efd44c5047293170b6d42b5b16aacae6560`. Both SHA256 values are
committed. The adapter runs the source-built `vize build Layout.vue --format
json --output out --ssr --no-config`, retains raw process and JSON output,
compares emitted module code bytes to the pinned reference, and requires empty
CSS, errors, warnings and macro artifacts. Exact HEAD and executable SHA are
bound through the existing build receipt. A missing build still yields the
planned failed row.

This is a legacy regression observation only. Its native row is explicitly
`unsupported`, its paired comparison is `not-compared`, and its native
acceptance numerator is zero. A completed native row fails this adapter's
validator. Whole-product native compiler acceptance needs #6835 to remove
armature/relief from L1 Vue parsing and replace the Croquis SFC descriptor,
which remains a legacy fact. #6842 is required for Vue 0.x, 1.x and quirks
dialect coverage.
Extending this one SSR fixture to every compiler target, dialect and T1/T2
source is still open. The source-built CLI test belongs in Actions after the
build receipt step; until that run finishes, the reference match is unverified.
