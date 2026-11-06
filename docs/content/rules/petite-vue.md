---
title: "petite-vue rules"
description: "Directives and scopes in HTML pages that use petite-vue."
---

# petite-vue rules

Directives and scopes in HTML pages that use petite-vue.

Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.

## Opt-in (3)

Not in any preset. Turn them on by name in the `linter.rules` passed to `withVue`.

| Rule | What it checks | Auto-fix |
| --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | Disallow directives that petite-vue does not support. |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | Require v-effect to have a non-empty expression. |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | Require v-scope to bind an object literal. |  |

[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)
