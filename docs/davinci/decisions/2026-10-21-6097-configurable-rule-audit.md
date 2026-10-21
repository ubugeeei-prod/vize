# #6097 configurable-rule audit

Date: 2026-10-21

## Authority

- Current upstream release: [`eslint-plugin-vue` v10.11.1](https://github.com/vuejs/eslint-plugin-vue/releases/tag/v10.11.1), published 2026-09-24.
- Release commit: [`ec9c53a6fbed637d7b0fbcd6893b0206ae32839f`](https://github.com/vuejs/eslint-plugin-vue/tree/ec9c53a6fbed637d7b0fbcd6893b0206ae32839f).
- The committed inventory is [`patina-eslint-vue-configurable-audit.json`](../../../tests/_fixtures/patina-eslint-vue-configurable-audit.json).

The audit enumerates each unique top-level rule module at the release commit and treats a non-empty
`meta.schema` as configurable. That produces 253 rules, of which 112 accept options.

## Result

Of the 112 configurable upstream rules, the existing compatibility map classifies 58 as mapped,
52 as unimplemented, and 2 as intentional divergences. Thirteen mapped Patina rules expose typed
project configuration today; 45 mapped rules still have no typed option surface.

The machine check verifies the complete configurable-rule list, status totals, mapped Patina rule
IDs, and their public JSON Schema entries. The smallest high-value next slice is
`vue/v-slot-style`: it is strongly recommended, fixable, already implements authored ranges and
safe fixes, and has both uniform and position-specific behavior internally.

The stacked implementation updates that count to 14 configured mapped rules and leaves 44 mapped
rules without typed options. It exposes both upstream-compatible uniform values and the richer
position-specific object while rejecting bare `v-slot` for named slots at the config boundary.

## Provider boundary

The executable differential provider remains pinned to 10.9.2. Moving it to 10.11.1 changes
reviewed real-project observations, so that move requires separate corpus review rather than a
metadata-only refresh. The only rule added since the pinned provider is
`vue/no-shadow-native-events`; it has no options, so it does not change the 112-rule configurable
surface audited here.
