# Compiler fix-history SSR component slot-props slice — #6880

The compiler fix-history review ledger now admits its second SSR target fixture
while retaining the ten existing SFC reviews and the first SSR review. This
bounded slice reviews the already registered
`compiler/ssr/component-slot-props` fixture for fix
`4819858579748608f8a3837b62ee6a2fb638d11e`.

The authored component-level `v-slot="{ item }"` input, default public SSR
options, repeated historical observation, complete expected result, fix
provenance, and source witness were already pinned. The witness path is updated
from the former `lib.rs` location to its current `component_tests.rs` module;
the symbol and its authored assertion are unchanged. The review records the
requirement that the destructured slot prop remains bound in both the SSR string
branch and the vnode fallback branch. It also pins the complete public SSR
result: generated code, helper preamble, nullable map, and ordered diagnostics.
No output is regenerated or inferred.

This advances bounded semantic review from eleven to twelve fixture scopes. It
does not complete the fix across named or dynamic slots, nested destructuring,
alternate options or dialects, SFC SSR integration, hydration, or native
compilation. All pinned fix subjects retain whole-fix target and dialect review
pending, the semantic denominator remains unset, native acceptance remains
zero, and #6880 stays open.
