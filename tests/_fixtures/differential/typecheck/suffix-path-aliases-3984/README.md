# Suffix path alias reachability

Authored regression controls for [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).
These inputs are not attributed to an external reporter or project.

The existing Canon explicit scan registers only `src/App.vue`. Its import uses
the valid single-wildcard path mapping `@/*.vue` → `src/*.vue` and must reach
`src/components/api/DirectiveTable.vue`.

- `tsconfig.json.txt` is copied to the test project's `tsconfig.json`.
- `App.vue.txt` is copied to `src/App.vue`.
- `DirectiveTable.vue.txt` is copied to the imported child. Its second line
  must report TS2322 at authored zero-based `(1, 6)`.
- `DirectiveTable.repaired.vue.txt` replaces only the child. Repeating the
  same App-only cold scan with a fresh checker must retain both Vue files and
  report no diagnostics.

The native law is
`batch::type_checker::tests::scan::suffix_path_aliases::app_only_suffix_alias_reports_and_repairs_the_authored_child`.
The existing required-native Actions environment must execute it; a locally
unavailable Corsa executable does not establish a passing native result.
