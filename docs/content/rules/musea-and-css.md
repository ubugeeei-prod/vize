---
title: "Musea and CSS rules"
---

# Musea and CSS rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`css/no-display-none`](https://vizejs.dev/rules/musea-and-css.html#css-no-display-none) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-no-display-none-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-no-display-none-good) | Suggest using v-show instead of display: none |
| [`css/no-hardcoded-values`](https://vizejs.dev/rules/musea-and-css.html#css-no-hardcoded-values) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-no-hardcoded-values-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-no-hardcoded-values-good) | Suggest using CSS variables instead of hardcoded values |
| [`css/no-id-selectors`](https://vizejs.dev/rules/musea-and-css.html#css-no-id-selectors) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-no-id-selectors-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-no-id-selectors-good) | Discourage use of ID selectors in CSS |
| [`css/no-important`](https://vizejs.dev/rules/musea-and-css.html#css-no-important) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-no-important-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-no-important-good) | Discourage use of !important in CSS |
| [`css/no-utility-classes`](https://vizejs.dev/rules/musea-and-css.html#css-no-utility-classes) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-no-utility-classes-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-no-utility-classes-good) | Warn against implementing utility classes in component styles |
| [`css/no-v-bind-performance`](https://vizejs.dev/rules/musea-and-css.html#css-no-v-bind-performance) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-no-v-bind-performance-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-no-v-bind-performance-good) | Warn about performance cost of CSS v-bind() |
| [`css/prefer-logical-properties`](https://vizejs.dev/rules/musea-and-css.html#css-prefer-logical-properties) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-prefer-logical-properties-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-prefer-logical-properties-good) | Recommend CSS logical properties for better i18n support |
| [`css/prefer-nested-selectors`](https://vizejs.dev/rules/musea-and-css.html#css-prefer-nested-selectors) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-prefer-nested-selectors-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-prefer-nested-selectors-good) | Recommend using CSS nesting for descendant selectors |
| [`css/prefer-slotted`](https://vizejs.dev/rules/musea-and-css.html#css-prefer-slotted) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-prefer-slotted-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-prefer-slotted-good) | Recommend ::v-slotted() for styling slot content |
| [`css/require-font-display`](https://vizejs.dev/rules/musea-and-css.html#css-require-font-display) | [Bad](https://vizejs.dev/rules/musea-and-css.html#css-require-font-display-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#css-require-font-display-good) | Require font-display in @font-face rules |
| [`musea/no-empty-variant`](https://vizejs.dev/rules/musea-and-css.html#musea-no-empty-variant) | [Bad](https://vizejs.dev/rules/musea-and-css.html#musea-no-empty-variant-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#musea-no-empty-variant-good) | Disallow empty &lt;variant&gt; blocks |
| [`musea/prefer-design-tokens`](https://vizejs.dev/rules/musea-and-css.html#musea-prefer-design-tokens) | [Bad](https://vizejs.dev/rules/musea-and-css.html#musea-prefer-design-tokens-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#musea-prefer-design-tokens-good) | Prefer design token CSS variables over hardcoded primitive values |
| [`musea/require-component`](https://vizejs.dev/rules/musea-and-css.html#musea-require-component) | [Bad](https://vizejs.dev/rules/musea-and-css.html#musea-require-component-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#musea-require-component-good) | Require component attribute in &lt;art&gt; block |
| [`musea/require-title`](https://vizejs.dev/rules/musea-and-css.html#musea-require-title) | [Bad](https://vizejs.dev/rules/musea-and-css.html#musea-require-title-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#musea-require-title-good) | Require title attribute in &lt;art&gt; block |
| [`musea/unique-variant-names`](https://vizejs.dev/rules/musea-and-css.html#musea-unique-variant-names) | [Bad](https://vizejs.dev/rules/musea-and-css.html#musea-unique-variant-names-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#musea-unique-variant-names-good) | Require unique variant names |
| [`musea/valid-variant`](https://vizejs.dev/rules/musea-and-css.html#musea-valid-variant) | [Bad](https://vizejs.dev/rules/musea-and-css.html#musea-valid-variant-bad) · [Good](https://vizejs.dev/rules/musea-and-css.html#musea-valid-variant-good) | Require name attribute in &lt;variant&gt; blocks |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
