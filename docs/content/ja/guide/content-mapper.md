---
title: TypeScript Content Mapper
---

<!-- Generated translation; source: guide/content-mapper.md -->

# TypeScript Content Mapper

Content Mapper は、コンパイラー自身がパースできないファイルタイプをチェックするための
TypeScript のプラグイン機構です。[TypeScript 7.1 API ロードマップ](https://github.com/microsoft/typescript-go/issues/4830)
では、Vue に必要な TS Server プラグインの後継として位置づけられています。この API は
[microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712) で
`typescript-go` の main ブランチにマージされました。

Vize は `vize` npm パッケージの中に準拠した Content Mapper を同梱しています。Content Mapper
対応の `tsgo` ビルドが `vize content-mapper` を起動し、`.vue` ファイルを直接チェックします —
ホバー、定義ジャンプ、リネーム、補完、診断はすべて元の SFC にマップされ、並行する
`.vue.ts` プロジェクトを生成する必要はありません。

> **⚠️ プレビュー:** Content Mapper は upstream にマージ済みですが、リリース済みの
> TypeScript 7 platform package にはまだ含まれていません。プロトコルを含むリリースが出るまでは、
> `typescript-go` の main から Content Mapper 対応の native TypeScript バイナリをビルドし、
> サポートされる型チェック手段としては [`vize check`](./cli.md#check) を使い続けてください。

## セットアップ

`vize` をインストールし、`tsconfig.json` でマッパーを宣言します:

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

外部マッパープロセスの実行には明示的なオプトインが必要です:

```bash
tsgo --runExternalCode --noEmit -p tsconfig.json
```

VS Code では、信頼されたワークスペースであれば Vize 拡張機能が TypeScript 7
content-mapper ホストに `.vue` サポートを自動登録し、同じマッパーがエディターでも動作します。

## オプション

マッパーエントリは `options` オブジェクトを受け取ります:

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

| オプション   | デフォルト | 用途                                                           |
| ------------ | ---------- | -------------------------------------------------------------- |
| `optionsApi` | `true`     | テンプレート内で Vue Options API のインスタンスバインディングを解決 |

不正なオプションでビルドが失敗することはありません。Vize は tsconfig 内の位置を指す
オプション診断(`vize1`〜`vize3`)として報告し、デフォルト値で続行します。また Vize は
プロジェクトの `noUnusedLocals` コンパイラーオプションへの依存を宣言しているため、
`<script setup>` 内の未使用ローカルの報告は各プロジェクトの設定に従います。

## テンプレートディレクティブ

`<script>` ブロックはそのまま透過するため `@ts-expect-error` が通常どおり使えます。
テンプレート式には TS コメントを書けないので、Vize は Vue 標準の HTML コメント
ディレクティブをプロトコル経由でマップします:

```vue
<template>
  <!-- @vue-expect-error -->
  {{ count.toFixed(true) }}

  <!-- @vue-ignore -->
  {{ untypedThirdPartyValue.field }}
</template>
```

- `<!-- @vue-expect-error -->` は次のテンプレート行の TypeScript 診断を抑制し、何も
  抑制されなかった場合は `vize4: Unused '@vue-expect-error' directive` を報告します。
- `<!-- @vue-ignore -->` は黙って抑制します。

ディレクティブは、コメントの後ろに内容が続く場合はその行の残り、そうでなければ次の
空でない行に適用されます。

## プロトコル

Vize は upstream にマージされた Content Mapper プロトコル v1 を話します: UTF-8 位置
エンコーディング、プロジェクトごとの `openProject`/`closeProject` ライフサイクル、そして
TypeScript と組み込み JSX の両方が正しくパースできる `.tsx` 仮想出力です。準拠性は CI で
担保されており、pin された `typescript-go` リビジョンから正確な upstream コンパイラーを
ビルドし、パック済み npm 成果物を通して CLI・ビルドモード・LSP の全スイートを実行します。

`vize` ソースで報告される診断コード:

| コード  | 意味                                             |
| ------- | ------------------------------------------------ |
| `vize1` | マッパーオプションの値がオブジェクトではない     |
| `vize2` | 未知のマッパーオプション                         |
| `vize3` | マッパーオプションの型が不正                     |
| `vize4` | 未使用の `@vue-expect-error` ディレクティブ      |

## 制限事項

- TypeScript 7 のリリースに API が含まれるまで、`typescript-go` の main からビルドした
  `tsgo` が必要です。
- マップされた入力の declaration map は
  [microsoft/typescript-go#4860](https://github.com/microsoft/typescript-go/issues/4860) 待ちです。
- upstream API がプレビューの間は、プロダクションの型チェック手段としては `vize check` が
  引き続きサポート対象です。
