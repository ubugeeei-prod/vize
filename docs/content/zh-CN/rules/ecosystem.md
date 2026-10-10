---
title: "生态系统规则"
---

# 生态系统规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。


| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link-good) | 应用内部链接优先使用 NuxtLink |
| [`ecosystem/pinia-prefer-store-to-refs`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs-good) | 解构 Pinia store 时优先使用 storeToRefs() |
| [`ecosystem/router-link-require-to`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-router-link-require-to) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-router-link-require-to-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-router-link-require-to-good) | 要求 RouterLink 和 NuxtLink 组件具有 `to` 目标 |
| [`ecosystem/void-link-require-href`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-void-link-require-href) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-void-link-require-href-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-void-link-require-href-good) | 要求 Void Vue Link 组件具有 `href` |
| [`ecosystem/void-link-valid-method`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-void-link-valid-method) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-void-link-valid-method-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-void-link-valid-method-good) | 验证 Void Vue Link 的静态 method prop |
| [`ecosystem/vue-i18n-no-missing-key`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key-good) | 报告本地 SFC 消息中不存在的静态 vue-i18n 键 |
| [`ecosystem/vue-router-prefer-named-link`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link-good) | RouterLink 优先使用具名路由对象，而不是静态路径字符串 |
| [`ecosystem/vue-router-prefer-named-push`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push-good) | Vue Router 编程式导航优先使用具名路由对象 |
| [`ecosystem/vue-test-utils-no-html-snapshot`](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot-good) | 避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照 |
| [`nuxt/no-nuxt-config-test-key`](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-no-nuxt-config-test-key) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-no-nuxt-config-test-key-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-no-nuxt-config-test-key-good) | 禁止在 Nuxt 配置中设置 `test` 键 |
| [`nuxt/no-page-meta-runtime-values`](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-no-page-meta-runtime-values) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-no-page-meta-runtime-values-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-no-page-meta-runtime-values-good) | 禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行 |
| [`nuxt/nuxt-config-keys-order`](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-nuxt-config-keys-order) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-nuxt-config-keys-order-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-nuxt-config-keys-order-good) | 优先使用推荐的 Nuxt 配置属性顺序 |
| [`nuxt/prefer-import-meta`](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-prefer-import-meta) | [错误示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-prefer-import-meta-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ecosystem.html#nuxt-prefer-import-meta-good) | 优先使用 `import.meta.*`，而不是 `process.*` |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)
