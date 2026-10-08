---
title: "vize:croquis/cf/setup-context-violation"
---

# `vize:croquis/cf/setup-context-violation`

Vue が許さない使い方で setup コンテキストを使っています。

既定の重大度: context-dependent  
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

`ref(0)` を通常の script の module scope に作り、この analyzer が扱うインスタンスごとの setup 文脈から外しています。

`App.vue`

```vue
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

## 良い

script setup に移し、各インスタンスが count を持ち、テンプレートから読み取れる形にします。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[ファイル間ルール一覧](../cross-file.md)
