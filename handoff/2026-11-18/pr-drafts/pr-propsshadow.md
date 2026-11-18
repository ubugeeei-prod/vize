## Summary

eslint-plugin-vue parses an SFC's `<script>` and `<script setup>` as one program. A `defineProps<Props>()` whose interface lives in the sibling `<script lang="ts">` block therefore has known props, as in reka-ui and nuxt-ui components. `vue/no-props-shadow` only resolved types declared in `<script setup>`, which left about 700 shadows unreported in the Real Project Matrix (#12336).

The rule now resolves prop names from the sibling script too (`same_sfc_type_prop_names`). An annotated binding is reported with its type annotation, since an ESTree identifier spans it (`label: string`).

## Tests

- `options::tests`: props declared in the sibling script, interface heritage across the two blocks, and an annotated binding's range. One existing expectation now includes the annotation.
- Differential case `linter/no-props-shadow-sibling-script-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
