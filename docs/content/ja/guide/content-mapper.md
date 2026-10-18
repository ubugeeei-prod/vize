---
title: TypeScript Content Mapper
---

<!-- Reviewed translation; source: guide/content-mapper.md -->

# TypeScript Content Mapper

Content Mapper に対応したネイティブ TypeScript ホストで Vue ファイルを扱うためのガイドです。
日常の型チェックは[静的解析](./static-analysis.md)、エディターの設定は
[VS Code 連携](../integrations/vscode.md)を参照してください。

Content Mapper は、TypeScript が直接解析できない種類のファイルを型チェックするための
プラグイン機構です。[TypeScript 7.1 API ロードマップ](https://github.com/microsoft/typescript-go/issues/4830)
では、Vue が必要とする TS Server プラグインの後継として位置づけられています。最初の API は
[microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712) で
マージされました。この開発用リポジトリは現在アーカイブされており、ネイティブ TypeScript の
開発は [microsoft/TypeScript](https://github.com/microsoft/TypeScript) で続いています。
[Content Mapper の準拠性検証ワークフロー](https://github.com/ubugeeei-prod/vize/blob/main/.github/workflows/content-mapper-conformance.yml)
で、このページに示すホストの正確な取得方法とビルド手順を確認できます。

Vize の Content Mapper は `vize` npm パッケージに同梱されています。対応する `tsgo` が
`vize content-mapper` を起動して `.vue` ファイルを直接検査します。ホバー、定義への移動、
名前の変更、補完、診断の位置は、元の SFC に対応づけられます。別の `.vue.ts` プロジェクトを
生成する必要はありません。

> **⚠️ プレビュー:** この連携には、Content Mapper と `--runExternalCode` に対応した
> ネイティブ TypeScript ホストが必要です。バージョン番号だけでは対応を判断できません。
> マッパーの動作確認には、このページで示す検証済みのホストを使ってください。
> 本番の型チェックでは、引き続きサポート対象の [`vize check`](./cli.md#check) を使えます。

## セットアップ

`vize` をインストールし、`tsconfig.json` にマッパーを指定します。

```bash
vp install -D vize
```

```json
{
  "compilerOptions": {
    "module": "preserve",
    "strict": true
  },
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"]
    }
  ],
  "include": ["src"]
}
```

外部マッパーの実行には明示的な許可が必要です。次のコマンドは、Vize の準拠性検証で
`tsgo` という名前にしている Content Mapper 対応バイナリを使います。検証ワークフローは
現在のネイティブコンパイラーのエントリーポイント `tsc/cmd/tsc` を、このページで示す
正確なリビジョンからビルドします。

```bash
tsgo --runExternalCode --noEmit -p tsconfig.json
```

VS Code では、信頼されたワークスペースで Content Mapper 対応の TypeScript ホストを使うと、
Vize 拡張機能が `.vue` の対応を自動登録します。同じマッパーがエディターでも動作します。

## オプション

マッパーの設定には `options` オブジェクトを指定できます。

```json
{
  "contentMappers": [
    {
      "package": "vize",
      "extensions": [".vue"],
      "options": { "optionsApi": false }
    }
  ]
}
```

| オプション   | デフォルト | 用途                                                            |
| ------------ | ---------- | --------------------------------------------------------------- |
| `optionsApi` | `true`     | テンプレート内で Vue Options API のインスタンスの変数を解決する |

不正なオプションがあっても、そのためにビルドは失敗しません。Vize は tsconfig の該当箇所に
オプションの診断（`vize1`〜`vize3`）を表示し、既定値で続行します。また、プロジェクトの
`noUnusedLocals` コンパイラーオプションに従って、`<script setup>` 内の未使用変数を報告します。

Vue の検査設定は、選択された tsconfig のトップレベルの `vueCompilerOptions` に記述します。

```json
{
  "vueCompilerOptions": {
    "checkUnknownProps": true,
    "strictCssModules": true
  }
}
```

これらの設定は既存の tsconfig の `extends` に従います。`checkUnknownProps` の既定値は
`strictTemplates`（未指定なら `false`）で、明示した `false` が優先されます。
`strictCssModules` の既定値は `false` です。Vize は設定の依存ファイルを TypeScript ホストに
返すため、継承した設定が変更された場合も生成結果を無効化できます。

## テンプレートディレクティブ

`<script>` ブロックでは、通常どおり `@ts-expect-error` を使えます。テンプレート式には
TypeScript のコメントを書けないため、Vize は Vue 標準の HTML コメントによる指示を
プロトコルを通して伝えます。

```vue
<template>
  <!-- @vue-expect-error -->
  {{ count.toFixed(true) }}

  <!-- @vue-ignore -->
  {{ untypedThirdPartyValue.field }}
</template>
```

- `<!-- @vue-expect-error -->` は次のテンプレート行の TypeScript 診断を抑制し、何も
  抑制しなかった場合は `vize4: Unused '@vue-expect-error' directive` を報告します。
- `<!-- @vue-ignore -->` は診断を抑制し、未使用の指示については報告しません。

コメントの後ろに内容が続く場合、指示は同じ行の残りの部分に適用されます。
そうでなければ、次の空行ではない行に適用されます。

## プロトコル

Vize は現在の Content Mapper の初期化仕様に従います。番号つきの `protocolVersion` は
交渉しません。位置の符号化方式には UTF-8 を選び、プロジェクトごとに
`openProject`/`closeProject` で状態を管理します。仮想出力は `.tsx` で、TypeScript と
埋め込まれた JSX の両方を正しく解析できます。

準拠性検証では、[Content Mapper の第2段階の変更](https://github.com/microsoft/TypeScript/pull/63936)
がマージされた `microsoft/TypeScript` の正確なリビジョン
`d6c4afddb2c55f4a9dea7b59293a99a8fdea1799` から、ネイティブコンパイラーをビルドします。
パック済み npm 成果物を使って CLI・ビルドモード・LSP の全スイートを実行し、元の Vue
ファイルを指す declaration map の出力や、別にパックした利用側から `.vue` への移動も検証します。
これはそのホストの動作確認であり、公開済みのすべてのパッケージに対応するという主張ではありません。

`vize` ソースで報告される診断コード:

| コード  | 意味                                         |
| ------- | -------------------------------------------- |
| `vize1` | マッパーオプションの値がオブジェクトではない |
| `vize2` | 未知のマッパーオプション                     |
| `vize3` | マッパーオプションの型が不正                 |
| `vize4` | 未使用の `@vue-expect-error` ディレクティブ  |

## 制限事項

- Content Mapper API と、外部コードの実行を明示的に許可する設定が必要です。
  アーカイブされた開発用リポジトリは、現在の開発元ではありません。
- declaration map を求めた
  [microsoft/typescript-go#4860](https://github.com/microsoft/typescript-go/issues/4860)
  は、第2段階の変更で完了しています。Vize は検証対象のホストでマップの出力と
  パックした利用側からの移動を確認しています。他のホストは、それぞれ動作確認が必要です。
- 本番の型チェックでは、引き続き `vize check` がサポート対象です。マッパーを使う前に、
  インストールしたホストとエディターがこの連携に対応していることを確認してください。
