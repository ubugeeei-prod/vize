---
title: "css/no-utility-classes"
description: "コンポーネント内でユーティリティクラスを定義する箇所を検出します。"
---

# `css/no-utility-classes`

コンポーネント内でユーティリティクラスを定義する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）。

## 悪い例

`.flex`、`.mt-4`、`.text-center` のように、個々の見た目を名前にしたクラスを定義しています。

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

## 良い例

コンポーネント固有の `.my-component` にスタイルをまとめます。

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-utility-classes": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Musea と CSS のルール](../musea-and-css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37)
