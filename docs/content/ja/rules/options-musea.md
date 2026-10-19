---
title: Musea のルールオプション
---
<!-- Reviewed translation; source: rules/options-musea.md -->

<span id="musea-rule-options"></span>

# Musea のルールオプション

Musea のスタイル検査に、プロジェクトで使うデザイントークンの一覧を渡す設定例です。
オプションの一覧、未知のフィールドの扱い、スコープごとの設定の置き換え方は、
[ルールのオプション](./options.md)を参照してください。

## `musea/prefer-design-tokens`

CSS に直接書いた値を、プロジェクトのデザイントークンにそろえたいときに使います。
トークンが設定されていない場合は検査しません。
CLI では、空でないトークン一覧を設定すると、このルールが選ばれます。
各トークンは、CSS の値と、`path` から作る CSS カスタムプロパティを対応づけます。

ルールを `off` にした場合は、空でないトークン一覧を設定しても無効のままです。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6"
            },
            {
              "path": "color.danger",
              "value": "#ef4444",
              "tier": "semantic"
            }
          ]
        }
      }
    }
  }
});
```

悪い例（上の設定で指摘される例）:

```vue
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

良い例:

```vue
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```
