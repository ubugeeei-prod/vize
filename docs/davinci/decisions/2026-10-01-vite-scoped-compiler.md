# Scoped Vite compiler settings

Issue: [#7247](https://github.com/ubugeeei-prod/vize/issues/7247).

Merge matching `entries[].compiler` in declaration order over the global compiler config, then apply explicit plugin overrides using the same option resolver as the global path. Resolve relative entry bases against the Vite project root, preserve ordered positive/negative patterns and ignores, support directory ancestors and hidden files, and retain compatibility subfield inheritance. Compile matchers once per resolved plugin configuration using the already locked Picomatch 4.0.5 implementation as an explicit dependency.

Pass the source filename through all request option paths: normal SFC, macro artifact, style fallback, post-transform, JSX transform, and HMR. Batch precompilation groups files by their actual native compiler options. Include entry scopes in the persistent cache identity so unchanged source text cannot restore compilation from an older config.

Register the authored SFC under `npm/builder/vite/src/test/fixtures/scoped-compiler.vue` in the package test entrypoint. Compare complete compilation output with independently selected native options for preserved and condensed components, development and production, fresh/cold/changed-scope caches, client/SSR loads, and HMR. Test the complete resolved compiler objects for bases, ordered overrides, negations, ignores, braces, literals, hidden files, outside-root paths, and explicit precedence. Candidate-native verification runs in GitHub Actions; local package wiring tests pass against the available installed addon.
