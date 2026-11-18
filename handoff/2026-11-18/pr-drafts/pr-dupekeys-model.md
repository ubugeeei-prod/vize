## Summary

`const modelValue = defineModel<boolean>()` beside a `modelValue` prop produced two `script/no-dupe-keys` findings: one on the binding and one on the `defineModel()` call. eslint-plugin-vue reports the key once. Checked against eslint-plugin-vue 10.12.0 with `interface Props { modelValue: boolean; title?: string }`:

| Declaration | eslint-plugin-vue | Patina before | Patina after |
| --- | --- | --- | --- |
| `const modelValue = defineModel<boolean>()` | binding | binding + call | binding |
| `const open = defineModel<boolean>()` | call | call | call |
| `const title = defineModel<string>('title')` | binding | binding + `'title'` | binding |
| `const open = defineModel<string>('title')` | `'title'` | `'title'` | `'title'` |

The model finding is now skipped when the model's own binding has the same name and has already been reported against the prop. Seen in airi `CardCreationDialog.vue` (Real Project Matrix, #12336).

## Tests

- `no_dupe_keys::setup::tests::a_model_named_like_its_colliding_binding_is_reported_once` pins the four rows above.
- Differential case `linter/no-dupe-keys-model-binding-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
