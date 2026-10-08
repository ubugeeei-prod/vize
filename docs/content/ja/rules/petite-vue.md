---
title: petite-vue ルール
---

# petite-vue ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | [悪い例](./reference/petite-vue-no-unsupported-directive.md#悪い) · [良い例](./reference/petite-vue-no-unsupported-directive.md#良い) | petite-vue が対応しないディレクティブを検出します。 |
| [`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | [悪い例](./reference/petite-vue-valid-v-effect.md#悪い) · [良い例](./reference/petite-vue-valid-v-effect.md#良い) | v-effect に空でない式を指定します。 |
| [`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | [悪い例](./reference/petite-vue-valid-v-scope.md#悪い) · [良い例](./reference/petite-vue-valid-v-scope.md#良い) | v-scope の値にオブジェクトリテラルを指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
