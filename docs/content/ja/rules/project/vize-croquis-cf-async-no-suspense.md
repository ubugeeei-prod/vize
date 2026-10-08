---
title: "vize:croquis/cf/async-no-suspense"
---

# `vize:croquis/cf/async-no-suspense`

async コンポーネントが Suspense 境界なしで描画されています。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

Current support: `no-source-async-fact`

生成元は macros.is_async() を読みますが、現在のソース解析は top-level await を script-setup の scope に記録します。そのため以下の完全な悪い例・良い例では、現在の CLI は async-no-suspense を検出しません。Suspense の使い方を説明する例で、必要な macro 情報を渡す処理は今後の実装課題です。

## 共通のプロジェクト ファイル

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

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

子に top-level await があり、親に `<Suspense>` がありません。現在のソース解析には、このコードの生成に必要な macro 情報が渡されません。

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

同じ async な子を `<Suspense>` と loading fallback で包みます。規約の例であり、現在の検査は両方のソースでこのコードを生成しません。

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

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[ファイル間ルール一覧](../cross-file.md)
