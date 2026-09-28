# #6852 — legacy fix differential inputs

A legacy behavior fix must add its input to a registered product differential
corpus. A product regression test or snapshot alone does not establish that the
Davinci lane sees the same input. The existing T1 Rust feature tests cover
selected compiler and markup surfaces; the shared `#6891` manifest/result
contract currently has an executable legacy formatter adapter only. It has no
verified native formatter adapter and does not yet cover all five products.

The PR source planner now reports on conventional `fix(...)` PRs that change
known legacy Rust source paths. It identifies compiler, typechecker, linter,
formatter and LSP ownership, compares each product manifest against the PR
comparison base, and checks that a newly registered active case pins actual
input bytes by SHA-256. Missing manifests and missing or invalid inputs are
reported explicitly. The report grants no parity or native acceptance. It is
non-blocking while existing legacy fixes are in flight and product adapters are
incomplete. Other change types and JS product source paths are not classified
yet; this is an audit, not the finished requirement.

Before making this a required T1 gate:

1. Register executable legacy and native adapters for compiler, typechecker,
   linter, formatter and LSP in the shared `#6891` contract. Each must produce
   complete planned case/target rows and compare the product's full public
   output with the promised byte or diagnostic/response comparator.
2. Bind each added input and result to exact manifest, source build and adapter
   receipts. Missing, unsupported, fallback and legacy-backed native results
   fail the comparison; none receive native-only credit under `#6853`.
3. Cover the outstanding legacy fix PRs with product-registered inputs, then
   prove on a real PR and protected merge group that missing input and
   divergent output both fail while a complete fix passes.
4. Extend ownership mapping to JS product sources and non-`fix(...)` behavior
   changes, then remove `continue-on-error` and keep the full differential
   execution in the required queue report.

The issue stays open until these conditions are met. Legacy bug fixes are owned
by the separate bug agent; this work only supplies their future fixture policy.
