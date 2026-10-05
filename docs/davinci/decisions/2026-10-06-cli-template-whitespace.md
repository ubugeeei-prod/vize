# CLI template whitespace

Issue: [#7880](https://github.com/ubugeeei-prod/vize/issues/7880).

## Decision

The CLI projects `compiler.whitespace` together with `compiler.templateSyntax` from the existing single raw-config read/evaluation. `preserve`, `condense`, and `vue2-line-breaks` select the existing thread-local scoped parser modes around each actual per-file SFC compile. Both ordinary output/capture and stats compilation use that scope inside their worker. The existing guard restores mode on return or unwind. No public SFC options field or pipeline stage is added.

The private raw document retains the whitespace JSON value so unknown strings and previously ignored nonstring/null values keep their default behavior. `--no-config` remains condense with legacy breaks disabled. Existing Vapor, syntax and other config precedence stays intact. Stats cache identity includes preserve and legacy-break bits above the pre-existing bits; all three semantic modes have separate entries.

## Evidence and remaining work

The corpus preserves the reporter’s complete `App.vue`, JSON config and TS config with byte/hash pins and scoped Git filter exclusion. Whole Rust CLI checks cover all config routes and DOM/SSR/Vapor plus JS/JSON/stats output. A separate actual-cache scenario verifies complete output lengths, repeated hits and parser-scope restoration. The existing tooling job compares twelve full DOM/SSR rendered outputs with official Vue 3.5.35; the reported 3.6.0-rc.10 is explicitly distinct. All complete process streams/modules and render observations are retained before assertions.

Current status is implementation/source preparation only. Fresh exact-head Actions, protected instruction ceilings and full Rust/fixture suites, actual signed merge/reporter credit, and public-release original replay remain TODO. No source test, queue entry or merged code is described as publication. This legacy fix does not complete Davinci or claim editor/typecheck speed gains.
