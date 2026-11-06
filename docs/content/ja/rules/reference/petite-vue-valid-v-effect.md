---
title: "petite-vue/valid-v-effect"
description: "v-effect に空でない式を指定します。"
---

# `petite-vue/valid-v-effect`

v-effect に空でない式を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。

## 悪い例

各 `v-effect` の値が未指定、空文字、空白のみであり、実行する式がありません。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 良い例

一方は `el.textContent` を更新し、もう一方は `count` を増やす式を指定しています。このルールが確認するのは式が空でないことであり、処理内容の妥当性ではありません。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "petite-vue/valid-v-effect": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [petite-vue ルール](../petite-vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35)
