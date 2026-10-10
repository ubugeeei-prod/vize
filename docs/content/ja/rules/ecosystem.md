---
title: "エコシステム ルール"
---

# エコシステム ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-nuxt-prefer-nuxt-link-good) | Nuxt の内部リンクに NuxtLink を使います。 |
| [`ecosystem/pinia-prefer-store-to-refs`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-pinia-prefer-store-to-refs-good) | Pinia store の分割代入に storeToRefs() を使います。 |
| [`ecosystem/router-link-require-to`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-router-link-require-to) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-router-link-require-to-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-router-link-require-to-good) | RouterLink / NuxtLink に to を指定します。 |
| [`ecosystem/void-link-require-href`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-void-link-require-href) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-void-link-require-href-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-void-link-require-href-good) | Void Vue の Link に href を指定します。 |
| [`ecosystem/void-link-valid-method`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-void-link-valid-method) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-void-link-valid-method-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-void-link-valid-method-good) | Void Vue の Link に有効な静的 method を指定します。 |
| [`ecosystem/vue-i18n-no-missing-key`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-i18n-no-missing-key-good) | SFC 内の翻訳データに存在しない静的キーを検出します。 |
| [`ecosystem/vue-router-prefer-named-link`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-router-prefer-named-link-good) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 |
| [`ecosystem/vue-router-prefer-named-push`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-router-prefer-named-push-good) | Vue Router のプログラムによる移動に名前付きルートを使います。 |
| [`ecosystem/vue-test-utils-no-html-snapshot`](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#ecosystem-vue-test-utils-no-html-snapshot-good) | wrapper.html() 全体の snapshot に依存するテストを検出します。 |
| [`nuxt/no-nuxt-config-test-key`](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-no-nuxt-config-test-key) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-no-nuxt-config-test-key-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-no-nuxt-config-test-key-good) | Nuxt が自動判定する test 環境の手動設定を検出します。 |
| [`nuxt/no-page-meta-runtime-values`](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-no-page-meta-runtime-values) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-no-page-meta-runtime-values-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-no-page-meta-runtime-values-good) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 |
| [`nuxt/nuxt-config-keys-order`](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-nuxt-config-keys-order) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-nuxt-config-keys-order-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-nuxt-config-keys-order-good) | Nuxt 設定のプロパティを推奨順に並べます。 |
| [`nuxt/prefer-import-meta`](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-prefer-import-meta) | [悪い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-prefer-import-meta-bad) · [良い](https://vizejs.dev/ja/rules/ecosystem.html#nuxt-prefer-import-meta-good) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
