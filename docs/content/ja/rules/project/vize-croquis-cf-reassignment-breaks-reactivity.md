---
title: "vize:croquis/cf/reassignment-breaks-reactivity"
---

# `vize:croquis/cf/reassignment-breaks-reactivity`

reactive な束縛を再代入すると、ただの値に置き換わります。

既定の重大度: error  
適用範囲: 解析対象のコンポーネント構成と、下記に示す対応済みの情報  
自動修正: なし。関連ファイルを確認して修正してください  
オプション: コード固有のオプションはありません。対応済みの CLI 検出は重大度を変更できます

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/reassignment-breaks-reactivity": "warn" },
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

子が作った prop の ref を `props.user` で上書きし、ref としての接続を失います。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

## 良い

`toRef` を `const` に保持し、参照を置き換える再代入を除きます。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

良い例のファイルは上で説明した変更を示します。プロジェクトには別の検出が残る場合があります。

[公開の説明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/ja.txt)

[生成元](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[ファイル間ルール一覧](../cross-file.md)
