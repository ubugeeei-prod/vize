---
title: Vue RFC の実験的な機能
description: Vue に入る前の 4 つの RFC を Vize で試すための設定、具体例、診断と現在の制限。
---

<!-- Reviewed translation; source: guide/experimentals-vue-rfcs.md -->

<span id="vue-rfc-experimental-details"></span>

# Vue RFC の実験的な機能

Vize では、Vue に入る前の 4 つの RFC を試せます。どの機能も専用のフラグを有効にするまでは無効で、
1 つを有効にしても他の機能は有効になりません。設計の参照先は GitHub の RFC です。このページでは、
Vize が現在実装している範囲と、その制限を説明します。試すときは、まず
[機能を有効にする](#opt-in-contract)を参照してください。

> [!WARNING]
> これらの機能は議論中の提案に基づいています。構文や動作は RFC に合わせて変わる可能性があり、
> これらを使ったコードは Vize でしかコンパイルできません。

<span id="opt-in-contract"></span>

## 機能を有効にする

| RFC | 追加される機能 | フラグ | コンパイラーのフィールド | 対応範囲 |
| --- | --- | --- | --- | --- |
| [#823](https://github.com/vuejs/rfcs/pull/823) | テンプレートでのパターンマッチ（`v-match` / `v-when`） | `patternedTemplate` | `experimentalPatternedTemplate` | DOM、SSR、Vapor、`vize check`、言語サーバーの診断 |
| [#831](https://github.com/vuejs/rfcs/pull/831) | 開始タグ内の `//` コメント | `inTagComment` | `experimentalInTagComments` | パーサーとすべてのコンパイル対象。実行時の出力はなし |
| [#833](https://github.com/vuejs/rfcs/pull/833) | 現在のコンポーネントを表す `<Self>` | `selfComponent` | `experimentalSelfComponent` | DOM、SSR、Vapor |
| [#734](https://github.com/vuejs/rfcs/pull/734) | スロットに渡す子要素の型チェック | `strictSlotChildren` | `experimentalStrictSlotChildren` | `vize check` と言語サーバーのみ |

`vite.config.ts` の `vue.experimentals` でフラグを有効にします。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    experimentals: {
      inTagComment: true,
      patternedTemplate: true,
      selfComponent: true,
      strictSlotChildren: true,
    },
  },
});
```

フラグが無効なときは、RFC がない場合と同じように扱います。`v-match`、`v-when`、`v-case` は有効化が
必要だと報告し、タグ内の `//` は不正な構文になり、`<Self>` は通常のコンポーネントタグになり、
スロットの子要素の型は生成しません。

コンパイラーを直接呼び出す連携では、確定した真偽値を渡します。

```ts
compile(templateSource, { experimentalPatternedTemplate: true });
compileVapor(templateSource, { experimentalSelfComponent: true, componentName: "TreeNode" });
parseTemplate(templateSource, { experimentalInTagComments: true });
compileSfc(sfcSource, { filename: "TreeNode.vue", experimentalStrictSlotChildren: true });
```

`compile`、`compileVapor`、`parseTemplate`、`compileSfc*` に直接渡すフィールドは、最終的な真偽値だけを
受け付けます。`pattenedTemplate` や `intagComment` のような別名や、`{}` 形式の指定は解釈しません。
Vite プラグインに直接渡した値は、明示的な `false` を含めて共有設定より優先されます。優先順位は
[実験的な機能](./experimentals.md)、すべての入口は[実験機能のリファレンス](./experimentals-reference.md)を
参照してください。

<span id="patterned-templates"></span>

## パターンによるテンプレートの分岐

`patternedTemplate` は RFC #823 の `v-match` / `v-when` 構文を追加します。コンテナーに `v-match` を置き、
直下の子要素それぞれに `v-when` を書きます。Vize は対象の値を 1 回だけ評価し、分岐を順に試して、
最初に一致した分岐だけを描画します。

```vue
<script setup lang="ts">
type Result =
  | { status: "success"; data: { title: string; published: boolean } }
  | { status: "error"; error: { message: string; retriable: boolean } }
  | { status: "loading" }
  | { status: "empty" };

const result = ref<Result>({ status: "loading" });
</script>

<template v-match="result">
  <ArticleView
    v-when="{ status: 'success', data: const article } if (article.published)"
    :article="article"
  />
  <RetryBanner
    v-when="{ status: 'error', error: const error } if (error.retriable)"
    :error="error"
  />
  <p v-when="{ status: 'empty' } | { status: 'loading' }">Waiting for content.</p>
  <ErrorBanner v-when="{ status: 'error', error: const error }" :error="error" />
  <p v-when="_">Unpublished article.</p>
</template>
```

動作のポイント:

- `const` で宣言した束縛は分岐の中だけで使えます。分岐の要素、その属性、ガード、子要素からは
  参照でき、兄弟の分岐からは参照できません。
- プロパティは試す分岐ごとに 1 回だけ読み、ガードは描画される分岐と同じ値を参照します。ガードは
  パターンが一致した後にだけ実行されます。
- DOM、SSR、Vapor のどれでも、`v-match` は余分なフラグメントやリストの目印を出力しません。ルートが
  1 つの場合は属性のフォールスルーが保たれ、`Transition` や `KeepAlive` の中でも動作し、ノード単位で
  ハイドレーションされます。
- 分岐の `:key` は、その分岐の束縛を使って評価します。
- SFC の外側の `<template v-match>` にも対応します。外部の `src` テンプレートとプリプロセッサーは
  エラーになります。

| パターン | 例 | 一致する条件 |
| --- | --- | --- |
| リテラル | `v-when="'ready'"`、`v-when="404"` | 厳密に等しい |
| 値 | `v-when="Status.Ready"` | 識別子やメンバー式の値と等しい（SameValueZero のため `NaN` も一致） |
| ワイルドカード | `v-when="_"` | 常に一致し、何も束縛しない |
| const 束縛 | `v-when="const value"` | 常に一致し、`value` を束縛する |
| オブジェクト | `v-when="{ kind: 'ok', value: const data }"` | 指定したプロパティが一致する。ほかのプロパティがあってもよい |
| オブジェクトの省略記法 | `v-when="{ kind: 'ok', const data }"` | `{ data: const data }` と同じ |
| オブジェクトの残り | `v-when="{ kind: 'error', ...const payload }"` | 残りの列挙可能な自身のプロパティを束縛する |
| 配列・タプル | `v-when="[const first, ...const rest]"` | 値が配列（`Array.isArray`）。rest があれば要素が多くてもよい |
| または | `v-when="'idle' \| 'loading'"` | いずれかの候補に一致する。左から順に試す |
| as 束縛 | `v-when="{ kind: 'ok' } as whole"` | パターンが一致し、値全体を `whole` に束縛する |
| ガード | `v-when="{ error: const e } if (e.retriable)"` | パターンが一致し、ガードが真になる |

<span id="patterned-diagnostics"></span>

### パターン構文の診断

Vize は誤った使い方を、未知のディレクティブとして素通りさせずに報告します。フラグが無効なときは、
有効化が必要だと報告します。

```vue
<template v-match="status">
  <p v-when="'ready'">Ready</p>
</template>
```

`v-when` は `v-match` のコンテナーの直下に置く必要があります。

```vue
<template v-match="status">
  <section>
    <p v-when="'ready'">Ready</p>
  </section>
</template>
```

ガードのない受け皿の分岐は 1 つだけで、最後に置く必要があります。

```vue
<template v-match="status">
  <p v-when="_">Other</p>
  <p v-when="'ready'">Ready</p>
  <p v-when="(_)">Again</p>
</template>
```

`v-match` には直下の分岐が 1 つ以上必要で、`v-when` にはディレクティブの引数や修飾子を付けられません。

```vue
<template v-match="status"></template>

<template v-match="status">
  <p v-when:ready="'ready'">Ready</p>
  <p v-when.once="'ready'">Ready again</p>
</template>
```

次の書き方もエラーになります。

- `let` と `var` の束縛はエラーになります。使える宣言は `const` だけです。
- `v-if`、`v-else-if`、`v-else`、`v-for`、`v-match` と同じ要素に置いた `v-when`。
- 古い `as const name` の書き方。`pattern as name` と書いてください。
- `v-case` と `v-case.default` は、以前の Vize の実験との互換用の別名としてだけ残っています。新しい
  テンプレートでは `v-when` と `_` を使ってください。

<span id="patterned-type-boundary"></span>

### パターンの型検査と制約

フラグを有効にすると、`vize check` はパターンを理解します。

- 分岐の束縛と対象の値は、一致した形に絞り込まれます。
- 網羅できていない場合は、元の対象の値の位置にエラーを報告し、`vize check` は失敗します。網羅を
  証明できないときは、ガードのない `v-when="_"` を追加してください。
- 到達しない分岐は警告で、チェックは失敗しません。
- ガードは網羅の証明に使われません。省略可能なプロパティは、残りの場合に `undefined` を含みます。
- 有限のオブジェクトやタプルの union、readonly 配列、rest 束縛に対応します。制限のないプリミティブ、
  `any`、`unknown`、union 型の値パターンは保守的に扱います。
- 「または」の候補の中での束縛は未対応です。束縛が必要な場合は分岐を分けてください。RFC の省略記法
  `?=`、`|=`、`~=` には対応していません。

| ツール | 現在の対応 |
| --- | --- |
| コンパイラー（DOM、SSR、Vapor） | RFC の構文すべて |
| `vize check` | 絞り込み、網羅のエラー、到達しない分岐の警告 |
| 言語サーバー | 構造と網羅の診断、ルートのパターン束縛のホバーと定義への移動。フラグを変えたらサーバーを再起動する |
| Playground | Croquis と Canon のチェックボックス。型の診断全体は Monaco の TypeScript ワーカーで行う |
| VS Code の文法 | フラグがなくても `v-when` のパターンを色分けする |
| 未対応 | Content Mapper への振り分け、ガード・入れ子のクロージャ・「または」の束縛での名前変更と補完（[#6176](https://github.com/ubugeeei-prod/vize/issues/6176)） |

上流の型ツールの受け入れ基準には、絞り込みと網羅性に加えてエディターでの移動と補完も含まれており、
これらは対応中です。Vize 以外で仮想 TypeScript を扱うホストは、`virtualTsHelpers` をアンビエント宣言
ファイルとして 1 回登録してください。

<span id="in-tag-comments"></span>

## 開始タグ内のコメント

`inTagComment` は、開始タグの属性の並びの中に `//` コメントを書けるようにします（RFC #831）。
コメントはコンパイル時だけのもので、Vize はツールのために本文を残し、実行時のコードは出力しません。

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

- タグ名の後、属性と属性の間、最後の属性の後に置けます。
- 行末のコメントの後では、`>` や `/>` を次の行に書いてください。同じ行にあると `//` が閉じ記号まで
  コメントとして読み込みます。
- タグ名、属性名、ディレクティブ名や引数、修飾子、属性値の中には書けません。属性値の中の `//` は
  そのまま文字列です。

```vue
<template>
  <a href="https://example.test/a//b">Link</a>
</template>
```

ツール向けには、コメントは `CommentKind::InTag` として `root.comments` に保存されます。要素のプロパティや
子の `<!-- -->` ノードにはならず、`comments: false` でも削除されません。
ブラウザーが先に解析する DOM 内のテンプレートでは、このコメントを使わないでください。

<span id="self-component"></span>

## 自分自身のコンポーネントを参照

`selfComponent` は、`<Self>` というタグを現在のコンポーネントとして予約します（RFC #833）。
再帰的なコンポーネントを簡単に書けます。

```vue
<template>
  <li>
    {{ node.id }}
    <ul v-if="node.children.length">
      <Self v-for="child in node.children" :key="child.id" :node="child" />
    </ul>
  </li>
</template>
```

| 状況 | `<Self>` の解決先 |
| --- | --- |
| コンパイラーのメタデータがある SFC のビルド | メタデータにある現在のコンポーネント名 |
| メタデータがない SFC のビルド | ファイル名から導いた名前 |
| テンプレート API を直接使う場合 | 指定した `componentName` |
| 名前がない場合 | 通常のコンポーネント解決 |

フラグを有効にすると、`<Self>` は `Self` という名前のローカルやインポートしたコンポーネントより優先され、
それらは隠れます。インポートに `Self` という名前を付けず、別名にしてください。

```vue
<script setup lang="ts">
import OtherSelf from "./OtherSelf.vue";
</script>

<template>
  <Self />
  <OtherSelf />
</template>
```

`<Self>` は大文字小文字を区別する完全一致で、`<self>` は特別扱いしません。props、属性、イベント、
ディレクティブ、ref、`v-if`、`v-for`、`v-show`、スロットは通常のコンポーネントと同じように動作し、
再帰には終了条件が必要です。render 関数と JSX は対象外です。

SFC のファイル名がない低レベルのテンプレート呼び出しでは、名前を渡してください。

```ts
import { compile } from "@vizejs/native";

compile("<Self :node=\"node\" />", {
  componentName: "TreeNode",
  experimentalSelfComponent: true,
});
```

<span id="strict-slot-children"></span>

## スロットの子要素を型検査

`strictSlotChildren` は、親がスロットに渡す子要素を型チェックします（RFC #734）。影響するのは
`vize check` と言語サーバーだけで、コンパイル結果は変わりません。

コンポーネントは、各スロットが受け付ける内容を宣言します。

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => [typeof TabItem, HTMLButtonElement];
    footer: () => [HTMLButtonElement];
  };
};
```

SFC では `defineSlots` で同じ宣言ができます。

```ts
defineSlots<{
  default(): [typeof TabItem, HTMLButtonElement];
  footer(): [HTMLButtonElement];
}>();
```

渡した子要素は、この宣言と照合されます。

```vue
<template>
  <Tabs>
    <TabItem />
    <button>Open</button>

    <template #footer>
      <button>Save</button>
    </template>
  </Tabs>
</template>
```

上のデフォルトスロットは `__VizeProvidedSlotChildren<[typeof TabItem, HTMLButtonElement]>` として表され、
TypeScript がスロットの戻り値の型と比較します。合わない子要素は TypeScript のエラーになります。

```vue
<template>
  <Tabs>
    <div />
  </Tabs>
</template>
```

| 子要素 | 型 |
| --- | --- |
| `<button>` などのネイティブ要素 | `HTMLButtonElement` |
| SVG・MathML の要素 | より狭い型がない場合は `SVGElement` または `MathMLElement` |
| `<TabItem>` などインポートしたコンポーネント | `typeof TabItem` |
| テキストや補間 | `string` |
| 名前付きの `<template #footer>` | 名前付きスロットと照合 |
| コメントや子要素を増やさないラッパー | 対象外 |

| スロットの戻り値の型 | 意味 |
| --- | --- |
| `() => HTMLInputElement` | input の子要素が 1 つ |
| `() => HTMLInputElement[]` | input の子要素がいくつでも |
| `() => [HTMLInputElement, HTMLInputElement]` | input の子要素がちょうど 2 つ（順番どおり） |
| `() => (typeof TabItem)[]` | `TabItem` の子要素がいくつでも |
| `() => [typeof TabItem, HTMLButtonElement]` | `TabItem` が 1 つ、その後にボタンが 1 つ |

制限:

- スロットが受け付ける子の型が 1 種類のときは、`__VizeProvidedSlotChildren<__T>` は 1 つの子要素を
  `Only` と `[Only]` のどちらとしても受け付けます。
- 開いたインデックスシグネチャや `any` のスロットは `any` になり、厳密なチェックは行いません。
- 組み込みコンポーネント、動的コンポーネント、`KeepAlive`、`Teleport`、`Transition`、`TransitionGroup`、
  `Suspense` は、まだ厳密な子要素の型を提供しません。
- props によって宣言が変わる高度なコンポーネントライブラリーは、`__vizeResolveSlots` を公開できます。
  `defineSlots` を使うコンポーネントは、親のために `__vizeSlots` マーカーを書き出します。
- 必須スロット名のチェックは別の機能です。このフラグは子要素の型チェックだけを追加します。
