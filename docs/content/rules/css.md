---
title: "CSS rules"
description: "Plain CSS in `<style>` blocks, including correctness and maintainability checks."
---

# CSS rules

Plain CSS in `<style>` blocks, including correctness and maintainability checks.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Stricter presets (10)

Only in `opinionated`, `nuxt` or `ecosystem`. The Presets column lists which.

| Rule | What it checks | Presets | Auto-fix |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | Suggest using v-show instead of display: none. | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | Suggest using CSS variables instead of hardcoded values. | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | Discourage use of ID selectors in CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | Discourage use of !important in CSS. | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | Warn against implementing utility classes in component styles. | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | Warn about performance cost of CSS v-bind(). | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | Recommend CSS logical properties for better i18n support. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | Recommend using CSS nesting for descendant selectors. | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | Recommend ::v-slotted() for styling slot content. | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | Require font-display in @font-face rules. | `opinionated`, `nuxt` |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
