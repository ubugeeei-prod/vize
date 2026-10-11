# Malformed setup metadata

The original Vue Language Tools inputs and configuration chain are byte exact
at revision `88e8500c1e5f1b29d80f42c8ca065cc9cbd56899`; hashes and the original
hosted error vectors are in `original-custody.json`. The copied MIT license
preserves attribution. `real-errors.vue.txt` adds an actual undeclared name and
number/string mismatch while keeping the original malformed setup expression.
`valid-commented-import.vue.txt` retains a valid import followed by a block
comment and newline before a genuine later expression error; its native
boundary variants must not create additional import or terminator errors.
`valid-quoted-import.vue.txt` checks a quoted imported binding name against the
actual separate Vue module source, preserving only its later expression error.
`overlapping-imports.vue.txt` keeps a valid leading import and a later glued
import; both declarations must be emitted once, with only the real TS1005.

The internal Canon filter `vue_codegen::malformed_setup_tests::` preserves
complete before/after descriptor, Croquis, generated code and mapping packets.
Its native controls retain original configurations, pinned Vue 3.6.0-rc.6 and
full same-config API/LSP diagnostic vectors. Set `VIZE_TEST_REQUIRE_TSGO=1` to
require the native runtime; `VIZE_NATIVE_SYNTAX_CAPTURE_DIR` retains complete
packets before temporary workspaces are removed. Syntax-only CLI output does
not prove semantic completion. The original malformed syntax must stay visible,
the false generic/import diagnostics must disappear, and real TS2304/TS2322
diagnostics must remain. No upstream snapshot or Vize difference is waived.

Decision: `docs/davinci/decisions/2026-10-11-6879-malformed-setup-descriptor-metadata.md`.

`valid-cr-import.vue.txt` retains the exact lone-CR boundary that native Corsa
reported correctly while the older LF-only source coordinate converter lost
the mapped expression diagnostic. The same input is checked against the prior
converter and the existing Carton LSP/UTF-16 authority; LS/PS remain within LSP
lines, while compiler diagnostics retain their separate TypeScript convention.
