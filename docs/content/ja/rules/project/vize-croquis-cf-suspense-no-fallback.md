---
title: "vize:croquis/cf/suspense-no-fallback"
---

# `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` に fallback がありません。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

fallback のない Suspense は有効な Vue の構文です。ローディング UI を置くという方針の例であり、コンパイルエラーではありません。この公開契約には現在の生成元がありません。

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

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

## 悪い

Suspense の境界に非同期の子がありますが fallback がなく、この例の待機中の状態に表示する内容がありません。

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /></Suspense>
</template>

```

## 良い

`#fallback` slot にローディングの段落を指定し、非同期の子が完了するまで表示します。

`App.vue`

```vue
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
