---
title: エコシステム ルール
---

# エコシステム ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 目的 |
| --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Nuxt の内部リンクに NuxtLink を使います。 |
| [`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Pinia store の分割代入に storeToRefs() を使います。 |
| [`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | RouterLink / NuxtLink に to を指定します。 |
| [`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Void Vue の Link に href を指定します。 |
| [`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Void Vue の Link に有効な静的 method を指定します。 |
| [`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | SFC 内の翻訳データに存在しない静的キーを検出します。 |
| [`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 |
| [`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Vue Router のプログラムによる移動に名前付きルートを使います。 |
| [`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | wrapper.html() 全体の snapshot に依存するテストを検出します。 |
| [`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Nuxt が自動判定する test 環境の手動設定を検出します。 |
| [`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 |
| [`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Nuxt 設定のプロパティを推奨順に並べます。 |
| [`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)

型付き Vue Router の四つの診断には、[完全なプロジェクトの例](./cross-file.md)を参照してください。
