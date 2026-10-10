---
title: "HTML 规则"
---

# HTML 规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="html规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`html/deprecated-attr`](https://vizejs.dev/zh-CN/rules/html.html#html-deprecated-attr) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-deprecated-attr-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-deprecated-attr-good) | 禁止已弃用的 HTML 属性 |
| [`html/deprecated-element`](https://vizejs.dev/zh-CN/rules/html.html#html-deprecated-element) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-deprecated-element-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-deprecated-element-good) | 禁止已弃用的 HTML 元素 |
| [`html/id-duplication`](https://vizejs.dev/zh-CN/rules/html.html#html-id-duplication) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-id-duplication-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-id-duplication-good) | 禁止重复的元素 ID |
| [`html/no-consecutive-br`](https://vizejs.dev/zh-CN/rules/html.html#html-no-consecutive-br) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-consecutive-br-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-consecutive-br-good) | 禁止连续的 &lt;br&gt; 元素 |
| [`html/no-dupe-style-properties`](https://vizejs.dev/zh-CN/rules/html.html#html-no-dupe-style-properties) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-dupe-style-properties-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-dupe-style-properties-good) | 禁止内联 style 属性中重复的属性声明 |
| [`html/no-duplicate-class`](https://vizejs.dev/zh-CN/rules/html.html#html-no-duplicate-class) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-duplicate-class-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-duplicate-class-good) | 禁止静态 class 属性中重复的类名 |
| [`html/no-duplicate-dt`](https://vizejs.dev/zh-CN/rules/html.html#html-no-duplicate-dt) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-duplicate-dt-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-duplicate-dt-good) | 禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称 |
| [`html/no-empty-palpable-content`](https://vizejs.dev/zh-CN/rules/html.html#html-no-empty-palpable-content) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-empty-palpable-content-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-no-empty-palpable-content-good) | 禁止预期具有可见内容的空元素 |
| [`html/require-datetime`](https://vizejs.dev/zh-CN/rules/html.html#html-require-datetime) | [错误示例](https://vizejs.dev/zh-CN/rules/html.html#html-require-datetime-bad) · [正确示例](https://vizejs.dev/zh-CN/rules/html.html#html-require-datetime-good) | 要求 &lt;time&gt; 元素具有 datetime 属性 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)
