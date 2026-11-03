---
title: "vize:croquis/cf/destructuring-breaks-reactivity"
---

# `vize:croquis/cf/destructuring-breaks-reactivity`

reactive オブジェクトの分割代入はフィールドをコピーし、追跡を失います。

| 既定の重大度 | 自動修正 | オプション |
| --- | --- | --- |
| error | なし。関連ファイルを確認して修正してください | コード固有のオプションはありません。対応済みの CLI の診断は、重大度を変更できます |

適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報

## 共通のプロジェクト ファイル

以下のファイルは、悪い例・良い例の両方で変更せずに使います。プロジェクトには Vue をインストールし、例で import している場合は vue-router や Pinia もインストールしてください。バージョン固有の対応条件がある場合は、その注意に従ってください。エントリーファイルによって、コンポーネント間の関係を明示しています。

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

## 悪い例

`props` オブジェクトを通常の分割代入で展開すると、現在の `item` の値を取り出します。Vue 3.5 の `defineProps()` から直接行う分割代入とは別です。

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

## 良い例

`toRef(props, "item")` を使い、`props` のプロパティとのつながりを保ちます。

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

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 関連ページ

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
