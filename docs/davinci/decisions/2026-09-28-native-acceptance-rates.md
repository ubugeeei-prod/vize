# Native-only acceptance rate publication (#6853)

The shared result envelope fixes every planned `(case ID, target)` coordinate.
The acceptance reporter uses those coordinates as its denominator, with one
count per target and one total per product. It validates the manifest digest and
expected source revision before counting. The machine-readable artifact is
`vize.differential.acceptance` v1 with `scope: "registered-fixtures"`,
`plannedCases`, `total`, and `targets`. Each target has `planned`,
`nativeHandled`, `nativeEquivalent`, `unsupported`, `legacyBacked`, and
`unverified`. A failed, skipped or unverifiable run remains in `planned` and
counts as `unverified`. Only the shared whole-product native provenance
classifier may increase `nativeHandled`; each adapter supplies its exact
required stages and verifies the build receipt and observation. Equality needs
an independently verified product comparison in addition to native handling.

The formatter CLI's five planned `fmt` inputs yield **0/5 native-only**, five
unsupported, and zero equivalent. The exact-head Actions execution publishes
the JSON at `target/differential/native-acceptance.json` and a table in the job
summary. The source-built linter observer now publishes its independently
validated report and acceptance artifact under
`target/differential/linter-api/`: **10/59 native-only**, all ten equivalent,
49 explicitly unsupported, and zero legacy-backed or unverified rows. Its
Actions summary uses the same shared reporter and denominator.

These rates measure registered fixtures only. They do not certify complete
dialect, T1/T2, or product coverage. Compiler has a separate fixture-scope gate
in [#7080](https://github.com/ubugeeei-prod/vize/pull/7080), with Croquis-backed
compiles credited zero. Type checker and LSP adapters also exercise the shared
reporter, but do not yet contribute a combined per-product publication; no rate
is fabricated for unregistered coverage.

Next: bring the complete dialect/input plans into the shared manifests; require
source/build-bound runtime observations and product comparators for every
product; publish one combined per-product table across the complete T1/T2
corpus. #6853 remains open until that work is done.
