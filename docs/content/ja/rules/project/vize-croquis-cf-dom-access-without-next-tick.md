---
title: "vize:croquis/cf/dom-access-without-next-tick"
---

# `vize:croquis/cf/dom-access-without-next-tick`

Vue が更新を反映する前に DOM を読んでいます。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

## 共通のプロジェクト ファイル

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

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

`read-label.ts`

```ts
export function readLabel(node: HTMLElement | null): string {
  return node?.textContent ?? '';
}

```

## 悪い

クリック処理が `count` を増やしてすぐ段落の DOM を読み、Vue による更新の反映を待っていません。`sampled` に前の count が入る可能性があります。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
function increment() {
  count.value++;
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

## 良い

状態を書いた後に `nextTick()` を待ち、Vue が段落を更新してから `readLabel` でテキストを読みます。

`App.vue`

```vue
<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
async function increment() {
  count.value++;
  await nextTick();
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
