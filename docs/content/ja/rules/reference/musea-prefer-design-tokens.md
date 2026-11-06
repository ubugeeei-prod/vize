---
title: "musea/prefer-design-tokens"
description: "CSS に直接記述した値が登録済みのデザイントークンの値に一致する場合、対応する CSS 変数で表現します。"
---

# `musea/prefer-design-tokens`

CSS に直接記述した値が登録済みのデザイントークンの値に一致する場合、対応する CSS 変数で表現します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Musea の .art.vue ファイルの art / variant / style。

.art.vue ファイルと下記のデザイントークン一覧が必要です。任意の色からトークンを推測するルールではありません。

## 悪い例

art の例で、設定した primary のデザイントークンではなく、青の色を直接指定しています。

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

## 良い例

この例の設定に登録された --color-primary のトークンを参照します。

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
          ]
        }
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Musea ルール](../musea.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32)
