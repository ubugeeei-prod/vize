---
title: petite-vue ルール
---

# petite-vue ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](./all.md#petite-vue-no-unsupported-directive) | [悪い例](./all.md#petite-vue-no-unsupported-directive-bad) · [良い例](./all.md#petite-vue-no-unsupported-directive-good) | petite-vue が対応しないディレクティブを検出します。 |
| [`petite-vue/valid-v-effect`](./all.md#petite-vue-valid-v-effect) | [悪い例](./all.md#petite-vue-valid-v-effect-bad) · [良い例](./all.md#petite-vue-valid-v-effect-good) | v-effect に空でない式を指定します。 |
| [`petite-vue/valid-v-scope`](./all.md#petite-vue-valid-v-scope) | [悪い例](./all.md#petite-vue-valid-v-scope-bad) · [良い例](./all.md#petite-vue-valid-v-scope-good) | v-scope の値にオブジェクトリテラルを指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
