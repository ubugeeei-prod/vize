---
title: Vapor ルール
---

# Vapor ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。個別ページが現在の対応範囲を示す参照先です。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 目的 |
| --- | --- |
| [`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Vapor で null を返す getCurrentInstance() を検出します。 |
| [`script/no-next-tick`](./reference/script-no-next-tick.md) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |
| [`script/no-options-api`](./reference/script-no-options-api.md) | Vapor で Options API を使用する箇所を検出します。 |
| [`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | Vapor で削除済みの inline-template 属性を検出します。 |
| [`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | Vapor が対応しない要素の @vue:* lifecycle event を検出します。 |
| [`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | 文字列リテラルの :class を静的 class に置き換えます。 |
| [`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | Vapor 向けの script setup に vapor 属性を付ける方針を適用します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
