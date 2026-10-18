---
title: "vize:croquis/cf/array-mutation"
---

# `vize:croquis/cf/array-mutation`

現在の生成元がないこの契約では、旧 Vue 2.7 の配列のインデックス代入を例にしています。Vue 2 は直接のインデックス代入による更新を通知しませんが、Vue 3 はその代入を追跡します。悪い例・良い例は Vue 2 の修正を示すもので、Vue 3 の不具合を示す例ではありません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

旧 Vue 2.7 に限る例です。対応する Vue 2.7 と SFC コンパイラーを使うことを前提とします。Vue 3 の Proxy は配列のインデックス代入を追跡するため、`items[0] = next` は Vue 3 ではリアクティブであり、不具合ではありません。この公開コードには現在の生成元がありません。

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

## 悪い

この旧 Vue 2.7 プロジェクトでは、`items[0] = next` が配列の変更を監視する Vue 2 の仕組みに通知せず配列を変更するため、表示中の最初の要素が更新されない場合があります。

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

## 良い

`splice(0, 1, next)` で Vue 2 が監視する配列の変更メソッドを使い、同じ置換を画面に反映します。

`replace-first.ts`

```ts
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
