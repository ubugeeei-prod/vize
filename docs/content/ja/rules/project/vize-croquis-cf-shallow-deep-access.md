---
title: "vize:croquis/cf/shallow-deep-access"
---

# `vize:croquis/cf/shallow-deep-access`

`shallowReactive` または `shallowRef` の深いプロパティを、追跡されるものとして読んでいます。

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

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.user.name }}</p><button @click="profile.user.name = 'Grace'">Rename</button>
</template>

```

## 悪い

`shallowReactive` が追跡するのはルートの `user` プロパティであり、内側のオブジェクトは raw のままです。`profile.user.name` の変更は、追跡された深い変更としてテンプレートへ通知されません。

`profile.ts`

```ts
import { shallowReactive } from 'vue';
export function makeProfile() { return shallowReactive({ user: { name: 'Ada' } }); }

```

## 良い

深い `reactive` で内側の user オブジェクトも包み、同じ名前の代入を表示の更新につなげます。

`profile.ts`

```ts
import { reactive } from 'vue';
export function makeProfile() { return reactive({ user: { name: 'Ada' } }); }

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
