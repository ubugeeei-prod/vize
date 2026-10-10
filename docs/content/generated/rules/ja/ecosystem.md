---
title: "エコシステム ルール"
---

# エコシステム ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [悪い](#ecosystem-nuxt-prefer-nuxt-link-bad) · [良い](#ecosystem-nuxt-prefer-nuxt-link-good) | Nuxt の内部リンクに NuxtLink を使います。 |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [悪い](#ecosystem-pinia-prefer-store-to-refs-bad) · [良い](#ecosystem-pinia-prefer-store-to-refs-good) | Pinia store の分割代入に storeToRefs() を使います。 |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [悪い](#ecosystem-router-link-require-to-bad) · [良い](#ecosystem-router-link-require-to-good) | RouterLink / NuxtLink に to を指定します。 |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [悪い](#ecosystem-void-link-require-href-bad) · [良い](#ecosystem-void-link-require-href-good) | Void Vue の Link に href を指定します。 |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [悪い](#ecosystem-void-link-valid-method-bad) · [良い](#ecosystem-void-link-valid-method-good) | Void Vue の Link に有効な静的 method を指定します。 |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [悪い](#ecosystem-vue-i18n-no-missing-key-bad) · [良い](#ecosystem-vue-i18n-no-missing-key-good) | SFC 内の翻訳データに存在しない静的キーを検出します。 |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [悪い](#ecosystem-vue-router-prefer-named-link-bad) · [良い](#ecosystem-vue-router-prefer-named-link-good) | RouterLink の文字列パスを名前付きルートの指定に置き換えます。 |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [悪い](#ecosystem-vue-router-prefer-named-push-bad) · [良い](#ecosystem-vue-router-prefer-named-push-good) | Vue Router のプログラムによる移動に名前付きルートを使います。 |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [悪い](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [良い](#ecosystem-vue-test-utils-no-html-snapshot-good) | wrapper.html() 全体の snapshot に依存するテストを検出します。 |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [悪い](#nuxt-no-nuxt-config-test-key-bad) · [良い](#nuxt-no-nuxt-config-test-key-good) | Nuxt が自動判定する test 環境の手動設定を検出します。 |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [悪い](#nuxt-no-page-meta-runtime-values-bad) · [良い](#nuxt-no-page-meta-runtime-values-good) | definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。 |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [悪い](#nuxt-nuxt-config-keys-order-bad) · [良い](#nuxt-nuxt-config-keys-order-good) | Nuxt 設定のプロパティを推奨順に並べます。 |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [悪い](#nuxt-prefer-import-meta-bad) · [良い](#nuxt-prefer-import-meta-good) | Nuxt の環境フラグを process.* から import.meta.* に置き換えます。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Nuxt の内部リンクに NuxtLink を使います。

[悪い例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [良い例](#ecosystem-nuxt-prefer-nuxt-link-good)

既定の重大度: `warning`  
プリセット: `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**悪い**

Nuxt の内部ページへの移動に通常の a を使っています。

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**良い**

同じ移動先を NuxtLink に指定し、Nuxt のルーターを使います。

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [全ルール](all.md)

### `ecosystem/pinia-prefer-store-to-refs`

Pinia store の分割代入に storeToRefs() を使います。

[悪い例](#ecosystem-pinia-prefer-store-to-refs-bad) · [良い例](#ecosystem-pinia-prefer-store-to-refs-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**悪い**

store から name を直接分割代入し、リアクティブな store の読み取りから値を切り離しています。

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**良い**

store は保持し、storeToRefs で name のリアクティブな参照を取り出します。

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [全ルール](all.md)

### `ecosystem/router-link-require-to`

RouterLink / NuxtLink に to を指定します。

[悪い例](#ecosystem-router-link-require-to-bad) · [良い例](#ecosystem-router-link-require-to-good)

既定の重大度: `error`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

SFC の単一ルートにあるリンクは、親から属性を継承できるため対象外になる場合があります。この例は明示的な遷移先が必要な内部のリンクです。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**悪い**

nav の内側の RouterLink に to がなく、ルートの属性継承にも頼れません。

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**良い**

内部のリンクに `to="/settings"` で移動先を明示します。

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [全ルール](all.md)

### `ecosystem/void-link-require-href`

Void Vue の Link に href を指定します。

[悪い例](#ecosystem-void-link-require-href-bad) · [良い例](#ecosystem-void-link-require-href-good)

既定の重大度: `error`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**悪い**

@void/vue から import した Link に href がありません。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**良い**

同じ Link の href に設定画面の移動先を指定します。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [全ルール](all.md)

### `ecosystem/void-link-valid-method`

Void Vue の Link に有効な静的 method を指定します。

[悪い例](#ecosystem-void-link-valid-method-bad) · [良い例](#ecosystem-void-link-valid-method-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**悪い**

DELETE の操作に、ページ移動のリクエスト向けの prefetch を指定しています。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**良い**

prefetch を取り除き、DELETE のリクエストを事前取得しない形にします。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [全ルール](all.md)

### `ecosystem/vue-i18n-no-missing-key`

SFC 内の翻訳データに存在しない静的キーを検出します。

[悪い例](#ecosystem-vue-i18n-no-missing-key-bad) · [良い例](#ecosystem-vue-i18n-no-missing-key-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**悪い**

テンプレートは auth.missing を参照しますが、ローカルの英語メッセージには auth.login しかありません。

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**良い**

ローカルのメッセージにある auth.login を参照します。

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [全ルール](all.md)

### `ecosystem/vue-router-prefer-named-link`

RouterLink の文字列パスを名前付きルートの指定に置き換えます。

[悪い例](#ecosystem-vue-router-prefer-named-link-bad) · [良い例](#ecosystem-vue-router-prefer-named-link-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**悪い**

RouterLink の移動先をルート名ではなく、文字列のパスで指定しています。

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**良い**

バインドする route のオブジェクトに settings の name を指定します。

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [全ルール](all.md)

### `ecosystem/vue-router-prefer-named-push`

Vue Router のプログラムによる移動に名前付きルートを使います。

[悪い例](#ecosystem-vue-router-prefer-named-push-bad) · [良い例](#ecosystem-vue-router-prefer-named-push-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**悪い**

router.push に、現在の URL 表記に結び付く文字列のパスを渡しています。

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**良い**

settings のルート名を持つオブジェクトを router.push に渡します。

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [全ルール](all.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

wrapper.html() 全体の snapshot に依存するテストを検出します。

[悪い例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [良い例](#ecosystem-vue-test-utils-no-html-snapshot-good)

既定の重大度: `warning`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**悪い**

期待する振る舞いを検査する代わりに、wrapper の HTML 全体を snapshot にしています。

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**良い**

表示された文字に Saved が含まれることを直接検査します。

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [全ルール](all.md)

### `nuxt/no-nuxt-config-test-key`

Nuxt が自動判定する test 環境の手動設定を検出します。

[悪い例](#nuxt-no-nuxt-config-test-key-bad) · [良い例](#nuxt-no-nuxt-config-test-key-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Nuxt 設定ファイル（nuxt.config.ts）  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**悪い**

エクスポートした Nuxt 設定で `test` キーに真偽値 `true` を指定しており、このルールが拒否する旧形式の設定です。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**良い**

空の設定にすることで、真偽値の `test` プロパティを取り除きます。テスト設定のオブジェクトまで禁止する例ではありません。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [全ルール](all.md)

### `nuxt/no-page-meta-runtime-values`

definePageMeta の即時評価部分で実行時コンテキストを使う箇所を検出します。

[悪い例](#nuxt-no-page-meta-runtime-values-bad) · [良い例](#nuxt-no-page-meta-runtime-values-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**悪い**

`definePageMeta` のオブジェクトを作る際に `useRoute()` を即座に評価しています。メタデータはマクロによって setup の実行時コンテキストの外へ巻き上げられます。

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**良い**

`validate` にコールバックを渡し、`useRoute().params.id` の評価をその実行時まで遅らせます。このルールは関数本体内の遅延評価と、メタデータの即時評価を区別します。

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [全ルール](all.md)

### `nuxt/nuxt-config-keys-order`

Nuxt 設定のプロパティを推奨順に並べます。

[悪い例](#nuxt-nuxt-config-keys-order-bad) · [良い例](#nuxt-nuxt-config-keys-order-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: 対応する検出で利用可能  
適用範囲: Nuxt 設定ファイル（nuxt.config.ts）  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**悪い**

設定内で `ssr` が `modules` より先に置かれ、このルールが推奨する Nuxt 設定キーの順序が逆になっています。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**良い**

`modules` を `ssr` より前に移し、各値を保ったまま指定の順序にそろえます。変更するのは配置であり、設定値の意味ではありません。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [全ルール](all.md)

### `nuxt/prefer-import-meta`

Nuxt の環境フラグを process.* から import.meta.* に置き換えます。

[悪い例](#nuxt-prefer-import-meta-bad) · [良い例](#nuxt-prefer-import-meta-good)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**悪い**

`process.client` は旧形式の Nuxt 環境フラグであり、このルールは `import.meta` への移行を求めます。

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**良い**

`import.meta.client` に置き換え、ブラウザー側だけで実行する分岐を新しい環境フラグで表します。

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [全ルール](all.md)
