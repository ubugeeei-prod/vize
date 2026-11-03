---
title: "vize:croquis/cf/undefined-slot"
---

# `vize:croquis/cf/undefined-slot`

親が、子が公開していないスロットを埋めています。

| 既定の重大度 | 自動修正 | オプション |
| --- | --- | --- |
| 現在は生成されません | なし。関連ファイルを確認して修正してください | コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます |

適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

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

`Card.vue`

```vue
<script setup lang="ts">
defineSlots<{ header(): unknown }>();
</script>

<template>
<article><header><slot name="header" /></header></article>
</template>

```

## 悪い例

App が `footer` スロットを渡しますが、Card が宣言して描画するのは `header` だけです。渡した Notice を表示するスロットの挿入位置が子にありません。

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #footer>Notice</template></Card>
</template>

```

## 良い例

子の型付きスロット宣言とスロットの挿入位置に合わせて `header` を渡し、Notice を表示できるようにします。

`App.vue`

```vue
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #header>Notice</template></Card>
</template>

```

## 設定（Vite+）

この診断コードは現在 CLI では出力されないため、有効化する CLI のルール設定はありません。

## 関連ページ

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
