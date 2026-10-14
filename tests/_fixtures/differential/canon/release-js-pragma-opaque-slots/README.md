# JavaScript pragma and opaque slot host controls

Regression inputs for two false diagnostics found by the full Vue parity job
(Check run `38356181245`).

- `Pragma.vue.txt` is a JavaScript SFC whose block starts with `// @ts-check`.
  When `allowJs` is absent from the project configuration the block must still
  be checked; only an explicit `allowJs: false` denies the per-file opt-in.
- `App.vue.txt` passes a `header` slot to an external component.
  `opaque.d.ts.txt` declares that component the way an ecosystem adapter does,
  `DefineComponent<Record<string, unknown>, {}, any>`. Its open props make the
  host opaque, so the slot is accepted. A known empty slot contract and a
  wrongly typed slot still report `TS2339`.

The Rust controls are
`batch::type_checker::tests::recent_issues::release_opaque_slot_hosts` and
`batch::virtual_project::project::artifacts::pragma_projection_tests` in
`vize_canon`.
