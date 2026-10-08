---
title: HTML rules
---

# HTML rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. Individual pages are the reference for current support boundaries.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`html/deprecated-attr`](./reference/html-deprecated-attr.md) | [Bad](./reference/html-deprecated-attr.md#bad) · [Good](./reference/html-deprecated-attr.md#good) | Disallow deprecated HTML attributes |
| [`html/deprecated-element`](./reference/html-deprecated-element.md) | [Bad](./reference/html-deprecated-element.md#bad) · [Good](./reference/html-deprecated-element.md#good) | Disallow deprecated HTML elements |
| [`html/id-duplication`](./reference/html-id-duplication.md) | [Bad](./reference/html-id-duplication.md#bad) · [Good](./reference/html-id-duplication.md#good) | Disallow duplicate element IDs |
| [`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | [Bad](./reference/html-no-consecutive-br.md#bad) · [Good](./reference/html-no-consecutive-br.md#good) | Disallow consecutive &lt;br&gt; elements |
| [`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | [Bad](./reference/html-no-dupe-style-properties.md#bad) · [Good](./reference/html-no-dupe-style-properties.md#good) | Disallow duplicate properties in inline style attributes |
| [`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | [Bad](./reference/html-no-duplicate-class.md#bad) · [Good](./reference/html-no-duplicate-class.md#good) | Disallow duplicate class names in a static class attribute |
| [`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | [Bad](./reference/html-no-duplicate-dt.md#bad) · [Good](./reference/html-no-duplicate-dt.md#good) | Disallow duplicate &lt;dt&gt; names in &lt;dl&gt; |
| [`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | [Bad](./reference/html-no-empty-palpable-content.md#bad) · [Good](./reference/html-no-empty-palpable-content.md#good) | Disallow empty elements that expect visible content |
| [`html/require-datetime`](./reference/html-require-datetime.md) | [Bad](./reference/html-require-datetime.md#bad) · [Good](./reference/html-require-datetime.md#good) | Require datetime attribute on &lt;time&gt; element |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)

For nesting after components are composed, see [html/cross-component-nesting](./project/html-cross-component-nesting.md).
