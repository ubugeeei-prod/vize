---
title: "Ecosystem rules"
---

# Ecosystem rules

Every rule on this page includes its purpose, prerequisites, configuration, and complete Bad/Good examples. Highlighted lines show the changes; copied code keeps the complete source. Each packet states its current support limits.


| Rule | Examples | Purpose |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](https://vizejs.dev/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link-good) | Prefer NuxtLink for internal application links |
| [`ecosystem/pinia-prefer-store-to-refs`](https://vizejs.dev/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs-good) | Prefer storeToRefs() when destructuring Pinia stores |
| [`ecosystem/router-link-require-to`](https://vizejs.dev/rules/ecosystem.html#ecosystem-router-link-require-to) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-router-link-require-to-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-router-link-require-to-good) | Require a `to` target on RouterLink and NuxtLink components |
| [`ecosystem/void-link-require-href`](https://vizejs.dev/rules/ecosystem.html#ecosystem-void-link-require-href) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-void-link-require-href-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-void-link-require-href-good) | Require `href` on Void Vue Link components |
| [`ecosystem/void-link-valid-method`](https://vizejs.dev/rules/ecosystem.html#ecosystem-void-link-valid-method) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-void-link-valid-method-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-void-link-valid-method-good) | Validate static Void Vue Link method props |
| [`ecosystem/vue-i18n-no-missing-key`](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key-good) | Report static vue-i18n keys that are absent from local SFC messages |
| [`ecosystem/vue-router-prefer-named-link`](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link-good) | Prefer named route objects over static path strings in RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push-good) | Prefer named route objects for Vue Router programmatic navigation |
| [`ecosystem/vue-test-utils-no-html-snapshot`](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot) | [Bad](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot-good) | Avoid snapshotting wrapper.html() in Vue Test Utils tests |
| [`nuxt/no-nuxt-config-test-key`](https://vizejs.dev/rules/ecosystem.html#nuxt-no-nuxt-config-test-key) | [Bad](https://vizejs.dev/rules/ecosystem.html#nuxt-no-nuxt-config-test-key-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#nuxt-no-nuxt-config-test-key-good) | Disallow setting `test` key in Nuxt config |
| [`nuxt/no-page-meta-runtime-values`](https://vizejs.dev/rules/ecosystem.html#nuxt-no-page-meta-runtime-values) | [Bad](https://vizejs.dev/rules/ecosystem.html#nuxt-no-page-meta-runtime-values-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#nuxt-no-page-meta-runtime-values-good) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup |
| [`nuxt/nuxt-config-keys-order`](https://vizejs.dev/rules/ecosystem.html#nuxt-nuxt-config-keys-order) | [Bad](https://vizejs.dev/rules/ecosystem.html#nuxt-nuxt-config-keys-order-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#nuxt-nuxt-config-keys-order-good) | Prefer recommended order of Nuxt config properties |
| [`nuxt/prefer-import-meta`](https://vizejs.dev/rules/ecosystem.html#nuxt-prefer-import-meta) | [Bad](https://vizejs.dev/rules/ecosystem.html#nuxt-prefer-import-meta-bad) · [Good](https://vizejs.dev/rules/ecosystem.html#nuxt-prefer-import-meta-good) | Prefer using `import.meta.*` over `process.*` |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)
