---
title: "HTML rules"
description: "HTML validity and semantic markup in templates."
---

# HTML rules

HTML validity and semantic markup in templates.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Essential (1)

Correctness rules in the `essential` preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | Disallow duplicate element IDs. |  |

## Default (7)

Added by `happy-path`, the default preset.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | Disallow deprecated HTML attributes. |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | Disallow deprecated HTML elements. |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | Disallow consecutive &lt;br&gt; elements. |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | Disallow duplicate class names in a static class attribute. |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | Disallow duplicate &lt;dt&gt; names in &lt;dl&gt;. |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | Disallow empty elements that expect visible content. |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | Require datetime attribute on &lt;time&gt; element. |  |

## Stricter presets (1)

Only in `opinionated`, `nuxt` or `ecosystem`. The Presets column lists which.

| Rule | What it checks | Presets | Auto-fix |
| --- | --- | --- | --- |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | Disallow duplicate properties in inline style attributes. | `nuxt`, `opinionated` |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
