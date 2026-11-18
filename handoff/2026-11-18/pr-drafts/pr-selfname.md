## Summary

Vue's template compiler derives a component's self name from its file name minus the last extension (`filename.match(/([^/\\]+)\.\w+$/)` in compiler-core, then `capitalize(camelize(...))`). So `router-link.demo.vue` is `RouterLink.demo`, and `<router-link>` inside it is not a self-reference. `vue/require-component-registration` cut the file name at the first dot instead, treated `<router-link>` as recursion and skipped the check. That missed naive-ui's `typography/demos/*/router-link.demo.vue`, which eslint-plugin-vue reports (Real Project Matrix, #12336).

`file_stem` now drops only the last extension.

## Tests

- `self_name::tests::strips_directories_and_extension` adds `router-link.demo.vue` and `Item.stories.vue`.
- `tests::multi_dot_file::a_multi_dot_file_name_is_not_a_shorter_self_name`: the tag is reported in `router-link.demo.vue` and stays clean in `router-link.vue`.
- Differential case `linter/self-name-multi-dot-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
