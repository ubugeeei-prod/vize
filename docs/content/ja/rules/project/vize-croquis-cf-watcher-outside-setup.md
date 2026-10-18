---
title: "vize:croquis/cf/watcher-outside-setup"
---

# `vize:croquis/cf/watcher-outside-setup`

`watch` または `watchEffect` が `setup` の外で呼ばれています。

既定の重大度: 現在は生成されません  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます

この公開診断コードには現在の生成元がありません。悪い例・良い例はリスクと修正の説明です。このコードを検出させる設定は現在ありません。

所有側が停止関数を保持して呼び出す場合や、アプリケーション全体で動かし続けることを意図する場合、モジュール直下のウォッチャーは有効です。この例ではウォッチャーの有効期間をコンポーネントの生存期間に結び付けることを前提としています。この契約には現在の生成元がありません。

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

`App.vue`

```vue
<script setup lang="ts">
import Observer from './Observer.vue';
</script>

<template>
<Observer /><Observer />
</template>

```

`Observer.vue`

```vue
<script setup lang="ts">
import { useObserver } from './use-observer';
const { count, observed } = useObserver();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ observed }}</p>
</template>

```

## 悪い

ウォッチャーを各 `Observer` の setup 内ではなく、モジュールの読み込み時に作っています。そのため、両インスタンスが ref を共有し、個々の `Observer` をアンマウントしてもウォッチャーは自動停止しません。

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
const count = ref(0);
const observed = ref(0);
watch(count, next => { observed.value = next; });
export function useObserver() { return { count, observed }; }

```

## 良い

setup からの同期的な呼び出しごとに、`useObserver` 内で個別の ref とウォッチャーを作ります。Vue がウォッチャーの有効期間を呼び出し元のコンポーネントの生存期間に結び付けます。

`use-observer.ts`

```ts
import { ref, watch } from 'vue';
export function useObserver() {
  const count = ref(0);
  const observed = ref(0);
  watch(count, next => { observed.value = next; });
  return { count, observed };
}

```

良い例のファイルは、上で説明した変更を示すものです。プロジェクト全体を検査すると、ほかの診断が出る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
