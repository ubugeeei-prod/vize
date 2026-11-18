`vue/enforce-style-attribute` now matches eslint-plugin-vue 10.12.0:

- The diagnostic covers the whole first denied `<style>...</style>` element. Before, it covered only the start tag.
- Plain, scoped and module are tested in turn. A `<style scoped module>` block is now reported when `module` is not allowed; before, `scoped` shadowed it.

In the Real Project Matrix (#12336) this rule caused 4297 false positives and 4308 false negatives in 130 projects (schedule run 43880794806). Each one was the same finding with a different end range. A local `lint-divergence-report.ts --all-mapped-rules` run over pinia, filebrowser, vuepress, vue-flow, elk and mealie now shows 0 divergences for this rule.

The rule tests cover plain, `module="m"`, `scoped module`, an allowed `scoped` block followed by denied ones (only the first is reported), and custom allow lists. Rule cost is unchanged: it does the same single pass over the style blocks.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
