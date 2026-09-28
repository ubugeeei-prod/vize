# Shared differential result contract (#6891)

The formatter CLI remains the only registered executable adapter. The shared
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
