---
title: "vize:croquis/cf/lifecycle-without-cleanup"
---

# `vize:croquis/cf/lifecycle-without-cleanup`

ライフサイクルフックが後片付けなしで処理を始めています。

既定の重大度: warning  
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

マウント時に window の resize リスナーを登録しますが、アンマウント時に同じコールバックを解除していません。

`App.vue`

```vue
<script setup lang="ts">
import { onMounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

## 良い

`onUnmounted` で、登録時と同じイベント名・関数の参照を使ってリスナーを解除します。

`App.vue`

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
onUnmounted(() => { window.removeEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[ファイル間ルール一覧](../cross-file.md)
