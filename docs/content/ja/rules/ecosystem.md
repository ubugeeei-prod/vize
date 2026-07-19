---
title: 生態系のルール
---

<!-- Generated translation; source: rules/ecosystem.md -->

# 生態系ルール

これらのルールは、Nuxt、Vue Router、Pinia、vue-i18n、Vue Test Utils、および Void Vue に関する規則をカバーしています。

エコシステム ルールは、`ecosystem` プリセットによって有効になります。ホストは、使用時に名前で有効にすることもできます。
`incremental`;これらは、`happy-path`、`nuxt`、または `opinionated` の一部ではありません。

LSP でエディター エコシステム ヘルパーが有効になっている場合、Vize は Vue Router ルート名も追加します
完了、ファイルルートパラメータの完了および`useRoute().params`の診断、Vue I18nキー
補完、ワークスペース JSON キー検証、静的 `t()` / `$t()` 呼び出しのインレイ プレビュー。

## `ecosystem/router-link-require-to`

`<RouterLink>`、`<router-link>`、`<NuxtLink>`、および `<nuxt-link>` では、`to` または `:to` が必要です。

デフォルトの重大度: `error`
プリセット: `ecosystem`

悪い：

```vue
<template>
  <RouterLink>Settings</RouterLink>
</template>
```

良い：

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-link`

RouterLink のようなコンポーネントの静的な内部パス文字列について警告します。名前付きルート オブジェクトは Vue を保持します
ルーター型のルートと、ルート名とパラメーターを中心としたエディターの補完。

デフォルトの重大度: `warning`
プリセット: `ecosystem`

悪い：

```vue
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

良い：

```vue
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

## `ecosystem/vue-router-prefer-named-push`

`router.push("/path")`、`router.replace("/path")`、および静的 `path` を持つルート オブジェクトについて警告します。

デフォルトの重大度: `warning`
プリセット: `ecosystem`

悪い：

```ts
router.push("/settings");
```

良い：

```ts
router.push({ name: "settings" });
```

## `ecosystem/nuxt-prefer-nuxt-link`

Nuxt 指向のコード内の内部 `<a href="/...">` リンクについて警告します。外部リンク、ダウンロード、および
`target="_blank"` はプレーンアンカーのままです。

デフォルトの重大度: `warning`
プリセット: `ecosystem`

悪い：

```vue
<template>
  <a href="/settings">Settings</a>
</template>
```

良い：

```vue
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## `ecosystem/pinia-prefer-store-to-refs`

Pinia ストアが直接構造化されると警告します。状態とゲッターには `storeToRefs()` を使用し、
ストア インスタンスでアクションを保持します。

デフォルトの重大度: `warning`
プリセット: `ecosystem`

悪い：

```ts
const { name } = useUserStore();
```

良い：

```ts
const store = useUserStore();
const { name } = storeToRefs(store);
```

## `ecosystem/vue-i18n-no-missing-key`

静的 `$t()`、`$te()`、`$tm()`、`t()`、`te()`、または `tm()` キーが同じキーに見つからない場合に警告します。
SFC のローカル `<i18n lang="json">` ブロック。

デフォルトの重大度: `warning`
プリセット: `ecosystem`

悪い：

```vue
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

良い：

```vue
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## `ecosystem/void-link-require-href`

`@void/vue` からインポートされた Void Vue `<Link>` コンポーネントには、`href` または `:href` が必要です。

デフォルトの重大度: `error`
プリセット: `ecosystem`

悪い：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

良い：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

## `ecosystem/void-link-valid-method`

不明な静的 Void Vue `<Link method>` 値と、`prefetch` などの GET 専用プロパティについて警告します。
リンクが変更方法を使用する場合は、`reloadDocument`。

デフォルトの重大度: `warning`
プリセット: `ecosystem`

悪い：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

良い：

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

## `ecosystem/vue-test-utils-no-html-snapshot`

`expect(wrapper.html()).toMatchSnapshot()` で警告します。目に見えるテキストの周りに焦点を当てたアサーションを好みます。
属性、発行されたイベント、またはコンポーネントの状態。

デフォルトの重大度: `warning`
プリセット: `ecosystem`

悪い：

```ts
expect(wrapper.html()).toMatchSnapshot();
```

良い：

```ts
expect(wrapper.text()).toContain("Saved");
```
