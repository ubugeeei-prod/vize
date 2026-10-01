# Config path matcher host ownership

Issue: [#6834](https://github.com/ubugeeei-prod/vize/issues/6834).
Prerequisite: [configuration host ownership](./2026-10-01-config-host-owner.md).

Carton owns the existing ordered glob matcher and its tests. The module
moves unchanged in a move-only commit, and the actual CLI lint-plan facade
and LSP workspace-folder scopes consume the host owner. L0 loses the
matcher module, optional `globset` dependency and `lint-glob` feature.
Carton owns that optional dependency and feature, enabled by the actual
CLI/LSP consumers. No normal or build dependency points back into `crates/`.

Ordered positive and negative patterns, directory-parent matching,
literal glob escapes, Windows separators, base-path selection and lexical
normalization remain unchanged. Invalid patterns retain their existing
warning bytes. The core effective rule-plan types, including their compact
strings, remain L0 values and are passed directly to the host matcher.
No conversion, additional pipeline stage, serialization or compiler/checker
route change is introduced. Numeric ceilings and frozen fixtures remain
unchanged; only source ownership witnesses are regenerated.

The product storage gate permits only the two existing CLI matcher re-export
lines and the LSP's exact `LintPlanScope` import in its workspace-folder file.
Mutation laws still reject Carton storage, other symbols, grouped escape
imports and these host imports in other source files. Slash and Windows
source-path variants are checked against the actual consumers.

Replay with `vp node tools/support/levels/move-config-matcher-host.ts moves`,
commit the pure move, then run `integrate` and `check`. The published
host-import gate companion must accompany replay. Preflight rejects
collisions and unexpected caller/manifest states. Repeated phases remain
clean, and the earlier config and project replays retain this exact later
host module extension.

The preceding configuration and project ownership changes have merged through
the protected queue. Publication uses fresh `main`, source review,
exact-head Actions and all100 instruction measurements. This is an independent
host ownership slice, not a new portable-provider API. #6834 remains open:
framework placement of effective types, i18n, profiler and full platform
isolation are unfinished. `VueVersion` still has actual level consumers.
