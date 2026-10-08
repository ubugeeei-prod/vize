---
title: Musea and CSS rules
---

# Musea and CSS rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`css/no-display-none`](./reference/css-no-display-none.md) | [Bad](./reference/css-no-display-none.md#bad) · [Good](./reference/css-no-display-none.md#good) | Suggest using v-show instead of display: none |
| [`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | [Bad](./reference/css-no-hardcoded-values.md#bad) · [Good](./reference/css-no-hardcoded-values.md#good) | Suggest using CSS variables instead of hardcoded values |
| [`css/no-id-selectors`](./reference/css-no-id-selectors.md) | [Bad](./reference/css-no-id-selectors.md#bad) · [Good](./reference/css-no-id-selectors.md#good) | Discourage use of ID selectors in CSS |
| [`css/no-important`](./reference/css-no-important.md) | [Bad](./reference/css-no-important.md#bad) · [Good](./reference/css-no-important.md#good) | Discourage use of !important in CSS |
| [`css/no-utility-classes`](./reference/css-no-utility-classes.md) | [Bad](./reference/css-no-utility-classes.md#bad) · [Good](./reference/css-no-utility-classes.md#good) | Warn against implementing utility classes in component styles |
| [`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | [Bad](./reference/css-no-v-bind-performance.md#bad) · [Good](./reference/css-no-v-bind-performance.md#good) | Warn about performance cost of CSS v-bind() |
| [`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | [Bad](./reference/css-prefer-logical-properties.md#bad) · [Good](./reference/css-prefer-logical-properties.md#good) | Recommend CSS logical properties for better i18n support |
| [`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | [Bad](./reference/css-prefer-nested-selectors.md#bad) · [Good](./reference/css-prefer-nested-selectors.md#good) | Recommend using CSS nesting for descendant selectors |
| [`css/prefer-slotted`](./reference/css-prefer-slotted.md) | [Bad](./reference/css-prefer-slotted.md#bad) · [Good](./reference/css-prefer-slotted.md#good) | Recommend ::v-slotted() for styling slot content |
| [`css/require-font-display`](./reference/css-require-font-display.md) | [Bad](./reference/css-require-font-display.md#bad) · [Good](./reference/css-require-font-display.md#good) | Require font-display in @font-face rules |
| [`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | [Bad](./reference/musea-no-empty-variant.md#bad) · [Good](./reference/musea-no-empty-variant.md#good) | Disallow empty &lt;variant&gt; blocks |
| [`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | [Bad](./reference/musea-prefer-design-tokens.md#bad) · [Good](./reference/musea-prefer-design-tokens.md#good) | Prefer design token CSS variables over hardcoded primitive values |
| [`musea/require-component`](./reference/musea-require-component.md) | [Bad](./reference/musea-require-component.md#bad) · [Good](./reference/musea-require-component.md#good) | Require component attribute in &lt;art&gt; block |
| [`musea/require-title`](./reference/musea-require-title.md) | [Bad](./reference/musea-require-title.md#bad) · [Good](./reference/musea-require-title.md#good) | Require title attribute in &lt;art&gt; block |
| [`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | [Bad](./reference/musea-unique-variant-names.md#bad) · [Good](./reference/musea-unique-variant-names.md#good) | Require unique variant names |
| [`musea/valid-variant`](./reference/musea-valid-variant.md) | [Bad](./reference/musea-valid-variant.md#bad) · [Good](./reference/musea-valid-variant.md#good) | Require name attribute in &lt;variant&gt; blocks |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
