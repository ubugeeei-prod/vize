---
title: "script/no-dupe-keys"
description: "Options API の props / data / computed などのキー重複を検出します。"
---

# `script/no-dupe-keys`

Options API の props / data / computed などのキー重複を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`foo` が props と data の両方に、`bar` が computed と methods の両方に宣言され、コンポーネントインスタンス上で同じキーの宣言が衝突しています。

```vue annotate="remove:5,8,9,10,11"
<script lang="ts">
export default {
  props: ['foo'],
  data() {
    return { foo: 1 } // duplicate of prop `foo`
  },
  computed: {
    bar() { return 2 }
  },
  methods: {
    bar() {} // duplicate of computed `bar`
  }
}
</script>
```

## 良い例

prop、data、computed に別々の名前 `foo`、`bar`、`baz` を使い、オプション間の重複を取り除きます。

```vue annotate="add:5,8"
<script lang="ts">
export default {
  props: ['foo'],
  data() {
    return { bar: 1 }
  },
  computed: {
    baz() { return 2 }
  }
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-dupe-keys": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L55)
