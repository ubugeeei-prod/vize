---
title: "Musea rules"
description: "Musea `.art.vue` stories, variants and design-token usage."
---

# Musea rules

Musea `.art.vue` stories, variants and design-token usage.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Opt-in (6)

Not in any preset. Turn them on by name in `vue.linter.rules`.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | Disallow empty &lt;variant&gt; blocks. |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | Prefer design token CSS variables over hardcoded primitive values. |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | Require component attribute in &lt;art&gt; block. |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | Require title attribute in &lt;art&gt; block. |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | Require unique variant names. |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | Require name attribute in &lt;variant&gt; blocks. |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
