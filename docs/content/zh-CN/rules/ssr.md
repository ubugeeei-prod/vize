---
title: "SSR 规则"
---

# SSR 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="ssr规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](https://vizejs.dev/zh-CN/rules/ssr.html#ssr-no-browser-globals-in-ssr) | [错误示例](https://vizejs.dev/zh-CN/rules/ssr.html#ssr-no-browser-globals-in-ssr-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ssr.html#ssr-no-browser-globals-in-ssr-good) | 禁止 SSR 上下文中的浏览器专用全局变量 |
| [`ssr/no-hydration-mismatch`](https://vizejs.dev/zh-CN/rules/ssr.html#ssr-no-hydration-mismatch) | [错误示例](https://vizejs.dev/zh-CN/rules/ssr.html#ssr-no-hydration-mismatch-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/ssr.html#ssr-no-hydration-mismatch-good) | 禁止导致水合不匹配的不确定值 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)
