---
title: "vize:croquis/cf/reference-escapes-scope"
---

# `vize:croquis/cf/reference-escapes-scope`

reactive な参照が、寿命を持つスコープの外へ漏れています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

ref を composable から返したり、スコープをまたいで共有したりすることは有効です。この例では cache にスナップショットを求めており、アンマウントで ref 自体が無効になると主張するものではありません。現在この契約の生成元はありません。

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

`saved.ts`

```ts
import type { Ref } from 'vue';
let saved: Ref<number> | number | undefined;
export function remember(value: Ref<number> | number): void { saved = value; }
export function remembered(): Ref<number> | number | undefined { return saved; }

```

## 悪い

プロセス全体の cache がコンポーネントの count ref をそのまま保持し、アンマウント後もインスタンスの状態を参照できるようにします。スナップショットを保存する意図にもかかわらず、後の変更も観測できます。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

## 良い

cache に現在の通常の数値を渡し、コンポーネントが持つ ref を保持せずスナップショットを保存します。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count.value);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
