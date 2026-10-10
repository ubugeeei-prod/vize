---
title: "Vapor ルール"
---

# Vapor ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`script/no-get-current-instance`](https://vizejs.dev/ja/rules/vapor.html#script-no-get-current-instance) | [悪い](https://vizejs.dev/ja/rules/vapor.html#script-no-get-current-instance-bad) · [良い](https://vizejs.dev/ja/rules/vapor.html#script-no-get-current-instance-good) | Vapor で null を返す getCurrentInstance() を検出します。 |
| [`script/no-next-tick`](https://vizejs.dev/ja/rules/vapor.html#script-no-next-tick) | [悪い](https://vizejs.dev/ja/rules/vapor.html#script-no-next-tick-bad) · [良い](https://vizejs.dev/ja/rules/vapor.html#script-no-next-tick-good) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |
| [`script/no-options-api`](https://vizejs.dev/ja/rules/vapor.html#script-no-options-api) | [悪い](https://vizejs.dev/ja/rules/vapor.html#script-no-options-api-bad) · [良い](https://vizejs.dev/ja/rules/vapor.html#script-no-options-api-good) | Vapor で Options API を使用する箇所を検出します。 |
| [`vapor/no-inline-template`](https://vizejs.dev/ja/rules/vapor.html#vapor-no-inline-template) | [悪い](https://vizejs.dev/ja/rules/vapor.html#vapor-no-inline-template-bad) · [良い](https://vizejs.dev/ja/rules/vapor.html#vapor-no-inline-template-good) | Vapor で削除済みの inline-template 属性を検出します。 |
| [`vapor/no-vue-lifecycle-events`](https://vizejs.dev/ja/rules/vapor.html#vapor-no-vue-lifecycle-events) | [悪い](https://vizejs.dev/ja/rules/vapor.html#vapor-no-vue-lifecycle-events-bad) · [良い](https://vizejs.dev/ja/rules/vapor.html#vapor-no-vue-lifecycle-events-good) | Vapor が対応しない要素の @vue:* lifecycle event を検出します。 |
| [`vapor/prefer-static-class`](https://vizejs.dev/ja/rules/vapor.html#vapor-prefer-static-class) | [悪い](https://vizejs.dev/ja/rules/vapor.html#vapor-prefer-static-class-bad) · [良い](https://vizejs.dev/ja/rules/vapor.html#vapor-prefer-static-class-good) | 文字列リテラルの :class を静的 class に置き換えます。 |
| [`vapor/require-vapor-attribute`](https://vizejs.dev/ja/rules/vapor.html#vapor-require-vapor-attribute) | [悪い](https://vizejs.dev/ja/rules/vapor.html#vapor-require-vapor-attribute-bad) · [良い](https://vizejs.dev/ja/rules/vapor.html#vapor-require-vapor-attribute-good) | Vapor 向けの script setup に vapor 属性を付ける方針を適用します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)
