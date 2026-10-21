# Compiler fix-history SSR component lone-spread slice — #6880

The compiler fix-history review ledger now admits its third SSR fixture while
retaining the ten existing SFC reviews. This bounded slice reviews the already
registered `compiler/ssr/component-lone-spread` fixture for fix
`7c5c7282989291cf92c3e3546a5c56e24eeb92d5`.

The authored `<Icon v-bind="iconProps" />` input, default public SSR options,
repeated historical observation, complete expected result, fix provenance, and
current source witness were already pinned. The review records the requirement
that an object spread remains in the emitted component prop bag when it is the
only authored prop entry. It also pins the complete public SSR result:
generated code, helper preamble, nullable map, and ordered diagnostics. No
product source or fixture byte is changed.

This advances bounded semantic review from twelve to thirteen fixture scopes.
It does not complete the fix across object `v-on`, mixed or repeated props,
control-flow nesting, alternate options or dialects, SFC SSR integration,
mounted behavior, or native compilation. All pinned fix subjects retain
whole-fix target and dialect review pending, the semantic denominator remains
unset, native acceptance remains zero, and #6880 stays open.
