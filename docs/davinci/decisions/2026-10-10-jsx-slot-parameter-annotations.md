# Native JSX slot callback parameter types

Issue: [#8419](https://github.com/ubugeeei-prod/vize/issues/8419).
Baseline: signed main `e5702be6d0119a716499827fa7b918ce8c9dcbae`.

With JSX typechecking enabled, native lowering retained only the callback's
binding-pattern span. A required `{ value: number }` slot therefore accepted
an explicitly annotated `{ value: string }` callback. The unchanged inferred
slot public-contract test also exposed this as an unused `@ts-expect-error`
when the proposed config defaults enabled JSX.

The single existing OXC parse retains both binding and full FormalParameter
spans. Native typecheck lowering and its existing editor recovery select the
formal span; compiler consumers retain the binding span. The generated callback
opener has a one-byte mapping to the actual callback AST start. Fine parameter
and body mappings retain their coordinates and reverse-lookup preference. No new parse stage,
serialized intermediate, synthetic type heuristic, or backend substitute is
introduced.

## Qualification

- Before the repair, the native span law fails with actual `props` instead of
  authored `props: { value: string }`; after the repair it retains exact UTF-8
  coordinates for named, computed and render-prop slots, including arrow,
  function, destructured, optional and default parameters.
- Pre-fix complete VDOM/Vapor module code, component code, preambles, setup
  metadata, scoped styles, template arrays, diagnostics and source maps are
  frozen in differential snapshots. The repaired compiler must match them.
- Authenticated public 0.439.0 CLI probes retain all raw stdout/stderr, exact
  process outcomes and the original native loader's successful return with
  automatically selected public Corsa 7.0.2. Incompatible arrow, function and
  destructured callbacks incorrectly pass; correct and untyped callbacks
  pass, while an independent number-body error still reports TS2339.
- Source CLI acceptance uses complete authored diagnostic packets and the
  unchanged inferred-slot contract with explicit JSX enabled. Source Actions,
  merge queue acceptance, actual merge and a public release remain required.

Broader JSX contracts, multiple/rest slot parameters and generic callback
signatures remain unfinished under #1497. This slice does not qualify the
proposed default switch by itself or close that broader issue.
