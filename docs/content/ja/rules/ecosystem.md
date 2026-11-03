---
title: "エコシステムと Nuxt のルール"
description: "Nuxt、Vue Router、Pinia、Vue I18n、Vue Test Utils、Void Vue を使うプロジェクト向けの検査です。"
---

# エコシステムと Nuxt のルール

Nuxt、Vue Router、Pinia、Vue I18n、Vue Test Utils、Void Vue を使うプロジェクト向けの検査です。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## より厳しいプリセット (13)

`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="ecosystem-nuxt-prefer-nuxt-link"></span>[`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | Nuxt の内部リンクに NuxtLink を使います。 | `nuxt` |  |
| <span id="ecosystem-pinia-prefer-store-to-refs"></span>[`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | Pinia store の分割代入に storeToRefs() を使います。 | `ecosystem` |  |
| <span id="ecosystem-router-link-require-to"></span>[`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | RouterLink / NuxtLink に to を指定します。 | `ecosystem` |  |
| <span id="ecosystem-void-link-require-href"></span>[`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | Void Vue の Link に href を指定します。 | `ecosystem` |  |
| <span id="ecosystem-void-link-valid-method"></span>[`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | Void Vue の Link に有効な静的 method を指定します。 | `ecosystem` |  |
| <span id="ecosystem-vue-i18n-no-missing-key"></span>[`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | SFC 内の翻訳データに存在しない静的キーを検出します。 | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-link"></span>[`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-push"></span>[`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Vue Router のプログラムによる移動に名前付きルートを使います。 | `ecosystem` |  |
| <span id="ecosystem-vue-test-utils-no-html-snapshot"></span>[`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | wrapper.html() 全体のスナップショットに依存するテストを検出します。 | `ecosystem` |  |
| <span id="nuxt-no-nuxt-config-test-key"></span>[`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | Nuxt 設定の test キーに旧形式の真偽値を指定する箇所を検出します。 | `nuxt` |  |
| <span id="nuxt-no-page-meta-runtime-values"></span>[`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 | `nuxt` |  |
| <span id="nuxt-nuxt-config-keys-order"></span>[`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | Nuxt 設定のプロパティを推奨順に並べます。 | `nuxt` | ✓ |
| <span id="nuxt-prefer-import-meta"></span>[`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 | `nuxt` | ✓ |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
