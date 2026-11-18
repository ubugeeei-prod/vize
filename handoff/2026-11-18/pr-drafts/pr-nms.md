## Summary

eslint-plugin-vue's `vue/no-multi-spaces` checks the tokens of every top-level SFC element. For the template body that means all of its tokens; for `<script>`, `<style>` and custom blocks, the tokens of the start and end tags. Patina only checked elements inside the template, so these gaps were false negatives in the Real Project Matrix (#12336):

| Project | Tag |
| --- | --- |
| motion-vue `reorder/Item.vue` | `<script setup  generic="..." lang="ts">` |
| reka-ui `Listbox/ListboxItem.vue` | `<script setup lang="ts"  generic="...">` |
| vue-element-admin `split-pane.vue` | `<style  scoped>` |
| habitica `toggleCheckbox.vue` | `<style  lang="scss" scoped>` |
| epic-spinners (4 docs components) | `<style  lang="scss">` |

The rule now also checks the start and end tags of the template, script, script setup, style and custom blocks. Like upstream, it reports a whitespace run longer than one byte between two tokens that start on the same line. Quoted values and line breaks are not reported.

`vue/no-multi-spaces` joins `SHARED_SFC_DESCRIPTOR_RULES`, so the engine hands it the descriptor it already parsed.

## Tests

- `no_multi_spaces::blocks::tests`: gaps in a script setup start tag, a script end tag, the template start tag (reported once), a style start tag and a custom block before `>`. Multi-line tags, spaces inside quoted values and a gap after a multi-line quoted value are not reported.
- Differential case `linter/no-multi-spaces-block-tags-12336`.

Local `lint-divergence-report` for epic-spinners, habitica, motion-vue, reka-ui and vue-element-admin: `vue/no-multi-spaces` goes from 8 FN to 0 FP / 0 FN.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
