---
title: "vize:croquis/cf/setup-context-violation"
---

# `vize:croquis/cf/setup-context-violation`

インスタンスごとに持つリアクティブな状態を setup 内で作ります。

| 既定の重大度 | 自動修正 | オプション |
| --- | --- | --- |
| context-dependent | なし。関連ファイルを確認して修正してください | コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます |

適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報

実験的な Rust CrossFileAnalyzer には、この診断コードの生成元があります。CLI の検査はこの個別コードを生成せず、ID を設定しても Rust 側の検査は有効になりません。以下の例は解析器が対応するグラフ構造や情報を説明するもので、Vite+ での検出を保証するものではありません。

## 範囲と制約

実験的な解析器の例では、インスタンスごとの setup ではなく通常の script のモジュールスコープに `ref(0)` を置いています。Vue ではこのようなリアクティブな状態を作れますが、この例ではインスタンスごとに count を持つことを前提とします。CLI はこの個別の診断コードを生成しません。

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 悪い例

通常の script ブロックのモジュールスコープで `ref(0)` を作っており、この解析器の例で想定する、インスタンスごとの setup コンテキストには含まれません。

`App.vue`

```vue
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

## 良い例

script setup に移し、各インスタンスが count を持ち、テンプレートから読み取れる形にします。

`App.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

## 設定（Vite+）

この診断コードは現在 CLI では出力されないため、有効化する CLI のルール設定はありません。

## 関連ページ

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[ファイル間ルール一覧](../cross-file.md)
