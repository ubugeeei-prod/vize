# Typed template binding regressions

Original reproductions from Vize #7895 and #7902, including both #7895 follow-up
comments. The Vue/TS/config blocks are retained verbatim. Native type-aware unit
tests run the parent SFCs against these on-disk sibling imports and assert the
complete unsafe-binding diagnostic vector. Additive controls cover unsafe call
results, unsafe callees in callbacks, unknown assignments, unresolved component
imports and authored `any` component exports. Component dependency projection
uses the actual on-disk SFC sources; no ambient component shim is added.

PR Rust shards intentionally disable the typechecker runtime. A full Check or
protected merge run must execute these tests with `VIZE_TEST_REQUIRE_TSGO=1`;
source-only green is not proof of the runtime behavior.
