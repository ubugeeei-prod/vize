---
title: "vize:croquis/cf/unused-attrs"
---

# `vize:croquis/cf/unused-attrs`

複数ルートのコンポーネントに渡した fallthrough 属性が使われていません。

既定の重大度: info  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

実験的な Rust CrossFileAnalyzer にはこのコードの生成元があります。CLI はこの個別コードを生成しません。ID を設定しても Rust 側の検査は有効になりません。例は analyzer の対象となる構成や情報を示し、Vite+ での検出を約束するものではありません。

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

親の `tracking-code` は prop として使われず、複数ルートの子から転送もされません。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue
<template><main>Content</main><aside>Help</aside></template>
```

## 良い

`<main>` に `$attrs` を binding し、継承属性の適用先を明示します。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[ファイル間ルール一覧](../cross-file.md)
