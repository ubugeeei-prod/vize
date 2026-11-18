## Summary

`vue/no-restricted-v-bind` reported the whole `:v-bind="{ item: recipe }"` attribute, while eslint-plugin-vue reports the directive key (`:v-bind`). Each finding therefore split into a false positive and a false negative in the Real Project Matrix (#12336): mealie `RecipeList.vue` and `shopping-lists/[id].vue`.

The rule now reports from the attribute start up to `=`, or up to the first whitespace for a valueless attribute. Modifiers stay in the range, as in `:v-show.camel`, the same as the ESTree `VDirectiveKey`.

## Tests

- `compatibility_template::tests::restricted_v_bind_reports_the_directive_key`: `:v-bind`, `:v-model` and `:v-show.camel`.
- Differential case `linter/no-restricted-v-bind-key-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
