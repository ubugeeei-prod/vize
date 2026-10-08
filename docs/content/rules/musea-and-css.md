---
title: Musea and CSS rules
---

# Musea and CSS rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`css/no-display-none`](./all.md#css-no-display-none) | [Bad](./all.md#css-no-display-none-bad) · [Good](./all.md#css-no-display-none-good) | Suggest using v-show instead of display: none |
| [`css/no-hardcoded-values`](./all.md#css-no-hardcoded-values) | [Bad](./all.md#css-no-hardcoded-values-bad) · [Good](./all.md#css-no-hardcoded-values-good) | Suggest using CSS variables instead of hardcoded values |
| [`css/no-id-selectors`](./all.md#css-no-id-selectors) | [Bad](./all.md#css-no-id-selectors-bad) · [Good](./all.md#css-no-id-selectors-good) | Discourage use of ID selectors in CSS |
| [`css/no-important`](./all.md#css-no-important) | [Bad](./all.md#css-no-important-bad) · [Good](./all.md#css-no-important-good) | Discourage use of !important in CSS |
| [`css/no-utility-classes`](./all.md#css-no-utility-classes) | [Bad](./all.md#css-no-utility-classes-bad) · [Good](./all.md#css-no-utility-classes-good) | Warn against implementing utility classes in component styles |
| [`css/no-v-bind-performance`](./all.md#css-no-v-bind-performance) | [Bad](./all.md#css-no-v-bind-performance-bad) · [Good](./all.md#css-no-v-bind-performance-good) | Warn about performance cost of CSS v-bind() |
| [`css/prefer-logical-properties`](./all.md#css-prefer-logical-properties) | [Bad](./all.md#css-prefer-logical-properties-bad) · [Good](./all.md#css-prefer-logical-properties-good) | Recommend CSS logical properties for better i18n support |
| [`css/prefer-nested-selectors`](./all.md#css-prefer-nested-selectors) | [Bad](./all.md#css-prefer-nested-selectors-bad) · [Good](./all.md#css-prefer-nested-selectors-good) | Recommend using CSS nesting for descendant selectors |
| [`css/prefer-slotted`](./all.md#css-prefer-slotted) | [Bad](./all.md#css-prefer-slotted-bad) · [Good](./all.md#css-prefer-slotted-good) | Recommend ::v-slotted() for styling slot content |
| [`css/require-font-display`](./all.md#css-require-font-display) | [Bad](./all.md#css-require-font-display-bad) · [Good](./all.md#css-require-font-display-good) | Require font-display in @font-face rules |
| [`musea/no-empty-variant`](./all.md#musea-no-empty-variant) | [Bad](./all.md#musea-no-empty-variant-bad) · [Good](./all.md#musea-no-empty-variant-good) | Disallow empty &lt;variant&gt; blocks |
| [`musea/prefer-design-tokens`](./all.md#musea-prefer-design-tokens) | [Bad](./all.md#musea-prefer-design-tokens-bad) · [Good](./all.md#musea-prefer-design-tokens-good) | Prefer design token CSS variables over hardcoded primitive values |
| [`musea/require-component`](./all.md#musea-require-component) | [Bad](./all.md#musea-require-component-bad) · [Good](./all.md#musea-require-component-good) | Require component attribute in &lt;art&gt; block |
| [`musea/require-title`](./all.md#musea-require-title) | [Bad](./all.md#musea-require-title-bad) · [Good](./all.md#musea-require-title-good) | Require title attribute in &lt;art&gt; block |
| [`musea/unique-variant-names`](./all.md#musea-unique-variant-names) | [Bad](./all.md#musea-unique-variant-names-bad) · [Good](./all.md#musea-unique-variant-names-good) | Require unique variant names |
| [`musea/valid-variant`](./all.md#musea-valid-variant) | [Bad](./all.md#musea-valid-variant-bad) · [Good](./all.md#musea-valid-variant-good) | Require name attribute in &lt;variant&gt; blocks |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
