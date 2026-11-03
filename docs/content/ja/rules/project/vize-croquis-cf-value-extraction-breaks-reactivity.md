---
title: "vize:croquis/cf/value-extraction-breaks-reactivity"
---

# `vize:croquis/cf/value-extraction-breaks-reactivity`

reactive な値をローカルへ取り出すと、その後の更新が失われます。

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

Vue 3.5 の reactive な props 分割代入で得た `item` を、`itemSnapshot` に一度だけ読み取っています。後の prop の置き換えは、そのスナップショットに反映されません。

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
const { item } = defineProps<{ item: { name: string } }>();
const itemSnapshot = item;
</script>
```

## 良い例

`computed` 内で `item` を読み取ります。Vue がリアクティブな props の分割代入を変換することで、評価のたびにその依存を追跡できるようにします。

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
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/value-extraction-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

## 関連ページ

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
