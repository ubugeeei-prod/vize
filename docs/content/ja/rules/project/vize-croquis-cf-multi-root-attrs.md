---
title: "vize:croquis/cf/multi-root-attrs"
---

# `vize:croquis/cf/multi-root-attrs`

複数ルートのコンポーネントが属性を受け取りますが、付ける場所がありません。

既定の重大度: warning  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

## 共通のプロジェクト ファイル

以下のファイルは悪い例・良い例で共通です。Vue を、Router の例では vue-router もインストールしてください。エントリー ファイルでコンポーネントの関係を明確にしています。

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

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

## 良い

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

良い例はこの検出を避ける修正です。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[ファイル間ルール一覧](../cross-file.md)
