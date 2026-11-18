## Summary

eslint-plugin-vue follows Composition API imports from four modules through `createCompositionApiTraceMap`: `vue`, `@vue/composition-api`, `@nuxtjs/composition-api` and unimport's `#imports`. Patina's matching rules each kept their own list, and none included `@nuxtjs/composition-api`. So `vue/no-ref-object-reactivity-loss` missed two `.value` copies in vue-storefront (`SearchResults.vue`, `Category.vue`) in the Real Project Matrix (#12336); those files import `computed` from `@nuxtjs/composition-api`.

A shared `rules::script::composition_api::is_composition_api_module` now backs the rules whose upstream counterparts use that trace map:

| Patina rule | Before | Now |
| --- | --- | --- |
| no-ref-object-reactivity-loss, no-ref-as-operand | vue, @vue/composition-api, #imports | + @nuxtjs/composition-api |
| no-async-in-computed-properties, no-side-effects-in-computed-properties | vue, @vue/composition-api | + @nuxtjs/composition-api, #imports |
| lifecycle / watch / expose after await | vue, @vue/composition-api, @vue/runtime-core | + @nuxtjs/composition-api, #imports (keeps @vue/runtime-core) |

Behavior change: these rules now also report code that imports the Composition API from `@nuxtjs/composition-api`, and the last two rows from `#imports`, as eslint-plugin-vue does.

## Tests

- `no_ref_object_reactivity_loss::tests::reads_every_composition_api_module_eslint_plugin_vue_traces`: each of the four modules reports a copy, and an unrelated module does not.
- Differential case `linter/nuxtjs-composition-api-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
