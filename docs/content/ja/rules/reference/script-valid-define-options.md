---
title: "script/valid-define-options"
description: "defineOptions の引数と使用回数を検査します。"
---

# `script/valid-define-options`

defineOptions の引数と使用回数を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

最初の呼び出しが専用マクロで宣言すべき `props` を `defineOptions` に入れています。後続の呼び出しはマクロを繰り返し、オブジェクトでない引数も渡しています。禁止された形式と呼び出し回数の制約を示す例です。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

## 良い例

`defineOptions` を一度だけ呼び、通常の対応オプション `name` と `inheritAttrs` を持つオブジェクトを渡します。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41)
