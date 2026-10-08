---
title: "vize:croquis/cf/object-identity-comparison"
---

# `vize:croquis/cf/object-identity-comparison`

reactive オブジェクトを同一性で比較していますが、unwrap のたびに同一性は変わります。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

ID がレコードを一意に識別することを前提とします。同じリアクティブ Proxy の参照同士を比較することは有効であり、この契約には現在の生成元がありません。

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

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

## 悪い

`proxy === raw` はラッパーの同一性を比較するため、同じユーザーを表していても false になります。アプリが意図するレコードの同一性と、オブジェクトの同一性が異なっています。

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

## 良い

安定したレコードの `id` を比較し、raw か Proxy かに依存せず意図した同一性を判定します。

`App.vue`

```vue
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
