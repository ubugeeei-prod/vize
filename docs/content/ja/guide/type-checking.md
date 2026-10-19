---
title: 型チェック
description: vize check と TypeScript のネイティブコンパイラーで Vue SFC、TypeScript、TSX を型チェックします。
---

# 型チェック

`vize check` は `.vue`、`.ts`、`.tsx`、`.d.ts` をプロジェクト全体で一度に型チェックします。
`vue-tsc --noEmit` に相当するコマンドで、同じ TypeScript の診断（同じ `TS####` コード）を、`.vue`
ファイル内の該当行を指して報告します。

> [!NOTE]
> 型チェックはアルファ版の機能です。CI の手順を置き換える前に、現在の `vue-tsc` と並べて実行し、
> 自分のプロジェクトで結果を比べてください。[安定性](../stability.md)も参照してください。

## 実行する

[Vize のヘルパー](./vite-plus.md)を使う Vite+ プロジェクトでは、生成されたタスクを実行します。

```bash
vp run typecheck
```

Vite+ を使わない場合は、`vize` パッケージをインストールしてコマンドを直接実行します。パッケージには
各プラットフォーム向けの TypeScript ネイティブコンパイラーが含まれているため、追加のインストールは
不要です。

```bash
vp install -D vize
vp exec vize check
```

引数を省略すると、最も近い `tsconfig.json` の `files`、`include`、`exclude` を読み込みます。
対象を絞る場合は、パスや別のプロジェクトを指定します。

```bash
vize check src
vize check --tsconfig tsconfig.app.json
```

## 出力の例

次のコンポーネントがあるとします。

```vue
<script setup lang="ts">
const props = defineProps<{ step: number }>();
const emit = defineEmits<{ change: [value: number] }>();
</script>

<template>
  <button @click="emit('change', props.step.toFixed())">{{ props.stepp }}</button>
</template>
```

`vize check` はテンプレート内の 2 つの誤りを、ソース上の位置とともに報告します。

```text
src/Counter.vue
  error:7:34 [TS2345] Argument of type 'string' is not assignable to parameter of type 'number'.
  error:7:66 [TS2551] Property 'stepp' does not exist on type ... Did you mean 'step'?

✗ Type checked 1 files in 487.30ms
  2 error(s)
```

エラーがあると終了コードが 0 以外になるため、そのまま CI のゲートに使えます。機械で読む場合は
`--format json`、警告でも失敗させる場合は `--max-warnings 0` を使ってください。

## チェック対象

| 対象                                      | デフォルト | 無効にする方法                    |
| ----------------------------------------- | ---------- | --------------------------------- |
| `<script>` と `<script setup>` のコード   | 有効       | `tsconfig` でファイルを除外する   |
| テンプレートの式とバインディング          | 有効       | `--no-check-template-bindings`    |
| 子コンポーネントに渡す props              | 有効       | `--no-check-props`                |
| emit するイベントとその引数               | 有効       | `--no-check-emits`                |
| Options API のテンプレートバインディング（`data`、`computed`、`methods` など） | 有効 | `typeChecker.optionsApi: false` |
| 通常の `.ts` / `.tsx` / `.d.ts`           | 有効       | `tsconfig` でファイルを除外する   |

同じ設定は `vite.config.ts` にも書けます。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  typecheck: {
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    optionsApi: true,
  },
});
```

型チェックを現在のツールに任せる場合は `typecheck: false` にします。Vite+ のヘルパー以外では、同じ
オブジェクトを `typeChecker` と書きます。すべてのキーは[設定リファレンス](./configuration-reference.md)
を参照してください。

## グローバルな型と自動インポート

Vize が参照できるのは、TypeScript プロジェクトから見える型だけです。`declare module "vue"` で追加した
グローバルや、`auto-imports.d.ts`、`components.d.ts` などの生成ファイルは、チェックする
`tsconfig.json` の `include` に含めてください。

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

> [!TIP]
> `$t`、`$route`、自動インポートしたコンポーネントが「存在しない」と報告される場合、多くは宣言
> ファイルが `include` の外にあります。`include` に追加するか、`--tsconfig` でそのファイルを含む
> プロジェクトを指定してください。

## vue-tsc から移行する

| 現在のコマンド                               | Vize での書き方                                   |
| -------------------------------------------- | ------------------------------------------------- |
| `vue-tsc --noEmit`                           | `vize check`                                      |
| `vue-tsc -p tsconfig.app.json --noEmit`      | `vize check --tsconfig tsconfig.app.json`         |
| `vue-tsc --declaration --emitDeclarationOnly` | `vize check --declaration --declaration-dir dist/types` |
| Vue - Official 拡張機能の診断                | `typecheck` を有効にした [Vize の言語サーバー](./language-server.md) |

`vue-tsc` は JavaScript 版の TypeScript コンパイラーで動きますが、Vize は TypeScript 7 のネイティブ
コンパイラー（Go 実装、Project Corsa）で動きます。計測した差は
[型チェッカーのベンチマーク](../architecture/performance.md#ベンチマーク-型検査)を参照してください。
細かなケースでは診断が異なることがあるため、[移行ガイド](./migration.md)の手順どおり、切り替える前に
結果を比べてください。

## 診断を調べる

Vize は各 SFC を仮想 TypeScript に変換してからチェックします。診断がおかしいと感じたら、生成された
コードを確認してください。

```bash
vize check --show-virtual-ts src/components/Counter.vue
```

`--save-virtual-ts-for <file>` はコンポーネントの隣にファイルとして保存し、`--profile` は処理時間と
すべての仮想ファイルを `node_modules/.vize` に書き込みます。[不具合を報告する](./testing.md)ときは
この出力を添えてください。

## 大きなプロジェクトでの速度

`vize check` は import グラフの独立した部分を複数のネイティブ TypeScript プロセスに分けて処理し、
プロセス数は CPU とファイル数から自動で決めます。メモリが少ない場合や順序を固定したい場合は、
1 プロセスで実行してください。

```bash
vize check --servers 1
```

プロファイルと調整の詳細は[型チェックのパフォーマンス](./check-performance.md)を参照してください。
開発中に何度も実行する場合は、サーバーを起動したままにしてチェックを接続できます（Unix のみ）。

```bash
vize check-server --socket /tmp/vize-check.sock
vize check --socket /tmp/vize-check.sock src
```

## 関連ページ

- [CLI リファレンス: `vize check`](./cli.md#check)
- エディターで型エラーを表示する[言語サーバー](./language-server.md)
- 同じチェッカーを使う[型情報を使う lint ルール](../rules/type-and-script.md)
- [JSX と TSX の型チェック](./jsx.md)
