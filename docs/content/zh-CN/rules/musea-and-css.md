---
title: "Musea 与 CSS 规则"
description: "检查 Musea 的 `.art.vue` 故事以及 `<style>` 块中的普通 CSS。"
---

# Musea 与 CSS 规则

检查 Musea 的 `.art.vue` 故事以及 `<style>` 块中的普通 CSS。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 更严格的预设 (10)

只在 `opinionated`、`nuxt` 或 `ecosystem` 中。具体是哪些见“预设”列。

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | 建议使用 v-show 代替 display: none。 | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | 建议使用 CSS 变量代替硬编码值。 | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | 不建议在 CSS 中使用 ID 选择器。 | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | 不建议在 CSS 中使用 !important。 | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | 警告在组件样式中实现工具类。 | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | 警告 CSS v-bind() 的性能开销。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | 建议使用 CSS 逻辑属性，以更好地支持国际化。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | 建议为后代选择器使用 CSS 嵌套。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | 建议使用 ::v-slotted() 为插槽内容设置样式。 | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | 要求 @font-face 规则具有 font-display。 | `opinionated`, `nuxt` |  |

## 按需启用 (6)

不属于任何预设。在 `vue.linter.rules` 中按名称启用。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | 禁止空的 &lt;variant&gt; 块。 |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | 优先使用设计令牌 CSS 变量，而不是硬编码的原始值。 |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | 要求 &lt;art&gt; 块具有 component 属性。 |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | 要求 &lt;art&gt; 块具有 title 属性。 |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | 要求 variant 名称唯一。 |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | 要求 &lt;variant&gt; 块具有 name 属性。 |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
