---
title: "vize:croquis/cf/pinia-getter"
---

# `vize:croquis/cf/pinia-getter`

この契約の例では、setup で Pinia の getter から現在のプリミティブ値をコピーし、更新に追従しないスナップショットを作っています。追跡される計算やテンプレートで getter を直接読む場合はリアクティビティを保てます。この例では `storeToRefs()` を使って値を取り出します。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

Pinia のインストールを前提とし、`main.ts` がマウント前にプラグインを登録します。追跡される計算やテンプレートの中で `store.doubled` を直接読むことは有効です。ここでの問題は通常の値としてスナップショットを取ることであり、現在この契約の生成元はありません。

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

`main.ts`

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
createApp(App).use(createPinia()).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`counter-store.ts`

```ts
import { defineStore } from 'pinia';
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  getters: { doubled: state => state.count * 2 },
});

```

## 悪い

`const doubled = store.doubled` が setup 時点の getter の数値をコピーし、後の `store.count` 更新に追従しなくなります。

`App.vue`

```vue
<script setup lang="ts">
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const doubled = store.doubled;
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

## 良い

`storeToRefs(store)` からリアクティブな getter の ref を受け取り、分割代入やテンプレートでの自動アンラップ後もストアとのつながりを保ちます。

`App.vue`

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const { doubled } = storeToRefs(store);
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
