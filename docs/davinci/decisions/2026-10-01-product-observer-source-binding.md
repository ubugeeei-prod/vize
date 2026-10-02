# Source-built product observers

Issues: [#6891](https://github.com/ubugeeei-prod/vize/issues/6891) and
[#6881](https://github.com/ubugeeei-prod/vize/issues/6881).

The shared `tests/differential/harness.mjs` manifest/result envelope remains
the product corpus contract. Public API histories need an observer when the
CLI does not expose the original API/options or complete return value.

`tests/differential/observer-build.ts` factors the successful Cargo artifact
selection and executable capture out of the existing formatter helper.
The formatter retains its existing receipt schema, option probes and complete
output contract. Compiler and linter adapters use the reusable mechanism;
there is no additional production stage or normal dependency.

Each observer specification fixes product, package, example, source path,
expected default features and concrete option/contract probes. The helper
requires committed Rust/Cargo inputs, including upstream level sources and
test support, and rejects untracked Rust input. It records actual revision,
tree, product source tree, lock and observer hashes. A successful Cargo
`build-finished` and exactly one correct package/example/source artifact are
required. Failed builds cannot reuse an existing executable. The frozen
executable, Cargo output/error logs, target/profile/features, toolchain and
actual probe stdout are bound in the receipt. Receipt validation checks the
raw Cargo artifact and runs each probe again. Windows captures retain `.exe`.

The existing composite upload action retains the complete `target/differential/`
tree and selected source-built CLI receipt after failed or successful tooling
tests. Its artifact prefix remains compatible; run, attempt, job and shard
coordinates prevent collisions without another build or product-specific list.

The concrete compiler/linter consumers must still verify complete actual
observations against immutable source-bound expectations, repeat observations
without normalization, retain every planned case/target coordinate, and fail
on missing outputs or process errors. A build receipt alone accepts no case.
Unavailable whole-product native paths remain `unsupported`; comparisons
remain `not-compared`, with zero native handled/equivalent credit.

Focused tooling laws exercise dirty upstream/support inputs, untracked Rust,
failed or missing build completion, duplicate artifacts, package/source/feature
drift and recipe/profile restrictions. Actual builds and registered corpus
execution remain in the full Actions tier. Full product fix-history review,
native adapters and native parity remain unfinished; these mechanisms do not
close #6881 or permit a legacy route replacement.
