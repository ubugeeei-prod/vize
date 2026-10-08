---
title: エコシステム ルール
---

# エコシステム ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](./all.md#ecosystem-nuxt-prefer-nuxt-link) | [悪い例](./all.md#ecosystem-nuxt-prefer-nuxt-link-bad) · [良い例](./all.md#ecosystem-nuxt-prefer-nuxt-link-good) | Nuxt の内部リンクに NuxtLink を使います。 |
| [`ecosystem/pinia-prefer-store-to-refs`](./all.md#ecosystem-pinia-prefer-store-to-refs) | [悪い例](./all.md#ecosystem-pinia-prefer-store-to-refs-bad) · [良い例](./all.md#ecosystem-pinia-prefer-store-to-refs-good) | Pinia store の分割代入に storeToRefs() を使います。 |
| [`ecosystem/router-link-require-to`](./all.md#ecosystem-router-link-require-to) | [悪い例](./all.md#ecosystem-router-link-require-to-bad) · [良い例](./all.md#ecosystem-router-link-require-to-good) | RouterLink / NuxtLink に to を指定します。 |
| [`ecosystem/void-link-require-href`](./all.md#ecosystem-void-link-require-href) | [悪い例](./all.md#ecosystem-void-link-require-href-bad) · [良い例](./all.md#ecosystem-void-link-require-href-good) | Void Vue の Link に href を指定します。 |
| [`ecosystem/void-link-valid-method`](./all.md#ecosystem-void-link-valid-method) | [悪い例](./all.md#ecosystem-void-link-valid-method-bad) · [良い例](./all.md#ecosystem-void-link-valid-method-good) | Void Vue の Link に有効な静的 method を指定します。 |
| [`ecosystem/vue-i18n-no-missing-key`](./all.md#ecosystem-vue-i18n-no-missing-key) | [悪い例](./all.md#ecosystem-vue-i18n-no-missing-key-bad) · [良い例](./all.md#ecosystem-vue-i18n-no-missing-key-good) | SFC 内の翻訳データに存在しない静的キーを検出します。 |
| [`ecosystem/vue-router-prefer-named-link`](./all.md#ecosystem-vue-router-prefer-named-link) | [悪い例](./all.md#ecosystem-vue-router-prefer-named-link-bad) · [良い例](./all.md#ecosystem-vue-router-prefer-named-link-good) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 |
| [`ecosystem/vue-router-prefer-named-push`](./all.md#ecosystem-vue-router-prefer-named-push) | [悪い例](./all.md#ecosystem-vue-router-prefer-named-push-bad) · [良い例](./all.md#ecosystem-vue-router-prefer-named-push-good) | Vue Router のプログラムによる移動に名前付きルートを使います。 |
| [`ecosystem/vue-test-utils-no-html-snapshot`](./all.md#ecosystem-vue-test-utils-no-html-snapshot) | [悪い例](./all.md#ecosystem-vue-test-utils-no-html-snapshot-bad) · [良い例](./all.md#ecosystem-vue-test-utils-no-html-snapshot-good) | wrapper.html() 全体の snapshot に依存するテストを検出します。 |
| [`nuxt/no-nuxt-config-test-key`](./all.md#nuxt-no-nuxt-config-test-key) | [悪い例](./all.md#nuxt-no-nuxt-config-test-key-bad) · [良い例](./all.md#nuxt-no-nuxt-config-test-key-good) | Nuxt が自動判定する test 環境の手動設定を検出します。 |
| [`nuxt/no-page-meta-runtime-values`](./all.md#nuxt-no-page-meta-runtime-values) | [悪い例](./all.md#nuxt-no-page-meta-runtime-values-bad) · [良い例](./all.md#nuxt-no-page-meta-runtime-values-good) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 |
| [`nuxt/nuxt-config-keys-order`](./all.md#nuxt-nuxt-config-keys-order) | [悪い例](./all.md#nuxt-nuxt-config-keys-order-bad) · [良い例](./all.md#nuxt-nuxt-config-keys-order-good) | Nuxt 設定のプロパティを推奨順に並べます。 |
| [`nuxt/prefer-import-meta`](./all.md#nuxt-prefer-import-meta) | [悪い例](./all.md#nuxt-prefer-import-meta-bad) · [良い例](./all.md#nuxt-prefer-import-meta-good) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)

型付き Vue Router の四つの診断には、[完全なプロジェクトの例](./cross-file.md)を参照してください。
