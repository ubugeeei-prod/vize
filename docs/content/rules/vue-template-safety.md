---
title: "Vue Rules: Template Safety"
---

# Vue Rules: Template Safety

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`vue/no-duplicate-attributes`](https://vizejs.dev/rules/vue-template-safety.html#vue-no-duplicate-attributes) | [Bad](https://vizejs.dev/rules/vue-template-safety.html#vue-no-duplicate-attributes-bad) · [Good](https://vizejs.dev/rules/vue-template-safety.html#vue-no-duplicate-attributes-good) | Disallow duplicate attributes on the same element |
| [`vue/no-textarea-mustache`](https://vizejs.dev/rules/vue-template-safety.html#vue-no-textarea-mustache) | [Bad](https://vizejs.dev/rules/vue-template-safety.html#vue-no-textarea-mustache-bad) · [Good](https://vizejs.dev/rules/vue-template-safety.html#vue-no-textarea-mustache-good) | Disallow mustache interpolation in `<textarea>` |
| [`vue/no-unsafe-url`](https://vizejs.dev/rules/vue-template-safety.html#vue-no-unsafe-url) | [Bad](https://vizejs.dev/rules/vue-template-safety.html#vue-no-unsafe-url-bad) · [Good](https://vizejs.dev/rules/vue-template-safety.html#vue-no-unsafe-url-good) | Warn about potentially unsafe URL bindings |
| [`vue/no-v-html`](https://vizejs.dev/rules/vue-template-safety.html#vue-no-v-html) | [Bad](https://vizejs.dev/rules/vue-template-safety.html#vue-no-v-html-bad) · [Good](https://vizejs.dev/rules/vue-template-safety.html#vue-no-v-html-good) | Warn against v-html to prevent XSS vulnerabilities |
| [`vue/no-v-text-v-html-on-component`](https://vizejs.dev/rules/vue-template-safety.html#vue-no-v-text-v-html-on-component) | [Bad](https://vizejs.dev/rules/vue-template-safety.html#vue-no-v-text-v-html-on-component-bad) · [Good](https://vizejs.dev/rules/vue-template-safety.html#vue-no-v-text-v-html-on-component-good) | Disallow v-text / v-html on component elements |
| [`vue/permitted-contents`](https://vizejs.dev/rules/vue-template-safety.html#vue-permitted-contents) | [Bad](https://vizejs.dev/rules/vue-template-safety.html#vue-permitted-contents-bad) · [Good](https://vizejs.dev/rules/vue-template-safety.html#vue-permitted-contents-good) | Enforce HTML content model rules |
| [`vue/use-unique-element-ids`](https://vizejs.dev/rules/vue-template-safety.html#vue-use-unique-element-ids) | [Bad](https://vizejs.dev/rules/vue-template-safety.html#vue-use-unique-element-ids-bad) · [Good](https://vizejs.dev/rules/vue-template-safety.html#vue-use-unique-element-ids-good) | Enforce unique element IDs using useId() instead of static literals |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
