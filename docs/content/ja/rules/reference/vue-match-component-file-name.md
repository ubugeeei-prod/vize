---
title: "vue/match-component-file-name"
description: "宣言したコンポーネント名をファイル名と一致させます。"
---

# `vue/match-component-file-name`

宣言したコンポーネント名をファイル名と一致させます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

この検査にはファイル名も使います。宣言したコンポーネント名を PascalCase または kebab-case に変換し、拡張子を除いた元のファイル名と比較します。コンポーネント定義が複数あるファイルは検査しません。

## 悪い例

宣言したコンポーネント名 `OtherPanel` は、対応する表記変換を行っても `MyPanel.vue` のファイル名と一致しません。

`MyPanel.vue`

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "OtherPanel" });
</script>
```

## 良い例

コンポーネント名を `MyPanel` とし、定義されているファイルの名前と一致させます。

`MyPanel.vue`

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "MyPanel" });
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/match-component-file-name": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/match_component_file_name.rs#L21)
