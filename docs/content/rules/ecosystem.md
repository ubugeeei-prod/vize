---
title: Ecosystem rules
---

# Ecosystem rules

Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.

Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.

| Rule | Examples | Purpose |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](./all.md#ecosystem-nuxt-prefer-nuxt-link) | [Bad](./all.md#ecosystem-nuxt-prefer-nuxt-link-bad) · [Good](./all.md#ecosystem-nuxt-prefer-nuxt-link-good) | Prefer NuxtLink for internal application links |
| [`ecosystem/pinia-prefer-store-to-refs`](./all.md#ecosystem-pinia-prefer-store-to-refs) | [Bad](./all.md#ecosystem-pinia-prefer-store-to-refs-bad) · [Good](./all.md#ecosystem-pinia-prefer-store-to-refs-good) | Prefer storeToRefs() when destructuring Pinia stores |
| [`ecosystem/router-link-require-to`](./all.md#ecosystem-router-link-require-to) | [Bad](./all.md#ecosystem-router-link-require-to-bad) · [Good](./all.md#ecosystem-router-link-require-to-good) | Require a `to` target on RouterLink and NuxtLink components |
| [`ecosystem/void-link-require-href`](./all.md#ecosystem-void-link-require-href) | [Bad](./all.md#ecosystem-void-link-require-href-bad) · [Good](./all.md#ecosystem-void-link-require-href-good) | Require `href` on Void Vue Link components |
| [`ecosystem/void-link-valid-method`](./all.md#ecosystem-void-link-valid-method) | [Bad](./all.md#ecosystem-void-link-valid-method-bad) · [Good](./all.md#ecosystem-void-link-valid-method-good) | Validate static Void Vue Link method props |
| [`ecosystem/vue-i18n-no-missing-key`](./all.md#ecosystem-vue-i18n-no-missing-key) | [Bad](./all.md#ecosystem-vue-i18n-no-missing-key-bad) · [Good](./all.md#ecosystem-vue-i18n-no-missing-key-good) | Report static vue-i18n keys that are absent from local SFC messages |
| [`ecosystem/vue-router-prefer-named-link`](./all.md#ecosystem-vue-router-prefer-named-link) | [Bad](./all.md#ecosystem-vue-router-prefer-named-link-bad) · [Good](./all.md#ecosystem-vue-router-prefer-named-link-good) | Prefer named route objects over static path strings in RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](./all.md#ecosystem-vue-router-prefer-named-push) | [Bad](./all.md#ecosystem-vue-router-prefer-named-push-bad) · [Good](./all.md#ecosystem-vue-router-prefer-named-push-good) | Prefer named route objects for Vue Router programmatic navigation |
| [`ecosystem/vue-test-utils-no-html-snapshot`](./all.md#ecosystem-vue-test-utils-no-html-snapshot) | [Bad](./all.md#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Good](./all.md#ecosystem-vue-test-utils-no-html-snapshot-good) | Avoid snapshotting wrapper.html() in Vue Test Utils tests |
| [`nuxt/no-nuxt-config-test-key`](./all.md#nuxt-no-nuxt-config-test-key) | [Bad](./all.md#nuxt-no-nuxt-config-test-key-bad) · [Good](./all.md#nuxt-no-nuxt-config-test-key-good) | Disallow setting `test` key in Nuxt config |
| [`nuxt/no-page-meta-runtime-values`](./all.md#nuxt-no-page-meta-runtime-values) | [Bad](./all.md#nuxt-no-page-meta-runtime-values-bad) · [Good](./all.md#nuxt-no-page-meta-runtime-values-good) | Disallow runtime context values inside `definePageMeta` at the eager level, which is extracted into a separate chunk at build time and runs before component setup |
| [`nuxt/nuxt-config-keys-order`](./all.md#nuxt-nuxt-config-keys-order) | [Bad](./all.md#nuxt-nuxt-config-keys-order-bad) · [Good](./all.md#nuxt-nuxt-config-keys-order-good) | Prefer recommended order of Nuxt config properties |
| [`nuxt/prefer-import-meta`](./all.md#nuxt-prefer-import-meta) | [Bad](./all.md#nuxt-prefer-import-meta-bad) · [Good](./all.md#nuxt-prefer-import-meta-good) | Prefer using `import.meta.*` over `process.*` |

[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)

For the four typed Vue Router diagnostics, see the [complete project examples](./cross-file.md).
