# Pinned L4 runtime vocabulary and module imports

Issue: [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

L4 owns compiler-use helper tables for the workspace's selected Vue 3.5.35
DOM/SSR release and Vue 3.6.0-rc.9 Vapor release. Runtime versions are explicit:
`vocabulary_for` returns no table for an unaudited pair. The default provider
uses these pinned releases, independently of markup dialect or language grammar
admission. Vue 0/1/2, other Vue 3 releases and full SFC-generated-script helper
coverage remain unfinished. No package version or lockfile changes.

Helper indices are local to one vocabulary. Existing core/SSR name order is
retained except the non-callable, non-compat `resolveFilter` slot. New script
helpers append after the template helpers. Targets obtain checked indices from
the selected table; the writer still records one u128 mask and first-use order.
Vapor rc.9 has no `prepend` export and uses `withVaporModifiers` and
`withVaporKeys` for its compiler event guards. No legacy generator name is
silently treated as a current callable helper.

`ModuleParts::for_runtime` consumes the provider before fragments are appended.
Every prepared fragment must use that vocabulary. SSR names come first from
`@vue/server-renderer`, followed by shared/script names from `vue`; each import
group retains final-body first-use order. Repeated uses import once. Custom
vocabularies retain their single-module behavior and can now describe several
module groups. Preambles are assembled last and preserve the whole-module span
links. This is neutral assembly, not a compiler product-route switch or a claim
of legacy target import-ranking parity.

Reference exports are the exact-tag upstream
[stable core](https://github.com/vuejs/core/blob/v3.5.35/packages/runtime-core/src/index.ts),
[stable DOM](https://github.com/vuejs/core/blob/v3.5.35/packages/runtime-dom/src/index.ts),
[stable SSR](https://github.com/vuejs/core/blob/v3.5.35/packages/server-renderer/src/internal.ts)
and [rc.9 Vapor](https://github.com/vuejs/core/blob/v3.6.0-rc.9/packages/runtime-vapor/src/index.ts).
A focused tooling law checks all table names against the real selected installed
packages, including the rc.9 dual-renderer browser build. It rejects missing,
undefined and compat-only null exports and version drift.

Seven new Rust laws cover all export/index round trips, exact-version rejection,
helper-mask boundaries, grouped imports, duplicate uses, recording-byte parity
and authored link shifts. Together with the unchanged writer/module laws, all
42 pass on byte-exact source modules with real cached L0/serde_json libraries;
those modules also pass Clippy with every workspace deny lint and warnings denied.
The 57 stable DOM and 19 SSR exports pass against the exact installed 3.5.35
packages. Rc.9 installed-package execution and complete workspace validation
remain Actions work; upstream-tag inspection is not substituted for that proof.

L4's skeleton ratchet drops from six modules to five. Expression rewriting and
DOM/SSR/Vapor/TS target emission remain unfinished; #6840 stays open. Compiler
route replacement still requires [#6880](https://github.com/ubugeeei-prod/vize/issues/6880)
and native parity. No pipeline stage, parse, serialization, normal/build legacy
dependency, production route, fixture output or numeric budget changes.
