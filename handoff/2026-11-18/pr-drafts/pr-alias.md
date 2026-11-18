Patina honors inline `eslint-disable` comments, so a codebase migrating from eslint-plugin-vue keeps its suppressions. That only worked when the comment named a rule Patina registers under the same ID, or one of 18 hard-coded aliases (17 of them in the rule map). The rule map has 62 eslint-plugin-vue rules that Patina implements under another name, and the other 45 were never suppressed. For example, `<!-- eslint-disable vue/prefer-separate-static-class -->` left `vapor/prefer-static-class` reporting. The Real Project Matrix (#12336) counted those lines as false positives: 51 in airi's VitePress components alone.

**Change**
- The aliases live in one sorted table, `context/eslint_vue_aliases.rs`.
- The table is the rule map's renamed `mapped` entries, plus the Vue 2 name `vue/no-shared-component-data` from the old list.
- A test compares the table with `tests/_fixtures/patina-eslint-vue-rule-map.json`. A new rename in the map fails until its alias is added.
- A second test asserts the table is sorted and that no upstream ID in it is itself a registered Patina rule. Mapping an ID therefore never hides a rule that has that exact name.
- Lookup is a binary search over 63 entries per listed rule ID. It runs only for directive comments.

**Coverage**, all with renamed IDs that were not aliased before:
- template `eslint-disable-next-line` with a comma list mixing a native and a renamed ID;
- `eslint-disable-line`;
- an `eslint-disable` / `eslint-enable` pair, followed by a finding that stays reported;
- script block `/* eslint-disable ... */` and `/* eslint-enable */`;
- `// eslint-disable-line`;
- `// eslint-disable-next-line` with a mixed list;
- an unrelated ID in the same comment position, which still leaves the finding.

The input-only differential case `linter/eslint-disable-mapped-aliases-12336` pins a reduction.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
