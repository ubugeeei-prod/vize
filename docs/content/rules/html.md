---
title: HTML rules
---

# HTML rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`html/deprecated-attr`](./all.md#html-deprecated-attr) | [Bad](./all.md#html-deprecated-attr-bad) · [Good](./all.md#html-deprecated-attr-good) | Disallow deprecated HTML attributes |
| [`html/deprecated-element`](./all.md#html-deprecated-element) | [Bad](./all.md#html-deprecated-element-bad) · [Good](./all.md#html-deprecated-element-good) | Disallow deprecated HTML elements |
| [`html/id-duplication`](./all.md#html-id-duplication) | [Bad](./all.md#html-id-duplication-bad) · [Good](./all.md#html-id-duplication-good) | Disallow duplicate element IDs |
| [`html/no-consecutive-br`](./all.md#html-no-consecutive-br) | [Bad](./all.md#html-no-consecutive-br-bad) · [Good](./all.md#html-no-consecutive-br-good) | Disallow consecutive &lt;br&gt; elements |
| [`html/no-dupe-style-properties`](./all.md#html-no-dupe-style-properties) | [Bad](./all.md#html-no-dupe-style-properties-bad) · [Good](./all.md#html-no-dupe-style-properties-good) | Disallow duplicate properties in inline style attributes |
| [`html/no-duplicate-class`](./all.md#html-no-duplicate-class) | [Bad](./all.md#html-no-duplicate-class-bad) · [Good](./all.md#html-no-duplicate-class-good) | Disallow duplicate class names in a static class attribute |
| [`html/no-duplicate-dt`](./all.md#html-no-duplicate-dt) | [Bad](./all.md#html-no-duplicate-dt-bad) · [Good](./all.md#html-no-duplicate-dt-good) | Disallow duplicate &lt;dt&gt; names in &lt;dl&gt; |
| [`html/no-empty-palpable-content`](./all.md#html-no-empty-palpable-content) | [Bad](./all.md#html-no-empty-palpable-content-bad) · [Good](./all.md#html-no-empty-palpable-content-good) | Disallow empty elements that expect visible content |
| [`html/require-datetime`](./all.md#html-require-datetime) | [Bad](./all.md#html-require-datetime-bad) · [Good](./all.md#html-require-datetime-good) | Require datetime attribute on &lt;time&gt; element |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)

For nesting after components are composed, see [html/cross-component-nesting](./project/html-cross-component-nesting.md).
