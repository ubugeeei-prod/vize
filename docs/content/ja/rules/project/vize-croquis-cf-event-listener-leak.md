---
title: "vize:croquis/cf/event-listener-leak"
---

# `vize:croquis/cf/event-listener-leak`

イベントリスナーが登録されたまま、解除されていません。

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

`App.vue`

```vue
<script setup lang="ts">
import { useWidth } from './use-width';
const width = useWidth();
</script>

<template>
<p>{{ width }}</p>
</template>

```

## 悪い例

マウント時に width の ref を参照する resize リスナーを window に追加しますが、アンマウント時に削除しません。再マウントを繰り返すと不要なリスナーや状態を保持し得ます。

`use-width.ts`

```ts
import { onMounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  return width;
}

```

## 良い例

`onUnmounted` で、マウント時に登録したものと同一の `resize` 関数を使ってリスナーを解除します。これで、そのインスタンスに属する外部リスナーの有効期間が終わります。

`use-width.ts`

```ts
import { onMounted, onUnmounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  onUnmounted(() => { window.removeEventListener('resize', resize); });
  return width;
}

```

## 設定（Vite+）

この診断コードは現在 CLI では出力されないため、有効化する CLI のルール設定はありません。

## 関連ページ

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[ファイル間ルール一覧](../cross-file.md)
