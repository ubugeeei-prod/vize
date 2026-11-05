---
title: "HTML 规则"
description: "检查模板中 HTML 的有效性和语义化标记。"
---

# HTML 规则

检查模板中 HTML 的有效性和语义化标记。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 基础 (1)

`essential` 预设中的正确性规则。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | 禁止重复的元素 ID。 |  |

## 默认 (7)

由默认预设 `happy-path` 添加。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | 禁止已弃用的 HTML 属性。 |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | 禁止已弃用的 HTML 元素。 |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | 禁止连续的 &lt;br&gt; 元素。 |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | 禁止静态 class 属性中重复的类名。 |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | 禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称。 |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | 禁止预期具有可见内容的空元素。 |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | 要求 &lt;time&gt; 元素具有 datetime 属性。 |  |

## 更严格的预设 (1)

只在 `opinionated`、`nuxt` 或 `ecosystem` 中。具体是哪些见“预设”列。

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | 禁止内联 style 属性中重复的属性声明。 | `nuxt`, `opinionated` |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
