---
title: Musea and CSS rules
---

# Musea and CSS rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Purpose |
| --- | --- |
| [`css/no-display-none`](./reference/css-no-display-none.md) | Suggest using v-show instead of display: none |
| [`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | Suggest using CSS variables instead of hardcoded values |
| [`css/no-id-selectors`](./reference/css-no-id-selectors.md) | Discourage use of ID selectors in CSS |
| [`css/no-important`](./reference/css-no-important.md) | Discourage use of !important in CSS |
| [`css/no-utility-classes`](./reference/css-no-utility-classes.md) | Warn against implementing utility classes in component styles |
| [`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | Warn about performance cost of CSS v-bind() |
| [`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | Recommend CSS logical properties for better i18n support |
| [`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | Recommend using CSS nesting for descendant selectors |
| [`css/prefer-slotted`](./reference/css-prefer-slotted.md) | Recommend ::v-slotted() for styling slot content |
| [`css/require-font-display`](./reference/css-require-font-display.md) | Require font-display in @font-face rules |
| [`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Disallow empty &lt;variant&gt; blocks |
| [`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Prefer design token CSS variables over hardcoded primitive values |
| [`musea/require-component`](./reference/musea-require-component.md) | Require component attribute in &lt;art&gt; block |
| [`musea/require-title`](./reference/musea-require-title.md) | Require title attribute in &lt;art&gt; block |
| [`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Require unique variant names |
| [`musea/valid-variant`](./reference/musea-valid-variant.md) | Require name attribute in &lt;variant&gt; blocks |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)
