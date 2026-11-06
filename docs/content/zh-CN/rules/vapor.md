---
title: "Vapor 规则"
description: "以 Vapor 模式编译的组件在模板和脚本上的限制。"
---

# Vapor 规则

以 Vapor 模式编译的组件在模板和脚本上的限制。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 默认 (1)

由默认预设 `happy-path` 添加。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | 禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）。 |  |

## 更严格的预设 (5)

只在 `opinionated`、`nuxt` 或 `ecosystem` 中。具体是哪些见“预设”列。

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）。 | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | 禁止 Vapor 模式中的 Options API 模式。 | `opinionated` |  |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | 禁止已弃用的 inline-template 属性。 | `nuxt`, `opinionated` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | 字符串字面量优先使用静态 class，而不是动态 class 绑定。 | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | 建议为 script setup 添加 vapor 属性。 | `nuxt`, `opinionated` |  |

## 按需启用 (1)

不属于任何预设。在传给 `withVue` 的 `linter.rules` 中按名称启用。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | 禁止面向 Vapor 的组件使用 nextTick()。 |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
