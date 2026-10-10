---
title: "Vapor 规则"
---

# Vapor 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="蒸汽统治"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`script/no-get-current-instance`](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-get-current-instance) | [错误示例](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-get-current-instance-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-get-current-instance-good) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null） |
| [`script/no-next-tick`](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-next-tick) | [错误示例](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-next-tick-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-next-tick-good) | 禁止面向 Vapor 的组件使用 nextTick() |
| [`script/no-options-api`](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-options-api) | [错误示例](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-options-api-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/vapor.html#script-no-options-api-good) | 禁止 Vapor 模式中的 Options API 模式 |
| [`vapor/no-inline-template`](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-no-inline-template) | [错误示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-no-inline-template-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-no-inline-template-good) | 禁止已弃用的 inline-template 属性 |
| [`vapor/no-vue-lifecycle-events`](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-no-vue-lifecycle-events) | [错误示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-no-vue-lifecycle-events-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-no-vue-lifecycle-events-good) | 禁止 @vue:xxx 元素生命周期事件（Vapor 不支持） |
| [`vapor/prefer-static-class`](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-prefer-static-class) | [错误示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-prefer-static-class-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-prefer-static-class-good) | 字符串字面量优先使用静态 class，而不是动态 class 绑定 |
| [`vapor/require-vapor-attribute`](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-require-vapor-attribute) | [错误示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-require-vapor-attribute-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/vapor.html#vapor-require-vapor-attribute-good) | 建议为 script setup 添加 vapor 属性 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)
