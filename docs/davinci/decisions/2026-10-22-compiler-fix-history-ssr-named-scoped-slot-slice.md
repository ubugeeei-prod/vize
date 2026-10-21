# Compiler fix-history SSR named scoped-slot slice — #6880

The compiler fix-history review ledger now admits its fourth SSR fixture while
retaining the ten existing SFC reviews. This bounded slice reviews the already
registered `compiler/ssr/named-scoped-slot` fixture for fix
`49c9bcc2d19e1a1abfc2bcd0a372e16339a41ff6`.

The authored nested `#header="{ collapsed }"` and
`#default="{ collapsed }"` input, default public SSR options, repeated
historical observation, complete expected result, fix provenance, and source
witness were already pinned. The witness path is updated from the former
`lib.rs` location to its current `component_tests.rs` module; the symbol and
its authored assertions are unchanged. The review records the requirement
that both named scoped-slot entries retain their destructured prop in the SSR
string and vnode fallback branches instead of resolving `collapsed` through
the component instance. It also pins the complete public SSR result: generated
code, helper preamble, nullable map, and ordered diagnostics. No output is
regenerated or inferred.

This advances bounded semantic review from thirteen to fourteen fixture
scopes. It does not complete the fix across dynamic or conditional slots,
other destructuring shapes, alternate options or dialects, SFC SSR integration,
hydration, or native compilation. All pinned fix subjects retain whole-fix
target and dialect review pending, the semantic denominator remains unset,
native acceptance remains zero, and #6880 stays open.
