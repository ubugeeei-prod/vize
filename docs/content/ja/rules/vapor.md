---
title: Vapor ルール
---

# Vapor ルール

ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。

Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。

| ルール | 例 | 目的 |
| --- | --- | --- |
| [`script/no-get-current-instance`](./all.md#script-no-get-current-instance) | [悪い例](./all.md#script-no-get-current-instance-bad) · [良い例](./all.md#script-no-get-current-instance-good) | Vapor で null を返す getCurrentInstance() を検出します。 |
| [`script/no-next-tick`](./all.md#script-no-next-tick) | [悪い例](./all.md#script-no-next-tick-bad) · [良い例](./all.md#script-no-next-tick-good) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |
| [`script/no-options-api`](./all.md#script-no-options-api) | [悪い例](./all.md#script-no-options-api-bad) · [良い例](./all.md#script-no-options-api-good) | Vapor で Options API を使用する箇所を検出します。 |
| [`vapor/no-inline-template`](./all.md#vapor-no-inline-template) | [悪い例](./all.md#vapor-no-inline-template-bad) · [良い例](./all.md#vapor-no-inline-template-good) | Vapor で削除済みの inline-template 属性を検出します。 |
| [`vapor/no-vue-lifecycle-events`](./all.md#vapor-no-vue-lifecycle-events) | [悪い例](./all.md#vapor-no-vue-lifecycle-events-bad) · [良い例](./all.md#vapor-no-vue-lifecycle-events-good) | Vapor が対応しない要素の @vue:* lifecycle event を検出します。 |
| [`vapor/prefer-static-class`](./all.md#vapor-prefer-static-class) | [悪い例](./all.md#vapor-prefer-static-class-bad) · [良い例](./all.md#vapor-prefer-static-class-good) | 文字列リテラルの :class を静的 class に置き換えます。 |
| [`vapor/require-vapor-attribute`](./all.md#vapor-require-vapor-attribute) | [悪い例](./all.md#vapor-require-vapor-attribute-bad) · [良い例](./all.md#vapor-require-vapor-attribute-good) | Vapor 向けの script setup に vapor 属性を付ける方針を適用します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)
