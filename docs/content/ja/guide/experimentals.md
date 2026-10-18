---
title: 実験的な機能
---

<!-- Reviewed translation; source: guide/experimentals.md -->

<span id="experimentals"></span>

# 実験的な機能

`experimentals` は、提案段階の Vue 構文、ツール向けの型検査、開発中のコンパイル方式を試すための共有設定です。
すべての機能は既定で無効です。試したい機能だけを、個別に有効にしてください。
安定した `compiler` オプションや Vue ランタイムの `features` とは別の設定です。
同じコンパイル方式を安定版のオプションでも指定できる場合は、安定版を使ってください。

このページでは、次の機能の設定値、有効時と無効時の挙動、優先順位を説明します。

- Vue RFC [#823](https://github.com/vuejs/rfcs/pull/823) のパターンによるテンプレートの分岐
- Vue RFC [#831](https://github.com/vuejs/rfcs/pull/831) のタグ内コメント
- Vue RFC [#833](https://github.com/vuejs/rfcs/pull/833) の `<Self>` による自身の参照
- Vue RFC [#734](https://github.com/vuejs/rfcs/pull/734) のスロットの子要素の型検査
- サーバースクリプト、SFC Vapor、JSX Vapor など、Vize の実験的なコンパイル方式

まず推奨設定と対応範囲を確認し、必要な設定キーを一つずつ追加してください。
RFC ごとの診断、直接呼び出せる API、実装の制限、未対応の構文は、[Vue RFC の実験機能の詳細](./experimentals-vue-rfcs.md)を参照してください。

## 推奨設定

新しい TypeScript 設定では、推奨する camelCase の名前を使ってください。

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    patternedTemplate: true, inTagComment: true,
    selfComponent: true, strictSlotChildren: true,
    serverScript: false, vapor: null, jsxVapor: {},
  },
});
```

JSON と Pkl でも同じ値を指定できます。
`{}` は機能を有効にする値です。将来、機能ごとの設定項目を追加できるよう、オブジェクトの形式を残したい場合に使います。

```json
{
  "experimentals": {
    "patternedTemplate": true,
    "inTagComment": true,
    "strictSlotChildren": {},
    "vapor": false
  }
}
```

```pkl
amends "node_modules/vize/pkl/vize.pkl"

experimentals {
  patternedTemplate = true
  inTagComment = true
  strictSlotChildren = new Mapping {}
  vapor = false
}
```

<span id="スイッチ値"></span>

## 設定値

各キーに指定する値で、機能の有効・無効を切り替えます。

| 値 | 意味 |
| --- | --- |
| 省略 | 無効 |
| `false` | 無効。共有設定で有効にした機能を、個別に無効にする場合も使えます。 |
| `null` | 無効。`null` を含む JSON 設定を生成する場合に使えます。 |
| `true` | 有効 |
| `{}` | 有効。将来、機能ごとの設定項目を追加できるオブジェクト形式です。 |

キーの省略、`false`、`null` は無効、`true` と `{}` は有効です。
型付きの TypeScript と Pkl の設定では、この四つの値だけを使ってください。
互換性のため、設定の読み込み処理は JSON に存在する `false` と `null` 以外の値も有効とみなしますが、この形式は推奨していません。

## 優先順位

共有の `vize.config.*` は、プロジェクト設定を読み込むツールの既定値になります。
Vite プラグインに直接渡したオプションは、共有設定より優先します。
明示的に無効にする指定も、共有設定を上書きします。

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      experimentals: {
        patternedTemplate: false,
        inTagComment: true,
      },
    }),
  ],
});
```

この例では、`patternedTemplate: false` が共有設定で有効にした機能を、このプラグインだけで無効にします。
`inTagComment: true` は、同じプラグインでタグ内コメントの解析を有効にします。

安定版のコンパイラオプションがある場合は、そちらが優先します。

- `vize({ vapor })`、`compiler.vapor`、`experimentals.vapor` の順に適用します。
- `vize({ jsxMode })`、`compiler.jsxMode`、`experimentals.jsxVapor` の順に適用します。
- `experimentalPatternedTemplate` などの解決済みフィールドは、低レベルのコンパイラ設定です。設定の解決を自身で行う統合だけが、直接使ってください。

<span id="surface-matrix"></span>

## 対応範囲と制限

| 設定キー | 有効時の挙動 | 無効時の挙動 | 適用される場所 | 制限 |
| --- | --- | --- | --- | --- |
| `patternedTemplate` | `v-match` と、その直下にある `v-when` 分岐を解析してコンパイルします。 | `v-match`、`v-when`、`v-case` には、機能を有効にする設定が必要だと報告します。 | DOM、SSR、Vapor、SFC、WASM のコンパイル API | Canon とネイティブ Maestro は型の絞り込みと網羅性検査に対応します。Content Mapper とナビゲーションの全組み合わせの検証は未完了です。 |
| `inTagComment` | 開始タグ内の `//` コメントを解析し、ツール向けに保持します。 | 開始タグ内の `//` を不正な構文として報告します。 | パーサー、DOM、SSR、Vapor、SFC、WASM のコンパイル API | 実行時の出力は生成しません。ブラウザーが先に解析する DOM 内のテンプレートは対象外です。 |
| `selfComponent` | 名前が完全に一致する `<Self>` を、自身のコンポーネントとして扱います。 | `<Self>` を通常のコンポーネントタグとして扱います。 | DOM、SSR、Vapor、SFC、WASM のコンパイル API | レンダー関数と JSX は対象外です。 |
| `strictSlotChildren` | 仮想 TypeScript コードに、スロットの子要素の型検査を生成します。 | 子要素の型検査は生成しません。 | `vize check`、LSP とプロジェクトの型検査 API | 任意のキーを許すスロットや `any` は、TypeScript の通常の許容範囲のままです。 |
| `serverScript` | ネイティブコンパイラとの統合に `experimentalServerScript` を渡します。 | ネイティブの設定値は `false` のままです。 | このフィールドを公開するネイティブコンパイラとの統合 | ランタイムが意味を定めるまでは、統合先が挙動を定義します。 |
| `vapor` | 安定版の設定がなければ、SFC を Vapor でコンパイルします。 | SFC は、安定版の設定または既定の方式でコンパイルします。 | Vite プラグイン、パッケージのビルド設定 | `vize({ vapor })` と `compiler.vapor` が優先します。 |
| `jsxVapor` | 安定版の JSX 設定がなければ、JSX/TSX を Vapor でコンパイルします。 | JSX/TSX は、安定版の設定または既定の方式でコンパイルします。 | Vite プラグイン、パッケージのビルド設定 | `vize({ jsxMode })` と `compiler.jsxMode` が優先します。 |

API ごとの対応範囲、最小の検証例、低レベルのネイティブ API の使い方は、[実験機能のリファレンス](./experimentals-reference.md)を参照してください。

<span id="flag-reference"></span>

## 設定キーと互換名

| 設定キー | 提案・実装元 | 解決先のフィールド | 互換名 |
| --- | --- | --- | --- |
| `patternedTemplate` | Vue RFC [#823](https://github.com/vuejs/rfcs/pull/823) | `experimentalPatternedTemplate` | `pattenedTemplate` |
| `inTagComment` | Vue RFC [#831](https://github.com/vuejs/rfcs/pull/831) | `experimentalInTagComments` | `intagComment` |
| `selfComponent` | Vue RFC [#833](https://github.com/vuejs/rfcs/pull/833) | `experimentalSelfComponent` | JSON の `self_component` |
| `strictSlotChildren` | Vue RFC [#734](https://github.com/vuejs/rfcs/pull/734) | `experimentalStrictSlotChildren` | JSON の `strict_slot_children` |
| `serverScript` | サーバースクリプトのコンパイラ実験 | `experimentalServerScript` | JSON の `"server script"`、`server_script` |
| `vapor` | SFC の Vapor 出力 | `compiler.vapor` の設定がない場合に適用 | なし |
| `jsxVapor` | JSX/TSX の既定の Vapor 出力 | `compiler.jsxMode` の設定がない場合に適用 | なし |

推奨する名前と互換名を同時に設定しないでください。
互換名は古い設定を引き続き読み込むためにあります。両方を指定すると、互換名を解決する順序に挙動が左右されます。
新しい設定では、`patternedTemplate`、`inTagComment`、`selfComponent`、`strictSlotChildren`、`serverScript`、`vapor`、`jsxVapor` を使ってください。

<span id="patterned-templates"></span>

<span id="パターン付きテンプレート"></span>

## パターンによるテンプレートの分岐

`patternedTemplate` は、Vue RFC #823 の `v-match` と、その直下に置く `v-when` の分岐を有効にします。

```ts
export default defineConfig({
  experimentals: {
    patternedTemplate: true,
  },
});
```

```vue
<script setup lang="ts">
type Entry =
  | { kind: "article"; data: { title: string; published: boolean } }
  | { kind: "draft"; reason: string }
  | { kind: "archived"; id: string };

const entry = ref<Entry>({ kind: "draft", reason: "editing" });
</script>

<template v-match="entry">
  <article v-when="{ kind: 'article', data: const article } if (article.published)">
    {{ article.title }}
  </article>
  <p v-when="{ kind: 'draft' } | { kind: 'archived' }">Hidden</p>
  <p v-when="_">No published article</p>
</template>
```

照合する式は、一度だけ評価します。
分岐は上から順に調べ、最初に一致した分岐だけを描画します。後続の分岐には進みません。
`v-when="_"` は、他の分岐が一致しなかった場合のパターンです。
条件を付けない最上位のフォールバックは、一つだけ、最後の分岐に置く必要があります。

対応するパターン:

| パターン | 例 | 注意事項 |
| --- | --- | --- |
| リテラル | `v-when="'ready'"`、`v-when="404"` | 厳密等価で比較します。 |
| 値 | `v-when="Status.Active"` | 識別子とメンバー式は、`NaN` を含む SameValueZero で比較します。 |
| ワイルドカード | `v-when="_"` | 新しい変数を作らずに一致します。 |
| const による束縛 | `v-when="const value"` | `let` と `var` は使えません。 |
| オブジェクト | `v-when="{ kind: 'ok', value: const result }"` | 余分なプロパティがあっても一致します。 |
| オブジェクトの省略記法 | `v-when="{ kind: 'ok', const data }"` | `{ const data }` は `{ data: const data }` と同じです。 |
| オブジェクトの残余 | `v-when="{ kind: 'error', ...const payload }"` | 残余の束縛にも `const` が必要です。単独の `...` も使えます。 |
| 配列・タプル | `v-when="[const first, ...const rest]"` | `Array.isArray` で配列と判定される必要があります。残余があれば、追加の要素も許します。 |
| または | `v-when="'idle' \| 'loading'"` | 左から右に調べます。それぞれの選択肢の中で変数を束縛する構文は未対応です。 |
| as による束縛 | `v-when="{ kind: 'ok', const data } as entry"` | 入れ子の束縛に加えて、一致した値全体も束縛します。 |
| 条件 | `v-when="{ kind: 'error', error: const e } if (e.retriable)"` | パターンが一致した後に条件を評価します。 |

`v-case` は、古い Vize の実験機能との互換名として残っています。新しいテンプレートでは `v-when` を使ってください。
RFC #823 で議論されている省略記法の候補は、Vize の公開構文ではありません。`?=`、`|=`、`~=` は使えません。

不正な位置に書いた構文は、黙ってコンパイルせずに診断します。
`v-when` は `v-match` の直下に置き、`v-match` には少なくとも一つの分岐が必要です。
`v-when` にディレクティブの引数や修飾子は付けられません。
機能が無効な場合は、`experimentals.patternedTemplate` を有効にする必要があると報告します。
Canon は、機能を有効にすると、分岐ごとの型の絞り込み、未網羅のエラー、到達不能な分岐の警告に対応します。
残るコンパイラとエディターの制限は、RFC の詳細ページで確認してください。

<span id="in-tag-comments"></span>

## タグ内のコメント

`inTagComment` は、Vue RFC #831 の開始タグの属性一覧に書いた `//` コメントを解析します。

```ts
export default defineConfig({
  experimentals: {
    inTagComment: true,
  },
});
```

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

コメントは、タグ名の後、属性やディレクティブを書き終えた位置の間、最後の属性の後で `>` や `/>` より前に置けます。
行末に `//` を書いた場合は、タグを閉じる記号を次の行に置いてください。
同じ行にある `>` や `/>` はコメントの内容として読み込まれます。
Vize は元のテキストを、ツール向けにテンプレートのルートの `comments` 一覧に保持します。
子ノードにはせず、実行時の出力も生成しません。属性値の中にある `//` は通常の文字列です。

```vue
<template>
  <a href="https://example.test/path//segment">Link</a>
</template>
```

通常のテンプレートの `comments` コンパイラオプションは、この構文の有効・無効には影響しません。
機能が無効な場合、パーサーは開始タグ内の不正なスラッシュとして報告します。
この構文は、SFC やツールがテンプレートを解析する処理向けです。
ブラウザーの HTML パーサーが先に読み込む DOM 内のテンプレートでは、使える前提にしないでください。

<span id="self-component"></span>

## 自身のコンポーネントを参照する

`selfComponent` は、Vue RFC #833 の `<Self>` を、自身のコンポーネントを再帰的に参照するタグとして予約します。

```ts
export default defineConfig({
  experimentals: {
    selfComponent: true,
  },
});
```

```vue
<script setup lang="ts">
defineProps<{
  node: { id: string; children: Array<{ id: string; children: unknown[] }> };
}>();
</script>

<template>
  <li>
    {{ node.id }}
    <ul v-if="node.children.length">
      <Self v-for="child in node.children" :key="child.id" :node="child" />
    </ul>
  </li>
</template>
```

SFC のビルドでは、コンパイラのメタデータまたはファイル名から自身のコンポーネント名を取得し、`<Self>` をその名前に解決します。
SFC のファイル名がない直接のテンプレート API では、`componentName` を渡せます。
`selfComponent` が無効なら、`<Self>` は通常のコンポーネントタグのままで、ローカルの `Self` というコンポーネントも通常どおり扱います。
有効な場合、名前が完全に一致する `<Self>` は、ローカル、import したもの、グローバルのいずれでも、`Self` という名前のコンポーネントより優先します。

この設定は、DOM、SSR、Vapor のコンパイルに渡されます。
有効でコンポーネント名がある場合、生成したコンポーネントの解決処理は、その名前と Vue の自身の参照を示すヒントを使います。
予約するタグは大文字と小文字を区別します。`<Self>` は特別ですが、`<self>` は通常のタグです。

<span id="strict-slot-children"></span>

## スロットの子要素を型検査する

[Playground](https://vizejs.dev/play/?tab=canon) の Canon では、**Experimental** からこの設定を有効にできます。
サンプルは、button だけを受け付けるスロットに input を渡します。
TypeScript（Monaco）を有効にすると、生成された子要素の型を TypeScript が検査して診断を出します。Virtual TS タブでは型検査のコードも確認できます。
これはブラウザーで単一ファイルを検査する機能です。CLI の Corsa によるプロジェクト全体の型検査とは、検査範囲が異なります。

`strictSlotChildren` は、Vue RFC #734 のスロットの子要素に対する、仮想 TypeScript コードの型検査を有効にします。
対象は `vize check`、LSP とプロジェクトの型検査 API、型宣言を扱う仮想コードです。
通常のテンプレートのコンパイルに、実行時の挙動は追加しません。

```ts
export default defineConfig({
  experimentals: {
    strictSlotChildren: true,
  },
});
```

コンポーネントは、`__vizeSlots` でスロットの型を公開できます。

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => (typeof TabItem)[];
    footer: () => [HTMLButtonElement];
  };
};
```

Vize は、渡された子要素をその型と比較します。

```vue
<template>
  <Tabs>
    <TabItem />
    <div />

    <template #footer>
      <button>Save</button>
    </template>
  </Tabs>
</template>
```

機能が有効な場合、Vize はスロットの子要素の型検査を合成し、TypeScript が渡されたノードとスロットの戻り値の型を比較できるようにします。
ネイティブ DOM の子要素は対応する `HTML*Element`、コンポーネントの子要素は `typeof ComponentRef`、テキストや補間は `string` に対応します。
名前付きスロットと既定のスロットの両方を検査します。`defineSlots` も同じ形式のマーカーを公開します。
値を持たないコメントや、構造だけを表すラッパーノードは検査しません。

この検査はツール向けの制約です。違反は `vize check` または言語サーバーの TypeScript 診断として報告します。
Vue テンプレートをコンパイルするだけのビルドでは、検査を行いません。
任意のキーを許すインデックスシグネチャや `any` を使うスロットの型は、TypeScript の通常の許容範囲になります。

<span id="server-script"></span>

## サーバースクリプト

`serverScript` は、サーバースクリプトのコンパイラ実験を明示的に検証する統合先だけで有効にしてください。

```ts
export default defineConfig({
  experimentals: {
    serverScript: true,
  },
});
```

この設定は、ネイティブコンパイラの `experimentalServerScript` フィールドに解決されます。
キーの省略、`false`、`null` の場合、そのフィールドは無効のままです。
一部のランタイムでは、コンパイラが実験に対応するまで、切り替えのための設定だけを予約しています。
アプリケーションで、安定した構文として依存しないでください。
フレームワークがこの設定を使い始めた場合は、その統合先のドキュメントを優先してください。

## Vapor

`vapor` は、安定版の `compiler.vapor` と直接渡した `vize({ vapor })` が未設定の場合に、SFC を実験的な Vapor バックエンドでコンパイルします。

```ts
export default defineConfig({
  experimentals: {
    vapor: true,
  },
});
```

Vapor をプロジェクトの通常のビルド方式として使う場合は、`compiler.vapor` または `vize({ vapor })` を使ってください。
これらの値は、`experimentals.vapor` が `true` でも優先します。
`vize({ vapor: false })` は、そのプラグインだけで Vapor を明示的に無効にする指定です。
`experimentals.vapor` は、試験実行、互換性の確認、実験的なフォールバックの検証に使います。

## JSX Vapor

`jsxVapor` は、安定版の `compiler.jsxMode` と直接渡した `vize({ jsxMode })` が未設定の場合に、JSX/TSX ファイルの既定の出力を Vapor にします。

```ts
export default defineConfig({
  experimentals: {
    jsxVapor: true,
  },
});
```

プロジェクト全体の通常のビルド方式として使う場合は、`compiler.jsxMode: "vapor"` または `vize({ jsxMode: "vapor" })` を使ってください。
これらの値は `experimentals.jsxVapor` より優先し、`jsxMode: "vdom"` は明示的に Vapor を無効にする指定です。
コンポーネントごとの `"use vue:vapor"` と `"use vue:vdom"` ディレクティブは、そのファイルの意図を指定するため、既定の方式を上書きできます。

<span id="direct-api-fields"></span>

## 低レベル API のフィールド

ほとんどのアプリケーションでは、`experimentals` を設定します。
低レベルの統合では、解決済みの真偽値を `compile`、`compileVapor`、`parseTemplate`、`compileSfc`、`compileSfcBatch`、`compileSfcBatchWithResults` に渡せます。
これらのフィールドは、互換名、`{}` のオブジェクト形式、共有設定の優先順位を解釈しません。
詳しくは、[実験機能の低レベル API リファレンス](./experimentals-reference.md#direct-api-fields)を参照してください。
