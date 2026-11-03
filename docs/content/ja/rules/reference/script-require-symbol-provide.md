---
title: "script/require-symbol-provide"
description: "provide / inject のキーに衝突しにくい Symbol を使います。"
---

# `script/require-symbol-provide`

provide / inject のキーに衝突しにくい Symbol を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`provide` と `inject` が `'user'`、`'theme'` のような文字列キーを使い、同じ綴りを使うほかの provider と衝突し得ます。

```vue annotate="remove:1,2,3,4,6,7"
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

## 良い例

共有する `UserKey` を `Symbol` で作り、`InjectionKey<User>` の型を付けます。両呼び出しに同じキーを渡し、文字列リテラルを使わないようにします。

```vue annotate="add:1,2,3,5,6,7,8,9"
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
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
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39)
