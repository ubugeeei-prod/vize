## Summary

vue-eslint-parser reads a Vue 2 `slot-scope` (or `scope`) attribute as a directive whose value is a parameter list, so `vue/no-empty-pattern` reports `slot-scope="{}"`. Patina's template AST keeps such attributes as plain attributes, so the rule missed them (vue-element-admin `drag-table.vue` in the Real Project Matrix, #12336).

The rule now scans `slot-scope` and `scope` attribute values for `{}` and `[]` in pattern positions: at the start of the list, or after `(`, `,`, `:` or `[`. An empty literal after `=` is a default value and stays clean. The scan works on the authored text, so no new JavaScript parse is added (the shared-parse gate holds).

## Tests

- `slot_scope::tests`: `{}`, nested `{}` and `[ ]`, `scope='[]'`; `{ row = {} }`, `{ row }` and `scope` are clean.
- `compatibility_template::tests::empty_pattern`: the rule reports `{}` in `slot-scope` and nothing in `scope="{ row }"`.
- Differential case `linter/no-empty-pattern-slot-scope-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
