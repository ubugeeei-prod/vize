---
title: "vize:croquis/cf/provide-inject-type"
---

# `vize:croquis/cf/provide-inject-type`

provide した値と inject の型が一致しません。

| 既定の重大度 | 自動修正 | オプション |
| --- | --- | --- |
| warning | なし。関連ファイルを確認して修正してください | コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます |

適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報

この検査は提供元と使用側の明示的な型注釈を比較し、リテラルからの型推論は使いません。この例の提供元の `as string` 注釈を残してください。

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 悪い例

提供側の `title` は明示的な `string` ですが、子孫が同じキーを `inject<number>` で要求します。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<number>("title");
</script>
<template><p>Content</p></template>
```

## 良い例

使用側の型注釈を `inject<string>` にし、提供側の注釈と一致させます。この生成元は推論されたリテラル型ではなく明示的な注釈を比較するため、`as string` を残します。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
const title = inject<string>("title");
</script>
<template><p>Content</p></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: { preset: "incremental", crossFile: true,
    rules: { "croquis/cf/provide-inject-type": "warn" },
  },
});
```

```sh
vp run lint
```

## 関連ページ

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
