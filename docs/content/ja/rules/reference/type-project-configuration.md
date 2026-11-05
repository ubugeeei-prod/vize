---
title: "type/project-configuration"
description: "型情報を使う lint を妨げる TypeScript プロジェクト設定のエラーを修復します。"
---

<!-- Reviewed translation; source: rules/reference/type-project-configuration.md -->

# `type/project-configuration`

型情報を使う lint で、選択された TypeScript プロジェクト設定や、その継承先を読み込めないとき、
このエラーを報告します。プリセットのルール一覧とは別に発生する処理上の診断です。
自動修正はありません。

## 診断が出る条件

有効な型情報の検査に型の問い合わせが必要になると、Vize はネイティブの型チェック用セッションを
起動する前に、選択された `tsconfig` を検証します。検証には `extends` の継承関係も含まれます。
設定ファイルを読めない場合、構文が不正な場合、継承する設定を解決できない場合は、
その型の問い合わせを実行できません。

メッセージには、選択された設定と失敗の理由が示されます。リンターの同じインスタンスでは、
設定ごとに1回だけ、最初にその問題に遭遇したソースファイルの先頭を、長さゼロの範囲で示します。
そのファイルの型エラーではなく、プロジェクトの準備に関するエラーです。
失敗した型の問い合わせを必要としない検査は、独立した問題を引き続き報告できます。

## 悪い例

この `tsconfig.json` は、まだ生成されていない設定ファイルを参照しています。

```json
{
  "extends": "./.generated/tsconfig.base.json",
  "include": ["src"]
}
```

## 良い例

lint を実行する前に、継承先の `.generated/tsconfig.base.json` を作成します。
この小さな例では、内容全体を次のようにできます。元の `tsconfig.json` は上のまま使います。

```json
{
  "compilerOptions": {
    "strict": true
  }
}
```

フレームワークを使うプロジェクトでは、この説明用の基底ファイルへ置き換えず、
フレームワークの準備コマンドで必要な設定を生成してください。

## 修正方法

1. メッセージに示された設定を開きます。選択された `tsconfig` が対象のソースファイル用かを
   確認してください。明示的に選択する設定は `vue.typeChecker.tsconfig` です。
2. 不正な設定構文やアクセスできないファイルを修復します。見つからない `extends` の各参照は、
   必要なファイルの生成、パスの修正、継承元の設定を提供するパッケージのインストールで解決します。
3. 準備が完了してから lint を再実行します。設定の修正後は新しい実行を開始し、
   リンターが修復後の設定を改めて検証できるようにしてください。

診断を消すためだけに `extends` を削除しないでください。継承するコンパイラオプションが
プロジェクトに必要な場合があります。診断 ID が `type/corsa-runtime` の場合は、
[型と script のルール](../type-and-script.md)で、ネイティブのランタイムの選択と互換性を確認してください。

## 関連ページ

- [Lint の実行設定](../../guide/lint-execution.md)
- [Vue の設定を共有する](../../guide/vue-configuration.md)
- [型と script のルール](../type-and-script.md)
- [型チェック](../../guide/type-checking.md)
- [診断のソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/native_type_aware.rs)
- [プロジェクト設定の検証処理](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/linter/corsa_session/paths/store.rs)
