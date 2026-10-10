---
title: 実験機能リファレンス
---

<!-- Reviewed translation; source: guide/experimentals-reference.md -->

<span id="experimentals-reference"></span>

# 実験機能リファレンス

このページは[実験機能](./experimentals.md)と[Vue RFC の実験機能の詳細](./experimentals-vue-rfcs.md)を補う詳細リファレンスです。
フラグをどこで解決するか、変更にどの検証が必要か、低レベル API のどの項目が解決済みの真偽値を受け取るかを確認できます。

<span id="entry-points"></span>

## 設定を渡す場所

設定を判断する責任がある、できるだけ上位の入口を使ってください。
プロジェクト設定は Vize の設定を読み込むツール向けです。
プラグインに直接渡すオプションは、個々の Vite プラグインに適用します。
ネイティブコンパイラの項目を直接使うのは、プロジェクトの設定方針をすでに解決した統合です。

| 設定を渡す場所                                                               | 受け取る値                                                    | 値を解決する主体                     | 注意点                                                                                                |
| ---------------------------------------------------------------------------- | ------------------------------------------------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------- |
| `vize.config.*`                                                              | `experimentals: { ... }` の有効・無効を表す値                 | 設定ローダー                         | npm コマンド、`vize check`、LSP セッション、プロジェクト設定を読む Vite プラグインの共通の既定値      |
| `vize({ experimentals })`                                                    | 上と同じ有効・無効を表す値                                    | Vite プラグインのオプション解決処理  | 直接指定した値が共通設定より優先される。`false` と `null` で、そのプラグインだけ無効にできる          |
| `vize({ vapor })`, `vize({ jsxMode })`, `compiler.vapor`, `compiler.jsxMode` | 安定版のコンパイラオプション                                  | コンパイラのオプション解決処理       | これらの設定が `experimentals.vapor` と `experimentals.jsxVapor` による代替の出力先選択より優先される |
| `compile`, `compileVapor`, `parseTemplate`                                   | `CompilerOptions` の `experimental*` 真偽値                   | 呼び出し元                           | 別名、`{}` の設定オブジェクト、共通設定の優先順位は解釈しない                                         |
| `compileSfc`, `compileSfcBatch`, `compileSfcBatchWithResults`                | SFC・バッチ処理のオプションの `experimental*` 真偽値          | 呼び出し元                           | 設定の解決後に、ネイティブ SFC または WASM の統合で使う                                               |
| `vize check` と LSP・型チェックのプロジェクト API                            | プロジェクトの `experimentals` と解決済みの型チェッカーフラグ | 設定ローダーとプロジェクトセッション | `strictSlotChildren` は仮想 TypeScript による検査を生成する。ランタイムコードの生成は行わない         |

<span id="flag-contracts"></span>

## フラグごとの契約

| フラグ               | 有効にする場面                                                                                                                                               | 最小限の検証                                                                                                                                     | フラグの対象外                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| `patternedTemplate`  | RFC [#823](https://github.com/vuejs/rfcs/pull/823) の `v-match` / `v-when` 分岐をテンプレートで使う場合                                                      | `v-match` の直下に子を持つ有効なコンポーネントと、フラグ無効時に `experimentals.patternedTemplate` を報告するコンポーネントをコンパイルする      | Content Mapper の設定伝達、エディターのナビゲーション全体の検証、or-pattern の束縛              |
| `inTagComment`       | 対象の prop の近くなど、開始タグ内に `@vue-expect-error` のような行単位の注釈を置く必要がある場合                                                            | 注釈付きコンポーネントを解析し、コメントが `root.comments` に保持され、出力コードは変わらないことを確認する                                      | ブラウザー内 DOM テンプレート、ランタイムのコメント、通常の `comments` コンパイラオプション     |
| `selfComponent`      | 再帰的な SFC で、`name` やファイル名の推論だけに頼らず、RFC [#833](https://github.com/vuejs/rfcs/pull/833) の自己参照を使う場合                              | `componentName` または SFC メタデータを指定して `<Self />` をコンパイルし、`Self` という名前のローカルインポートが参照先にならないことを確認する | JSX、render 関数、小文字の `<self>`                                                             |
| `strictSlotChildren` | ライブラリやアプリケーションが RFC [#734](https://github.com/vuejs/rfcs/pull/734) の型付きスロット子要素の契約を公開し、`vize check` や LSP で診断を得る場合 | 有効なデフォルト・名前付きスロットのタプルと、TypeScript が拒否する不正な子要素を型チェックする                                                  | ランタイムの描画、制約のない `any` のスロット契約、組み込み・動的コンポーネントの子要素の型付け |
| `serverScript`       | ホスト側の統合が server-script コンパイラの実験機能を管理し、検証している場合                                                                                | 有効時にだけ、ネイティブの `experimentalServerScript` 真偽値が渡されることを確認する                                                             | ホスト側が仕様を文書化するまでの、公開 server-script 構文の意味                                 |
| `vapor`              | 安定版の `compiler.vapor` に移行する前に、SFC の Vapor 出力を試す場合                                                                                        | `compiler.vapor` と直接指定する `vize({ vapor })` が未設定の場合にだけ、`experimentals.vapor: true` が Vapor を選ぶことを確認する                | プロジェクト全体に適用する安定版の Vapor 設定方針                                               |
| `jsxVapor`           | 安定版の `compiler.jsxMode` に移行する前に、JSX/TSX の Vapor 出力を試す場合                                                                                  | `jsxMode` が未設定の場合にだけ、`experimentals.jsxVapor: true` が JSX 出力の既定値を Vapor にすることを確認する                                  | ファイルごとの `"use vue:*"` ディレクティブと、安定版の JSX バックエンドの設定方針              |

RFC [#831](https://github.com/vuejs/rfcs/pull/831) も他の RFC フラグと同じ形式で有効化します。
ただし、対象はパーサーとツールの処理だけです。属性の近くにある `//` 注釈を保持しますが、ランタイムのコメントは出力しません。

<span id="direct-api-fields"></span>

## ネイティブ API の設定項目

通常のアプリケーションでは `experimentals` を設定してください。
設定を自前で解決する低レベルの統合では、ネイティブコンパイラの項目を直接指定できます。

```ts
import {
  compile,
  compileSfc,
  compileSfcBatchWithResults,
  compileVapor,
  parseTemplate,
} from "@vizejs/native";

compile(templateSource, {
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  experimentalServerScript: true,
  componentName: "TreeNode",
});

compileVapor(templateSource, {
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  componentName: "TreeNode",
});

parseTemplate(templateSource, {
  experimentalInTagComments: true,
});

compileSfc(sfcSource, {
  filename: "TreeNode.vue",
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  experimentalStrictSlotChildren: true,
  experimentalServerScript: true,
});

compileSfcBatchWithResults(files, {
  experimentalInTagComments: true,
  experimentalPatternedTemplate: true,
  experimentalSelfComponent: true,
  experimentalStrictSlotChildren: true,
  experimentalServerScript: true,
});
```

これらの項目は解決済みの真偽値を受け取ります。別名、`{}` の設定オブジェクト、共通設定の優先順位は解釈しません。
呼び出し元がそれらの解決ルールを管理する、統合の境界で使ってください。

<span id="config-recipes"></span>

## 設定例

意図した動作を検証できる、最小限のフラグだけを有効にします。
無関係な RFC の設定を、一つのプロジェクト用スイッチにまとめないでください。
機能が安定版の `compiler` またはツールのオプションに移るまでは、共通設定で実験フラグを既定で有効にしません。

共通設定で、一つの RFC 提案だけを有効にする例:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    inTagComment: true,
  },
});
```

共通設定では有効にしたまま、一時的に特定のプラグインだけ無効にする例:

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      experimentals: {
        inTagComment: false,
        patternedTemplate: null,
      },
    }),
  ],
});
```

設定の解決を済ませた低レベルの統合:

```ts
compile(templateSource, {
  experimentalInTagComments: resolvedExperimentals.inTagComments,
  experimentalPatternedTemplate: resolvedExperimentals.patternedTemplate,
});
```

ネイティブ API の呼び出しには、`intagComment` のような互換性のための別名や、`{}` のような設定オブジェクトを渡しません。
コンパイラを呼ぶ前に解決してください。

<span id="failure-examples"></span>

## 拒否される例

`patternedTemplate` は構造が不正な場合、処理を拒否します。

```vue
<template v-match="status"></template>

<template v-match="status">
  <p v-when:ready="'ready'">Ready</p>
  <p v-when.once="'ready'">Ready again</p>
</template>
```

`inTagComment` は、属性に別の式の文法を追加する機能ではありません。

```vue
<template>
  <LegacySelect :label="'// this is an attribute value, not an in-tag comment'" />

  <LegacySelect
    :// not a directive argument comment
    selected-id="abc"
  />
</template>
```

<span id="slot-cardinality"></span>

## スロットの子要素の個数

RFC #734 の戻り値の形は、TypeScript で子要素の個数を表す契約として保持します。

| スロットの戻り値の契約                       | Vize の仮想 TypeScript での意味                          |
| -------------------------------------------- | -------------------------------------------------------- |
| `() => HTMLInputElement`                     | input 子要素が一つ。単一の子要素を返す省略形も受け付ける |
| `() => HTMLInputElement[]`                   | input 子要素がゼロ個以上                                 |
| `() => [HTMLInputElement, HTMLInputElement]` | タプルの順序どおりに input 子要素がちょうど二つ          |
| `() => (typeof TabItem)[]`                   | `TabItem` コンポーネントの子要素がゼロ個以上             |
| `() => [typeof TabItem, HTMLButtonElement]`  | `TabItem` の子要素が一つ、その後に button 子要素が一つ   |

<span id="implementation-coverage"></span>

## 実装の検証範囲

| 対象                         | ドキュメントを変更する前に必要な検証                                                                             |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| 既定で無効になる動作         | フラグを無効にした解析、コンパイル、check、LSP の経路が、記載された無効時の動作を報告または維持する              |
| 設定の解決                   | 共通設定、Vite プラグインに直接渡す値、別名、`false`、`null`、`{}` を、ネイティブ API の真偽値とは分けて検証する |
| ネイティブのテンプレート API | `compile`、`compileVapor`、`parseTemplate` が、記載された `experimental*` 真偽値だけを受け付ける                 |
| ネイティブの SFC API         | `compileSfc`、`compileSfcBatch`、`compileSfcBatchWithResults` が、ファイル単位でもバッチ処理でも同じ真偽値を渡す |
| 型チェック API               | `strictSlotChildren` を、ランタイム出力のスナップショットではなく、仮想 TypeScript の診断で検証する              |
| 対応範囲の境界               | 未対応の RFC の動作には、非対応を確認する検証か明示的な記載がある。近い構文が動くことから対応済みと推測させない  |

<span id="release-safety-checklist"></span>

## リリース前の確認

リリースノートに実験機能への対応を記載する前に、出荷する API やコマンドで以下を確認します。

- 公開設定のキーとネイティブ API の直接指定項目が、既定では無効になることを検証している。
- 別名は、推奨名ではなく互換性のために受け付ける名前として記載している。
- Vite プラグインに直接渡す値で、共通設定を有効化でき、明示的に無効化もできる。
- `false`、`null`、`true`、`{}` が、記載された有効・無効の意味を維持している。
- `vapor` や `jsxVapor` のような出力先の代替フラグより、安定版の `compiler` オプションが優先される。
- RFC の例に、有効な例と、フラグ無効時または不正な構造に対する診断例の両方がある。
- patterned-template のエディターナビゲーションや、動的コンポーネントの strict slot 対応など、未対応の動作を明記している。
