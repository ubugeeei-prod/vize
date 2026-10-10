---
title: "Vue Rules: SFC Blocks"
---

# Vue Rules: SFC Blocks

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/no-preprocessor-lang`](https://vizejs.dev/rules/vue-sfc.html#vue-no-preprocessor-lang) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-no-preprocessor-lang-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-no-preprocessor-lang-good) | Discourage CSS preprocessor usage in favor of modern CSS |
| [`vue/no-script-non-standard-lang`](https://vizejs.dev/rules/vue-sfc.html#vue-no-script-non-standard-lang) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-no-script-non-standard-lang-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-no-script-non-standard-lang-good) | Discourage non-standard script lang values |
| [`vue/no-src-attribute`](https://vizejs.dev/rules/vue-sfc.html#vue-no-src-attribute) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-no-src-attribute-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-no-src-attribute-good) | Discourage src attribute on SFC blocks |
| [`vue/no-template-lang`](https://vizejs.dev/rules/vue-sfc.html#vue-no-template-lang) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-no-template-lang-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-no-template-lang-good) | Discourage lang attribute on template block |
| [`vue/require-scoped-style`](https://vizejs.dev/rules/vue-sfc.html#vue-require-scoped-style) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-require-scoped-style-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-require-scoped-style-good) | Require scoped attribute on style tags |
| [`vue/sfc-element-order`](https://vizejs.dev/rules/vue-sfc.html#vue-sfc-element-order) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-sfc-element-order-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-sfc-element-order-good) | Enforce consistent order of SFC top-level elements |
| [`vue/single-style-block`](https://vizejs.dev/rules/vue-sfc.html#vue-single-style-block) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-single-style-block-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-single-style-block-good) | Recommend having a single style block |
| [`vue/warn-custom-block`](https://vizejs.dev/rules/vue-sfc.html#vue-warn-custom-block) | [Bad](https://vizejs.dev/rules/vue-sfc.html#vue-warn-custom-block-bad) · [Good](https://vizejs.dev/rules/vue-sfc.html#vue-warn-custom-block-good) | Warn about custom blocks in SFC files |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
