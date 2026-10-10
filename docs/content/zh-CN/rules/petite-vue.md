---
title: "petite-vue 规则"
---

# petite-vue 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。


| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-no-unsupported-directive) | [错误示例](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-no-unsupported-directive-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-no-unsupported-directive-good) | 禁止 petite-vue 不支持的指令 |
| [`petite-vue/valid-v-effect`](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-valid-v-effect) | [错误示例](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-valid-v-effect-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-valid-v-effect-good) | 要求 v-effect 具有非空表达式 |
| [`petite-vue/valid-v-scope`](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-valid-v-scope) | [错误示例](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-valid-v-scope-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/petite-vue.html#petite-vue-valid-v-scope-good) | 要求 v-scope 绑定对象字面量 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)
