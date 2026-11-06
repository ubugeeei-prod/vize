---
title: "petite-vue 规则"
description: "检查使用 petite-vue 的 HTML 页面中的指令和作用域。"
---

# petite-vue 规则

检查使用 petite-vue 的 HTML 页面中的指令和作用域。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 按需启用 (3)

不属于任何预设。在传给 `withVue` 的 `linter.rules` 中按名称启用。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | 禁止 petite-vue 不支持的指令。 |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | 要求 v-effect 具有非空表达式。 |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | 要求 v-scope 绑定对象字面量。 |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
