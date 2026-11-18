import fs from "node:fs";
const d = process.argv[2];
const strip = (s) =>
  s.replace(/\n🤖 Generated with \[Claude Code\]\(https:\/\/claude.com\/claude-code\)\s*$/u, "").trim().replace(/^## Summary\n\n/u, "").replace(/^## /gmu, "#### ");
const vohs = strip(fs.readFileSync(`${d}/pr-vohs.md`, "utf8"));
const vue2 = strip(fs.readFileSync(`${d}/pr-vue2.md`, "utf8")).replace("The matrix config (from the v-on-handler-style PR this one stacks on)", "The matrix config from item 2");
const body = `## Summary

This PR changes the matrix harness only: Patina is untouched. It bundles three harness fixes to the Real Project Matrix (#12336) that sit on top of the isolated fixtures from #13286. Each item is its own commit, so any one can be reverted alone. With the other lane PRs, the local all-branch measurement on the 57 projects shared with run 44093557816 goes from 5230 FP / 3645 FN to 5 FP / 2 FN.

### 1. The baseline reports findings at the lines their nodes occupy

vue-eslint-parser 10.4 can return \`<script setup>\` nodes whose \`loc\` disagrees with their \`range\` when the SFC also has a plain \`<script>\`. In scalar \`packages/api-client/src/v2/components/code-input/CodeInput.vue\`, every setup node sits 29 lines below its range. Rules report \`loc\`, so the baseline put findings on lines that hold other code, and each one split into a false positive and a false negative (24 pairs in scalar alone).

The reproduction is the pinned \`TwoScripts.vue.txt\` fixture: a plain \`<script>\` followed by \`<script setup>\`, where the setup nodes' \`loc\` lines point past the code their ranges cover. The baseline parser now gives every script node, token and comment the \`loc\` its own \`range\` names. Template nodes are untouched.

The use-v-on-exact oracle pins this helper by hash, so its receipt moves to the new digest. Its packets are unchanged.

### 2. vue/v-on-handler-style runs with eslint-plugin-vue's default styles

${vohs}

### 3. Vue 2 projects are linted with their Vue 2 dialect

${vue2}

## Tests

- \`lint-baseline-script-locations.test.ts\`: script nodes, tokens and comments report the lines their ranges name, and template nodes keep theirs.
- \`lint-matrix-rule-options.test.ts\` and \`lint-vue2-dialect.test.ts\`, as described above.
- The \`lint-divergence*\`, isolation and rule-location tooling suites pass with the combined \`main.ts\` (isolation, matrix config and dialect).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
`;
fs.writeFileSync(`${d}/pr-harness.md`, body);
