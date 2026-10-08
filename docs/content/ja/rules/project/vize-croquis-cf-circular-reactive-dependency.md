---
title: "vize:croquis/cf/circular-reactive-dependency"
---

# `vize:croquis/cf/circular-reactive-dependency`

reactive な計算が互いに循環して依存しています。

既定の重大度: context-dependent  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

Example qualification: `illustrative-source-pair`

以下の完全な Vue プロジェクトは、更新の循環とその修正を具体的に示します。このソースによる CLI の検出は検証済みではありません。診断の生成元には、併記したグラフのように保持された参照の ID と流れが必要です。このソースだけで現在の処理がこの診断コードを生成すると断定しません。参照 ID を保持したグラフに対する専用の検出検証と、ソースの構文検査は分けて扱います。

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
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

## 悪い

App が count（A）を保持して provide します。CycleView は nextCount（B）を computed で求め、watch でその値を同じ inject 済みの count に書き戻します。immediate の実行後も計算の入力が更新され続けるため、A → B → A の更新の循環になります。下の参照グラフはこの二つの参照を表し、同名の無関係な変数をつなぐものではありません。

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

## 良い

B の値を A に書き戻す watch を取り除きます。count は App が保持し、明示的な Increment 操作でだけ変更します。CycleView は nextCount を読み取るだけで計算結果を書き戻しません。同じ二つの参照には A → B の依存だけが残ります。

`CycleView.vue`

```vue
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
