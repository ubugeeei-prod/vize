---
title: "petite-vue ルール"
---

# petite-vue ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`petite-vue/no-unsupported-directive`](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-no-unsupported-directive) | [悪い](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-no-unsupported-directive-bad) · [良い](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-no-unsupported-directive-good) | petite-vue が対応しないディレクティブを検出します。 |
| [`petite-vue/valid-v-effect`](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-valid-v-effect) | [悪い](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-valid-v-effect-bad) · [良い](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-valid-v-effect-good) | v-effect に空でない式を指定します。 |
| [`petite-vue/valid-v-scope`](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-valid-v-scope) | [悪い](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-valid-v-scope-bad) · [良い](https://vizejs.dev/ja/rules/petite-vue.html#petite-vue-valid-v-scope-good) | v-scope の値にオブジェクトリテラルを指定します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
