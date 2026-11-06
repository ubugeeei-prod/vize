---
title: "Musea 规则"
description: "检查 Musea 的 `.art.vue` 故事、变体和设计令牌用法。"
---

# Musea 规则

检查 Musea 的 `.art.vue` 故事、变体和设计令牌用法。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

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
