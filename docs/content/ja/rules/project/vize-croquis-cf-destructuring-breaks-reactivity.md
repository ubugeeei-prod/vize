---
title: "vize:croquis/cf/destructuring-breaks-reactivity"
---

# `vize:croquis/cf/destructuring-breaks-reactivity`

reactive オブジェクトの分割代入はフィールドをコピーし、追跡を失います。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 共通のプロジェクト ファイル

以下のファイルは悪い例・良い例で共通です。import する Vue と、例で使う場合は vue-router / Pinia をインストールしてください。バージョン固有の注意がある場合は、その前提に合わせてください。エントリー ファイルでコンポーネントの関係を明確にしています。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

## 悪い

`props` object の通常の分割代入は現在の `item` を取り出します。Vue 3.5 の `defineProps()` から直接行う分割代入とは別です。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

## 良い

`toRef(props, "item")` で `props` の property への接続を維持します。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
