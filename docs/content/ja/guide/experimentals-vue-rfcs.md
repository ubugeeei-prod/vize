---
title: Vue RFC の実験的な機能
description: Vue RFC の機能を試すための設定、具体例、診断と未対応事項を確認する。
---

<!-- Reviewed translation; source: guide/experimentals-vue-rfcs.md -->

<span id="vue-rfc-experimental-details"></span>

# Vue RFC の実験的な機能

[実験的な機能](./experimentals.md)で紹介している Vue RFC の機能を、具体例とともに説明します。試したい機能のフラグを有効にし、対応する処理と未対応事項を確認してから利用してください。上流の RFC は設計の参照先です。対応する Vize のフラグを明示的に有効にしない限り、RFC の機能は有効になりません。

<span id="opt-in-contract"></span>

## 機能を有効にする設定

RFC のフラグはすべて独立しています。一つの提案を有効にしても、別の提案は有効になりません。

| RFC                                                                             | Vize の設定                        | コンパイラに直接渡す項目         | 対象                                                      | フラグが無効な場合                                                     |
| ------------------------------------------------------------------------------- | ---------------------------------- | -------------------------------- | --------------------------------------------------------- | ---------------------------------------------------------------------- |
| [#823](https://github.com/vuejs/rfcs/pull/823) パターンによるテンプレートの分岐 | `experimentals.patternedTemplate`  | `experimentalPatternedTemplate`  | DOM、SSR、Vapor、SFC、WASM のコンパイル API               | `v-match`、`v-when`、`v-case` の利用にフラグが必要であることを報告する |
| [#831](https://github.com/vuejs/rfcs/pull/831) 開始タグ内のコメント             | `experimentals.inTagComment`       | `experimentalInTagComments`      | パーサー、DOM、SSR、Vapor、SFC、WASM のコンパイル API     | 開始タグ内の `//` を不正なタグ構文として解析する                       |
| [#833](https://github.com/vuejs/rfcs/pull/833) 自分自身のコンポーネントを参照   | `experimentals.selfComponent`      | `experimentalSelfComponent`      | DOM、SSR、Vapor、SFC、WASM のコンパイル API               | `<Self>` を通常のコンポーネントタグとして扱う                          |
| [#734](https://github.com/vuejs/rfcs/pull/734) スロットの子要素を型検査         | `experimentals.strictSlotChildren` | `experimentalStrictSlotChildren` | `vize check`、LSP、仮想 TypeScript を使うプロジェクト API | 子要素に対する追加の型検査を生成しない                                 |

共通設定とプラグインの設定では `experimentals` を使います。コンパイラの項目を直接指定するのは、別名、優先順位、`{}` の設定オブジェクトを解決し、最終的な真偽値を確定している統合に限ります。

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    inTagComment: true,
    patternedTemplate: true,
    selfComponent: true,
    strictSlotChildren: true,
  },
});
```

```ts
compile(templateSource, { experimentalPatternedTemplate: true });
compileVapor(templateSource, { experimentalSelfComponent: true, componentName: "TreeNode" });
parseTemplate(templateSource, { experimentalInTagComments: true });
compileSfc(sfcSource, { filename: "TreeNode.vue", experimentalStrictSlotChildren: true });
```

`compile`、`compileVapor`、`parseTemplate`、`compileSfc*` のネイティブ API に直接渡す項目は、`pattenedTemplate` や `intagComment` のような別名を解釈せず、`{}` の設定オブジェクトも受け付けません。Vite プラグインに直接指定した値は共通設定より優先されます。明示的に無効にした場合も同じです。

<span id="current-scope"></span>

## 現在の対応範囲

| RFC  | 現在の Vize が提供する処理                                                                                 | 対応範囲の制約                                                                                                             |
| ---- | ---------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| #823 | パーサー、実行コードへの変換、分岐内の変数束縛、ガード、rest / as パターン、フラグ無効時や不正な配置の診断 | Canon は型の絞り込みと網羅性を検査する。エディターのナビゲーション全体とコンパイラの文法の統一には残作業がある             |
| #831 | 開始タグ内の `//` の解析、`root.comments` への元のテキストの記録、AST のコメントを保持するコンパイル処理   | 実行時の出力、子のコメントノード、ブラウザー内 DOM テンプレートには対応しない                                              |
| #833 | DOM、SSR、Vapor のコンパイルで、正確に一致する `<Self>` を現在のコンポーネントに解決する                   | render 関数や JSX のマクロは対象外。フラグ有効時は `Self` という名前のローカル・インポート済みコンポーネントより優先される |
| #734 | デフォルト・名前付きスロットに渡す子要素への仮想 TypeScript による型検査                                   | テンプレートのコード生成は変わらない。制約のないスロット契約と `any` には、TypeScript の寛容な検査が適用される             |

設定を渡す場所と検証項目は、[実験的な機能のリファレンス](./experimentals-reference.md)を参照してください。

<span id="patterned-templates"></span>

## パターンによるテンプレートの分岐

`patternedTemplate` は RFC #823 の `v-match` / `v-when` による詳細形式の構文を実装します。判定対象の式は一度だけ評価し、直下の分岐を記述順に検査します。最初に一致した分岐だけを描画します。

HTML テンプレートを直接記述した SFC では、最外周の `<template>` にも `v-match` を指定できます。直下の `v-when` は DOM・SSR・Vapor で内側の分岐と同じように動作します。最外周の `<template>` タグの式だけを変更した場合も、テンプレートの HMR の更新対象になります。解析結果の `template.content` は引き続きブロック本文のみです。この形式では、外部 `src`、プリプロセッサ言語、元のソース情報を失った記述子を拒否します。Canon は、後述のフラグを有効にすると分岐内の型の絞り込みと網羅性を検査します。結合したコンパイラ SFC のソースマップは、従来どおり script のみに対応します。

Croquis は `analyzeSfc(source, { experimentalPatternedTemplate: true })` と Playground のチェックボックスで、最外周と入れ子の分岐を解析できます。RFC のパーサーで分岐内の宣言、外側の値を参照するパターン、ガード、構文診断を記録し、HTML エンティティを含む元の位置を維持します。DOM・Vapor・SSR もこのパーサーを使い、`pattern as name` で変数を束縛します。旧形式の `as const name` は拒否します。

Canon も同じパーサーを使います。`typeCheck(source, { experimentalPatternedTemplate: true, includeVirtualTs: true })`、Playground の Canon のチェックボックス、`experimentals.patternedTemplate` を有効にした `vize check` で型検査できます。Native Maestro は同じワークスペース設定を読み、構文エラー、網羅漏れ、到達不能な分岐の警告を報告し、未保存の編集も再検査します。最外周のパターンで束縛した変数にマウスを重ねると絞り込んだ型を表示し、定義への移動では元の宣言位置を返します。ワークスペースのフラグを変更したら、言語サーバーを再起動してください。

VS Code の文法定義は `v-when` の値をパターンとして扱います。`const` / `as` の宣言、`_` のワイルドカード、rest、選択肢、ガードには参照用の文法と同じスコープ名が付くため、フラグの有無にかかわらず既存のテーマで色分けできます。Content Mapper への設定伝達と、ガード・入れ子のクロージャー・選択肢内の束縛を含む名前変更や補完の全組み合わせの検証は未完了です。[#6176](https://github.com/ubugeeei-prod/vize/issues/6176)で追跡します。

パターンのプロパティは、各分岐を試すときに一度だけ読みます。ガードと描画処理は、rest のコピーも含めて同じ値を使います。rest のコピーは構造全体が一致してから作ります。構造が一致しない分岐や、まだ試していない後続の分岐のガードは評価しません。`v-when` は `v-if`、`v-else-if`、`v-else`、`v-for`、`v-match` と同じ要素には指定できません。

分岐はリストではなく変数のスコープに変換するため、DOM・SSR・Vapor のいずれも、分岐のためのフラグメントやリストのマーカーを描画しません。ルートが一つの分岐では属性の引き継ぎを保ち、`Transition` や `KeepAlive` の子としても動作します。ハイドレーションでもノードが一致します。各分岐の `:key` は、その分岐で束縛した変数を参照できるスコープ内で評価します。

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

束縛した変数は、その分岐内だけで参照できます。`v-when` を指定した要素、その属性・ディレクティブ、ガードの式、子要素からは参照できますが、隣の分岐からは参照できません。

利用できるパターンは次のとおりです。

| パターン             | 例                                             | 実行時の検査                                                                      |
| -------------------- | ---------------------------------------------- | --------------------------------------------------------------------------------- |
| リテラル             | `v-when="'ready'"`, `v-when="404"`             | 厳密な等価比較で判定する                                                          |
| 値                   | `v-when="Status.Ready"`                        | 識別子またはメンバー式の値と SameValueZero で比較する。`NaN` も比較対象になる     |
| ワイルドカード       | `v-when="_"`                                   | 常に一致し、変数を束縛しない                                                      |
| const による変数束縛 | `v-when="const value"`                         | 常に一致し、その分岐の `value` に値を束縛する                                     |
| オブジェクト         | `v-when="{ kind: 'ok', value: const data }"`   | 指定した構造と一致するか検査する。余分なプロパティは許容する                      |
| オブジェクトの省略形 | `v-when="{ kind: 'ok', const data }"`          | `{ const data }` は `{ data: const data }` と同じ意味になる                       |
| オブジェクトの rest  | `v-when="{ kind: 'error', ...const payload }"` | 残りの列挙可能な独自プロパティを束縛する。変数名を付けない `...` も受け付ける     |
| 配列・タプル         | `v-when="[const first, ...const rest]"`        | `Array.isArray` による配列の判定を要求する。rest がある場合は追加の要素を許容する |
| または               | `v-when="'idle' \| 'loading'"`                 | 選択肢を左から右に試す                                                            |
| as による変数束縛    | `v-when="{ kind: 'ok' } as whole"`             | 一致した値を `whole` に束縛する                                                   |
| ガード               | `v-when="{ error: const e } if (e.retriable)"` | パターンが一致した後に実行する                                                    |

<span id="patterned-diagnostics"></span>

### パターン構文の診断

フラグが無効な場合、Vize は未知のディレクティブとして通さず、機能を明示的に有効にする必要があることを報告します。

```vue
<template v-match="status">
  <p v-when="'ready'">Ready</p>
</template>
```

`v-when` は `v-match` を指定した要素の直下に置く必要があります。

```vue
<template v-match="status">
  <section>
    <p v-when="'ready'">Ready</p>
  </section>
</template>
```

ガードなしで全体を受け付けるフォールバックは一つだけ指定でき、最後の分岐に置く必要があります。

```vue
<template v-match="status">
  <p v-when="_">Other</p>
  <p v-when="'ready'">Ready</p>
  <p v-when="(_)">Again</p>
</template>
```

`v-match` には、直下の分岐が少なくとも一つ必要です。`v-when` にディレクティブの引数や修飾子は指定できません。

```vue
<template v-match="status"></template>

<template v-match="status">
  <p v-when:ready="'ready'">Ready</p>
  <p v-when.once="'ready'">Ready again</p>
</template>
```

`let` と `var` による変数束縛は拒否します。現在、Vize が受け付ける宣言は `const` だけです。`v-when` の引数や修飾子も拒否します。`v-case` と `v-case.default` は過去の実験との互換性のためにだけ残している別名です。新しいテンプレートでは `v-when` と `_` を使ってください。

<span id="patterned-type-boundary"></span>

### パターンの型検査と制約

Canon は、分岐内で束縛した変数と元の判定対象の型を絞り込みます。網羅漏れはエラー、到達不能な分岐は警告として報告します。型検査のアルゴリズムは、検証済みの型宣言ファイル `pattern_matching.d.ts` として TypeScript のプログラムごとに共有し、各テンプレートには複製しません。

ガード付きの分岐は、網羅性の証明には使いません。省略可能なプロパティの存在検査では、値が `undefined` になる場合と、キー自体が存在しない場合を維持します。有限のオブジェクト・タプルのユニオン型、読み取り専用の配列、rest による変数束縛に対応します。値を有限に列挙できないプリミティブ型、`any`、`unknown`、ユニオン型の値パターンは保守的に扱います。網羅性を証明できない場合は、ガードなしの `v-when="_"` を使ってください。

網羅漏れは元の判定対象の位置に表示され、`vize check` が失敗します。到達不能な分岐の警告だけであれば成功します。WASM 単体は構造上の診断と仮想 TypeScript を提供します。完全な型診断は、Playground の Monaco TypeScript ワーカーまたはネイティブの型検査器が行います。他の仮想 TypeScript のホストでは、`virtualTsHelpers` をアンビエント宣言として一度だけ登録してください。

RFC 全体の型ツールに関する受け入れ条件には、エディターのナビゲーションや補完も含まれますが、そこまでの認定は未完了です。or パターンの選択肢内での変数束縛も未対応です。変数を束縛する必要がある場合は、分岐を分けてください。`?=`、`|=`、`~=` は公開している分岐構文ではありません。

<span id="in-tag-comments"></span>

## 開始タグ内のコメント

`inTagComment` は RFC #831 の、開始タグの属性一覧に記述する `//` コメントを実装します。コメントはコンパイル時だけのものです。Vize はパーサーやツールで元の記述を扱えるようにテキストを保持しますが、実行時のコードは出力しません。

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

コメントはタグ名の後、完全な属性・ディレクティブの間、最後の属性の後かつ `>` / `/>` の前に置けます。末尾にコメントを置く場合は、タグを閉じる記号を次の行に置いてください。同じ行の `>` や `/>` は、`//` のコメントの一部として読み取られます。タグ名、属性名、ディレクティブ名、引数、修飾子、属性値の途中には、この構文を使えません。

コメントは、テンプレートのルートの `comments` 一覧に `CommentKind::InTag` として保存されます。要素の属性でも、子の `<!-- ... -->` ノードでもありません。通常のテンプレートの `comments` 設定の影響も受けません。`comments: false` でも、ツール向けの開始タグ内コメントは保持されます。属性値内の `//` は通常のテキストとして扱います。

```vue
<template>
  <a href="https://example.test/a//b">Link</a>
</template>
```

この構文は、テンプレートを自分で解析する SFC やツールの処理向けです。ブラウザー内 DOM の生のテンプレートは、Vize より先にブラウザーが解析するため、開始タグ内コメントに依存しないでください。

<span id="self-component"></span>

## 自分自身のコンポーネントを参照

`selfComponent` は RFC #833 に基づき、タグ名が完全に一致する `<Self>` を、現在のコンポーネントを参照するための予約タグとして扱います。

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

名前の解決方法は次のとおりです。

| 利用する場所                              | Vize が `<Self>` を解決する方法          |
| ----------------------------------------- | ---------------------------------------- |
| コンパイラのメタデータがある SFC のビルド | メタデータの現在のコンポーネント名を使う |
| メタデータがない SFC のビルド             | ファイル名からコンポーネント名を求める   |
| テンプレート API の直接呼び出し           | `componentName` が渡されていれば使う     |
| 名前がない場合                            | 通常のコンポーネントの名前解決を維持する |

フラグが有効な場合、`<Self>` は、`Self` という名前のローカル・インポート済み・グローバルのコンポーネントより先に解決されます。別のコンポーネントを使う場合は、インポート名を `Self` にしないでください。props、attrs、イベント、ディレクティブ、refs、`v-if`、`v-for`、`v-show`、スロットは通常のコンポーネントと同じです。再帰するテンプレートには、再帰を終了する条件が必要です。

```vue
<script setup lang="ts">
import OtherSelf from "./OtherSelf.vue";
</script>

<template>
  <Self />
  <OtherSelf />
</template>
```

タグ名は完全一致で判定し、大文字・小文字を区別します。フラグが有効な場合だけ `<Self>` を予約します。`<self>` は特別扱いしません。render 関数と JSX は、このフラグの対象外です。

SFC のファイル名を持たない低レベルのテンプレート呼び出しでは、名前を明示的に渡してください。

```ts
import { compile } from "@vizejs/native";

compile("<Self :node=\"node\" />", {
  componentName: "TreeNode",
  experimentalSelfComponent: true,
});
```

<span id="strict-slot-children"></span>

## スロットの子要素を型検査

`strictSlotChildren` は RFC #734 のツール向けの処理を実装します。親がコンポーネントのスロットに渡す子ノードに対して、仮想 TypeScript の型検査を追加します。テンプレートのコード生成や実行時の描画は変わりません。

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => [typeof TabItem, HTMLButtonElement];
    footer: () => [HTMLButtonElement];
  };
};
```

SFC の `defineSlots` からも、同じ公開契約を指定できます。

```ts
defineSlots<{
  default(): [typeof TabItem, HTMLButtonElement];
  footer(): [HTMLButtonElement];
}>();
```

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

フラグが有効な場合、Vize は渡されたデフォルトスロットを `__VizeProvidedSlotChildren<[typeof TabItem, HTMLButtonElement]>` で表し、コンポーネントの `__vizeSlots.default` の戻り値の型と比較するよう TypeScript に渡します。型が一致しない子要素は、TypeScript の診断になります。

```vue
<template>
  <Tabs>
    <div />
  </Tabs>
</template>
```

子ノードは次の型に対応します。

| 渡された子ノード                               | 仮想 TypeScript の型                                                                |
| ---------------------------------------------- | ----------------------------------------------------------------------------------- |
| `<button>` などのネイティブ要素                | `HTMLButtonElement`                                                                 |
| SVG または MathML の要素                       | より具体的な DOM の型が分からない場合は、`SVGElement` または `MathMLElement` を使う |
| `<TabItem>` などのインポート済みコンポーネント | `typeof TabItem`                                                                    |
| テキストまたは補間                             | `string`                                                                            |
| 名前付きの `<template #footer>`                | 名前付きスロットと比較する                                                          |
| コメントと、構造だけのラッパー                 | 子の値を持たない場合は検査を省略する                                                |

RFC #734 の戻り値の形は、子要素の個数と順序を表す TypeScript の契約として保持します。

| スロットの戻り値の契約                       | Vize の仮想 TypeScript での意味                      |
| -------------------------------------------- | ---------------------------------------------------- |
| `() => HTMLInputElement`                     | input 要素が一つ。子が一つの場合の省略形を受け付ける |
| `() => HTMLInputElement[]`                   | input 要素がゼロ個以上                               |
| `() => [HTMLInputElement, HTMLInputElement]` | タプルの順序で input 要素が二つ                      |
| `() => (typeof TabItem)[]`                   | `TabItem` コンポーネントがゼロ個以上                 |
| `() => [typeof TabItem, HTMLButtonElement]`  | `TabItem` が一つ、その後に button 要素が一つ         |

`__VizeProvidedSlotChildren<__T>` は、スロットの契約が受け付ける子の型が一つの場合、子を `Only` と `[Only]` のどちらで指定しても受け付けます。複数の子を指定する契約では、タプル・配列の形を保ちます。制約のないスロットのインデックスシグネチャと `any` は `any` に緩和されるため、TypeScript は子要素の厳密な診断を出しません。組み込み・動的コンポーネント、`KeepAlive`、`Teleport`、`Transition`、`TransitionGroup`、`Suspense` は、現在、厳密なコンポーネントの子の型に寄与しません。

スロットの契約が props に依存する高度なコンポーネントライブラリでは、`__vizeResolveSlots` を公開できます。`defineSlots` を使うコンポーネントは、親が参照するための `__vizeSlots` マーカーを公開します。必須のスロット名の検査は別の仕組みです。このフラグは、既存の仮想 TypeScript のスロットモデルに子ノードの型検査を追加します。

<span id="verification-checklist"></span>

## 検証項目

新しい実験的な動作をドキュメントに記載する前に、次の点をテストで確認してください。

- フラグの既定値は無効になっている。
- 共通設定とプラグインに直接渡す設定は、記載した優先順位で解決される。
- コンパイラに直接渡す項目は、解決済みの真偽値になっている。
- 低レベル API の説明では、実在する入口の `compile`、`compileVapor`、`parseTemplate`、`compileSfc*` を名前で示している。
- 不正な構文は黙ってコンパイルされず、診断になる。
- DOM、SSR、Vapor、SFC、WASM、`vize check` の検証範囲は、対応範囲の表と一致している。
- 正しい例、フラグが無効な場合または不正な配置の例、実装の制約を説明している。
