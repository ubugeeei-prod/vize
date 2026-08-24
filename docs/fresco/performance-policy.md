# Fresco Performance And Size Policy

This policy records the M0 performance and package-size contract for
[#3113](https://github.com/ubugeeei-prod/vize/issues/3113). It covers
`vize_fresco`, `@vizejs/fresco-native`, and `@vizejs/fresco`.

## Measurement Surfaces

Rust renderer and terminal behavior are measured by the crate Criterion benches:

```sh
cargo bench -p vize_fresco --bench render
cargo bench -p vize_fresco --bench capabilities
```

The `render` bench is the regression surface for buffer writes, Unicode text
width, wrapping, layout, diffing, virtual lists, diagnostic workspace state,
headless snapshots, and terminal output telemetry. The `capabilities` bench is
the regression surface for terminal capability resolution and style fallback.

The JavaScript package size surface is the published package, not the source
tree:

```sh
vp run --filter './npm/fresco' build
npm pack --json --pack-destination "$TMPDIR" npm/fresco
```

`@vizejs/fresco` publishes only `dist`, so size evidence must report the packed
tarball bytes and the unpacked `dist` bytes. Changes to `exports`,
dependencies, build output, optional feature entry points, or bundled examples
must include that before/after evidence in the pull request body.

Native binding size is tracked separately from the Vue package. Changes to
`@vizejs/fresco-native`, N-API declarations, or the `napi` feature must report
the produced platform binary names and sizes from the relevant release or local
native build artifact.

## CI Gates

Pull requests that touch Fresco runtime, renderer, package, or native binding
surfaces rely on these shared checks:

- `cargo-semver-checks (vize_fresco)` for public Rust API compatibility.
- `clippy-and-test` for Rust correctness and crate tests.
- `criterion-ab` for impacted Criterion benchmark comparisons.
- `build-js-packages` for Fresco native declaration checks, native package
  build readiness, and JS package build output.
- `test-js-packages` for `@vizejs/fresco` behavior and type-contract tests.
- `check-js` for TypeScript and package-level static checks.

Renderer benchmark regressions are blocking when the benchmark gate reports a
deterministic regression. When a benchmark is intentionally slower, the PR must
name the affected bench, explain the user-visible tradeoff, and include the
before/after output accepted by review.

Package size is record-only until a checked-in size baseline lands. Until then,
size-increasing changes are allowed only with explicit packed and unpacked
before/after numbers plus the reason the new bytes belong in the core package
rather than an optional provider.

## Review Checklist

For each Fresco performance-sensitive pull request:

- Cite the exact measurement command or CI check that covers the changed path.
- Record package-size evidence when public package contents or dependencies
  change.
- Keep optional rich display integrations out of the core package unless the PR
  also updates this policy with a new size budget.
- Do not mark a #3113 performance or size item complete from local numbers
  alone; the PR must reach green CI with the relevant benchmark and package
  checks visible.
