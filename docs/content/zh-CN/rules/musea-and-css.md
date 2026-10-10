---
title: "Musea 与 CSS 规则"
---

# Musea 与 CSS 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="博物馆与css规则"></span>
<span id="额外的css规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`css/no-display-none`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-display-none) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-display-none-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-display-none-good) | 建议使用 v-show 代替 display: none |
| [`css/no-hardcoded-values`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-hardcoded-values) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-hardcoded-values-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-hardcoded-values-good) | 建议使用 CSS 变量代替硬编码值 |
| [`css/no-id-selectors`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-id-selectors) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-id-selectors-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-id-selectors-good) | 不建议在 CSS 中使用 ID 选择器 |
| [`css/no-important`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-important) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-important-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-important-good) | 不建议在 CSS 中使用 !important |
| [`css/no-utility-classes`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-utility-classes) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-utility-classes-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-utility-classes-good) | 警告在组件样式中实现工具类 |
| [`css/no-v-bind-performance`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-v-bind-performance) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-v-bind-performance-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-no-v-bind-performance-good) | 警告 CSS v-bind() 的性能开销 |
| [`css/prefer-logical-properties`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-logical-properties) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-logical-properties-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-logical-properties-good) | 建议使用 CSS 逻辑属性，以更好地支持国际化 |
| [`css/prefer-nested-selectors`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-nested-selectors) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-nested-selectors-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-nested-selectors-good) | 建议为后代选择器使用 CSS 嵌套 |
| [`css/prefer-slotted`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-slotted) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-slotted-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-prefer-slotted-good) | 建议使用 ::v-slotted() 为插槽内容设置样式 |
| [`css/require-font-display`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-require-font-display) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-require-font-display-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#css-require-font-display-good) | 要求 @font-face 规则具有 font-display |
| [`musea/no-empty-variant`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-no-empty-variant) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-no-empty-variant-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-no-empty-variant-good) | 禁止空的 &lt;variant&gt; 块 |
| [`musea/prefer-design-tokens`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-prefer-design-tokens) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-prefer-design-tokens-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-prefer-design-tokens-good) | 优先使用设计令牌 CSS 变量，而不是硬编码的原始值 |
| [`musea/require-component`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-require-component) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-require-component-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-require-component-good) | 要求 &lt;art&gt; 块具有 component 属性 |
| [`musea/require-title`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-require-title) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-require-title-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-require-title-good) | 要求 &lt;art&gt; 块具有 title 属性 |
| [`musea/unique-variant-names`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-unique-variant-names) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-unique-variant-names-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-unique-variant-names-good) | 要求 variant 名称唯一 |
| [`musea/valid-variant`](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-valid-variant) | [错误示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-valid-variant-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/musea-and-css.html#musea-valid-variant-good) | 要求 &lt;variant&gt; 块具有 name 属性 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)
