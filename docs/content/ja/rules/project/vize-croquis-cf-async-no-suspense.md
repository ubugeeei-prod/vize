---
title: "vize:croquis/cf/async-no-suspense"
---

# `vize:croquis/cf/async-no-suspense`

async コンポーネントが Suspense 境界なしで描画されています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます

実験的な Rust CrossFileAnalyzer には、この診断コードの生成元があります。CLI の検査はこの個別コードを生成せず、ID を設定しても Rust 側の検査は有効になりません。以下の例は解析器が対応するグラフ構造や情報を説明するもので、Vite+ での検出を保証するものではありません。

現在の対応状況: `no-source-async-fact`

生成元は `macros.is_async()` を読みますが、現在のソース解析はトップレベルの `await` を script setup のスコープに記録します。そのため、以下の完全な悪い例・良い例のどちらからも、現在の CLI は `async-no-suspense` を生成しません。これは Suspense の使い方を説明する例であり、必要なマクロ情報を渡す処理は今後の実装課題です。

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

## 悪い

子にトップレベルの `await` がありますが、親には `<Suspense>` の境界がありません。現在のソース解析は、この診断コードの生成に必要なマクロ情報を渡しません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

## 良い

親は同じ非同期の子を `<Suspense>` で包み、読み込み中の代替表示を用意します。これは使い方を示す例であり、現在の検査は悪い例・良い例のどちらからもこの診断コードを生成しません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[ファイル間ルール一覧](../cross-file.md)
