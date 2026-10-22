# Compiler fix-history Vapor v-once slice — #6880

The compiler fix-history review ledger now admits its first Vapor fixture while
retaining the ten SFC and five SSR reviews. This bounded slice reviews the
already registered `compiler/vapor/once-directive` fixture for fix
`866be7b96ed336704bd1fd3a2f3dc2ad6e699151`.

The authored `<div v-once>{{ msg }}</div>` input, default public Vapor options,
repeated historical observation, complete expected result, and fix provenance
were already pinned. The review records that `v-once` lowers without a runtime
directive wrapper or render effect while retaining the initial text update. It
also pins the complete public Vapor result: generated code, static templates,
nullable map, and ordered error messages. The witness path and file hash are
normalized to the current `src/tests.rs` location. No product source or fixture
result byte is changed.

This advances bounded semantic review from fifteen to sixteen fixture scopes.
It does not complete the fix across `v-memo`, nested or control-flow shapes,
alternate options or dialects, SFC integration, mounted caching behavior, or
native compilation. All pinned fix subjects retain whole-fix target and
dialect review pending, the semantic denominator remains unset, native
acceptance remains zero, and #6880 stays open.
