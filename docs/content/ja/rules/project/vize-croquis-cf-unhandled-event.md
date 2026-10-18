---
title: "vize:croquis/cf/unhandled-event"
---

# `vize:croquis/cf/unhandled-event`

子が emit したイベントを、どの親も処理していません。

既定の重大度: info  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます

実験的な Rust CrossFileAnalyzer には、この診断コードの生成元があります。CLI の検査はこの個別コードを生成せず、ID を設定しても Rust 側の検査は有効になりません。以下の例は解析器が対応するグラフ構造や情報を説明するもので、Vite+ での検出を保証するものではありません。

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

`Child.vue` が `save` を emit しますが、直接の親である `Wrapper.vue` はそのイベントを購読していません。コンポーネントのイベントは、ラッパーを越えて自動的に親へ伝播しません。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

## 良い

`Wrapper.vue` が直接の子に `save` のリスナーを付けます。空のコールバックは、このルールでイベントを受け取る例を示すためのもので、保存処理の実装全体ではありません。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[ファイル間ルール一覧](../cross-file.md)
