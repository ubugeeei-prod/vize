---
title: "vize:croquis/cf/hydration-risk"
---

# `vize:croquis/cf/hydration-risk`

このコードはprop を ref にコピーする操作など、複数のリアクティビティ検出をまとめています。ファイル間検査がすべての Date.now() 式を検出するという意味ではありません。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/hydration-risk": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 共通のプロジェクト ファイル

以下のファイルは悪い例・良い例で共通です。Vue を、Router の例では vue-router もインストールしてください。エントリー ファイルでコンポーネントの関係を明確にしています。

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

## 悪い

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
const props = defineProps<{ count: number }>();
const count = ref(props.count);
</script>
<template><p>{{ count }}</p></template>
```

## 良い

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";
const props = defineProps<{ count: number }>();
const count = toRef(props, "count");
</script>
<template><p>{{ count }}</p></template>
```

良い例はこの検出を避ける修正です。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
