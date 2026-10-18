---
title: "vize:croquis/cf/reactive-export"
---

# `vize:croquis/cf/reactive-export`

reactive な状態がモジュールから export されています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

共有するアプリケーションの store を意図する場合、リアクティブな状態の export は有効です。この例は状態の分離を必要とするもので、あらゆる export が不正という意味ではありません。現在この契約の生成元はありません。

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

## 悪い

モジュールが初期化済みのリアクティブなオブジェクトを一つだけ公開するため、それを import する側はすべて同じ count を受け取ります。リクエスト間でモジュールを共有する SSR では、この例で必要とする、インスタンスやリクエストごとの状態の分離を保てません。

`state.ts`

```ts
import { reactive } from 'vue';
export const state = reactive({ count: 0 });

```

`App.vue`

```vue
<script setup lang="ts">
import { state } from './state';
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

## 良い

モジュールから状態を作る関数を公開し、App が setup 内で呼びます。共有する一つのインスタンスを公開する形から、コンポーネントのインスタンスごとに新しいリアクティブな `count` を作る形に変更します。

`state.ts`

```ts
import { reactive } from 'vue';
export function createState() { return reactive({ count: 0 }); }

```

`App.vue`

```vue
<script setup lang="ts">
import { createState } from './state';
const state = createState();
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
