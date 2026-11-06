---
title: "petite-vue/valid-v-scope"
description: "v-scope に値を指定する場合は、オブジェクトリテラルにします。"
---

# `petite-vue/valid-v-scope`

v-scope に値を指定する場合は、オブジェクトリテラルにします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。

## 悪い例

値を持つ四つの `v-scope` は識別子、関数呼び出し、算術式、数値であり、いずれもオブジェクトリテラルではありません。

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 良い例

値を省略した `v-scope` はルートスコープを使います。ほかは括弧付きのものも含めてオブジェクトリテラルであり、このルールで許可されます。

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
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
      "petite-vue/valid-v-scope": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [petite-vue ルール](../petite-vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L48)
