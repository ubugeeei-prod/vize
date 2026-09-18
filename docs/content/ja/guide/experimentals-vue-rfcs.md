---
title: Vue RFC Experimental Details
---

<!-- Generated translation; source: guide/experimentals-vue-rfcs.md -->

# Vue RFC Experimental Details

このページは [Experimentals](./experimentals.md) の Vue RFC 部分を詳しく説明します。Vize が ship している opt-in contract、各 flag で有効になる example、まだ RFC または tooling work として扱う boundary をまとめています。上流 PR は design source ですが、対応する Vize flag を明示的に on にしない限り RFC 機能は有効になりません。

## Opt-in Contract

RFC flag はすべて独立しています。1 つの proposal を有効にしても、別の proposal は有効になりません。

| RFC | Vize config flag | Direct compiler field | 対象 | flag が off の場合 |
| --- | --- | --- | --- | --- |
| [#823](https://github.com/vuejs/rfcs/pull/823) patterned templates | `experimentals.patternedTemplate` | `experimentalPatternedTemplate` | DOM, SSR, Vapor, SFC, WASM compile API | `v-match`、`v-when`、`v-case` は opt-in が必要だと報告 |
| [#831](https://github.com/vuejs/rfcs/pull/831) in-tag comments | `experimentals.inTagComment` | `experimentalInTagComments` | Parser, DOM, SSR, Vapor, SFC, WASM compile API | opening tag 内の `//` は不正な tag syntax として parse |
| [#833](https://github.com/vuejs/rfcs/pull/833) self references | `experimentals.selfComponent` | `experimentalSelfComponent` | DOM, SSR, Vapor, SFC, WASM compile API | `<Self>` は通常の component tag |
| [#734](https://github.com/vuejs/rfcs/pull/734) strict slot children | `experimentals.strictSlotChildren` | `experimentalStrictSlotChildren` | `vize check`, LSP, virtual-TS project API | 追加の child-type assertion は生成しない |

共有 config と plugin config では `experimentals` を使います。direct compiler field は、alias、precedence、`{}` switch object をすでに final boolean に解決した integration だけが使います。

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

`compile`、`compileVapor`、`parseTemplate`、`compileSfc*` の direct native field は
`pattenedTemplate` や `intagComment` のような alias を解釈せず、`{}` switch object も受け付けません。direct Vite plugin value は shared config より優先され、明示的な opt-out も優先されます。

## Current Scope

| RFC | Vize が現在 ship しているもの | 明示しておく boundary |
| --- | --- | --- |
| #823 | parser support、runtime lowering、branch-local binding、guard、rest/as pattern、flag-off と invalid placement の diagnostic | Canon は narrowing・網羅性を検査。editor navigation 全体と compiler grammar の統一は deferred |
| #831 | in-tag `//` parser support、`root.comments` の source text、AST comment を保持する compile pipeline | runtime output なし、child comment node なし、browser in-DOM template support なし |
| #833 | DOM、SSR、Vapor compilation の exact `<Self>` current-component resolution | render-function / JSX macro なし。flag 有効時は local/imported component named `Self` が shadow される |
| #734 | default slot と named slot の provided children に対する virtual TypeScript assertion | template codegen change なし。open slot contract と `any` は TypeScript 側の permissive check に degrade |

entry-point と proof checklist は [Experimentals Reference](./experimentals-reference.md) にあります。

## Patterned Templates

`patternedTemplate` は RFC #823 の long-form `v-match` / `v-when` syntax を実装します。subject expression は 1 回だけ評価され、direct branch child は source order で検査され、最初に match した branch だけが render されます。

inline HTML の SFC では最外周の `<template>` にも `v-match` を指定できます。
直下の `v-when` は DOM・SSR・Vapor で内側の match と同じように動作し、ヘッダーの式だけの変更も template HMR を無効化します。
parse 結果の `template.content` は引き続きブロック本文のみです。外部 `src`、プリプロセッサ言語、元の source metadata が失われた descriptor はこの形式では拒否します。
Canon は後述の opt-in で分岐内 narrowing・網羅性検査に対応します。結合したコンパイラ SFC の source map は既存の script-only の制限が残ります。

Croquis は `analyzeSfc(source, { experimentalPatternedTemplate: true })` と Playground の
チェックボックスで root・nested match を解析できます。RFC の parser で分岐内の宣言、
外側を参照する value pattern、guard、構文診断を記録し、HTML entity を含む元の位置を維持します。
DOM・Vapor・SSR もこの parser に統一し、`pattern as name` を使います。旧 `as const name` は拒否します。
Canon も同じ parser を使い、`typeCheck(source, { experimentalPatternedTemplate: true, includeVirtualTs: true })`、Playground Canon のチェックボックス、`experimentals.patternedTemplate` を有効にした `vize check` で型検査できます。Native Maestro も同じ workspace 設定を読み、構文・未網羅エラーと到達不能 branch の警告を報告し、未保存の編集を再検査します。Root pattern binding の hover は絞り込んだ型を、definition は元の宣言位置を返します。workspace flag を変更したら language server を再起動してください。Content Mapper への設定伝達、guard・nested closure・or-pattern binding を含む rename/completion の全組合せの検証は未完了です。
残作業は [#6176](https://github.com/ubugeeei-prod/vize/issues/6176) で追跡します。

pattern の property は各 arm の試行内で一度だけ読み、guard と描画側は rest copy を含めて同じ値を使います。rest copy は shape 全体の一致後に作り、shape が不一致の arm や後続の未試行 arm の guard は評価しません。`v-when` は `v-if`、`v-else-if`、`v-else`、`v-for`、`v-match` と同じ要素には指定できません。

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

binding は branch-local です。`v-when` を持つ element、attribute/directive、guard expression、children から見えます。sibling branch からは読めません。

対応している branch pattern:

| Pattern | Example | Runtime check |
| --- | --- | --- |
| Literal | `v-when="'ready'"`, `v-when="404"` | strict equality |
| Value | `v-when="Status.Ready"` | identifier または member expression と `NaN` を含む SameValueZero で比較 |
| Wildcard | `v-when="_"` | 常に match し、binding は導入しない |
| Const binding | `v-when="const value"` | 常に match し、この branch で `value` を bind |
| Object | `v-when="{ kind: 'ok', value: const data }"` | open structural object match。余分な property は許容 |
| Object shorthand | `v-when="{ kind: 'ok', const data }"` | `{ const data }` は `{ data: const data }` |
| Object rest | `v-when="{ kind: 'error', ...const payload }"` | 残りの own enumerable property を bind。lone `...` も accepted |
| Array / tuple | `v-when="[const first, ...const rest]"` | `Array.isArray` を要求。rest は追加 item を許容 |
| Or | `v-when="'idle' | 'loading'"` | alternatives を左から右に試す |
| As binding | `v-when="{ kind: 'ok' } as whole"` | matched value を `whole` として bind |
| Guard | `v-when="{ error: const e } if (e.retriable)"` | pattern 成功後に実行 |

### Patterned Diagnostics

flag がない場合、Vize は unknown directive として通さず、必要な opt-in を報告します。

```vue
<template v-match="status">
  <p v-when="'ready'">Ready</p>
</template>
```

`v-when` は `v-match` container の direct child でなければなりません。

```vue
<template v-match="status">
  <section>
    <p v-when="'ready'">Ready</p>
  </section>
</template>
```

unguarded top-level fallback は一意で、最後でなければなりません。

```vue
<template v-match="status">
  <p v-when="_">Other</p>
  <p v-when="'ready'">Ready</p>
  <p v-when="(_)">Again</p>
</template>
```

`v-match` には少なくとも 1 つ direct branch が必要です。`v-when` は directive argument や modifier を使えません。

```vue
<template v-match="status"></template>

<template v-match="status">
  <p v-when:ready="'ready'">Ready</p>
  <p v-when.once="'ready'">Ready again</p>
</template>
```

`let` と `var` binding は rejected です。現時点で Vize が受け付ける binding declaration は `const` だけです。`v-when` は directive argument や modifier も拒否します。`v-case` と `v-case.default` は古い Vize 実験の互換 alias としてだけ残しており、新しい template では `v-when` と `_` を使います。

### Patterned Type Boundary

Canon は arm の binding と元の subject を narrowing し、網羅漏れを error、到達不能 arm を warning として報告します。型アルゴリズムは通常の検証済み `pattern_matching.d.ts` として TypeScript program ごとに共有し、各 template に複製しません。

guard 付き arm は網羅性の証明に使いません。optional property の存在検査は値の `undefined` と key が存在しない余地を維持します。有限の object・tuple union、readonly array、rest binding に対応します。open primitive、`any`、`unknown`、union 型の value pattern は保守的に扱い、網羅性を証明できない場合は guard なしの `v-when="_"` を使います。

網羅漏れは元の subject の位置に出て `vize check` を失敗させます。到達不能 arm の warning だけなら成功します。WASM 単体は構造診断と virtual TypeScript を提供し、完全な型診断は Playground の Monaco TypeScript worker または native checker が行います。他の virtual-TS host は `virtualTsHelpers` を ambient declaration として一度だけ登録してください。

RFC 全体の upstream type-tooling acceptance criteria に含まれる editor navigation・completion までは未認定です。or-pattern alternative 内の binding も deferred なので、binding が必要な場合は branch を分けます。`?=`、`|=`、`~=` は public branch syntax ではありません。

## In-Tag Comments

`inTagComment` は RFC #831 の opening tag attribute list 内 compile-time-only `//` comment を実装します。Vize は parser/tooling fidelity のために source text を保持し、runtime code は emit しません。

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

置ける場所は tag name の後、complete attribute/directive の間、最後の attribute の後かつ `>` / `/>` の前です。trailing comment の後では closing delimiter を次の行に置いてください。`>` や `/>` を同じ行に置くと `//` が closing delimiter まで comment として消費します。この syntax は tag name、attribute name、directive name、argument、modifier、attribute value の中では使えません。

comment は template root の `comments` list に `CommentKind::InTag` として保存されます。element prop ではなく、child `<!-- ... -->` node でもなく、通常の template `comments` option にも左右されません。`comments: false` でも tooling 用の in-tag comment は保持されます。attribute value 内の `//` は通常の text のままです。

```vue
<template>
  <a href="https://example.test/a//b">Link</a>
</template>
```

この syntax は template を自分で parse する SFC/tooling pipeline 向けです。raw in-DOM template は先に browser に parse されるため、そこで in-tag comment に依存しないでください。

## Self Component

`selfComponent` は RFC #833 の exact tag `<Self>` を current component 用に予約します。

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

resolution rule:

| Context | Vize が `<Self>` を解決する方法 |
| --- | --- |
| compiler metadata がある SFC build | metadata の current component name を使う |
| metadata がない SFC build | filename から component name を derive |
| direct template API | `componentName` が渡されていれば使う |
| name がない | 通常の component resolution semantics を維持 |

flag が有効な場合、`<Self>` は local、imported、global component named `Self` より先に解決されます。別 component を使いたい場合は import 名を `Self` にしないでください。props、attrs、events、directives、refs、`v-if`、`v-for`、`v-show`、slot は通常の component usage と同じです。recursive template には termination condition が必要です。

```vue
<script setup lang="ts">
import OtherSelf from "./OtherSelf.vue";
</script>

<template>
  <Self />
  <OtherSelf />
</template>
```

tag は exact かつ case-sensitive です。flag が有効な場合だけ `<Self>` が予約されます。`<self>` は特別ではありません。render function と JSX はこの flag の対象外です。

SFC filename がない low-level template call では、明示的に name を渡してください。

```ts
import { compile } from "@vizejs/native";

compile("<Self :node=\"node\" />", {
  componentName: "TreeNode",
  experimentalSelfComponent: true,
});
```

## Strict Slot Children

`strictSlotChildren` は RFC #734 の tooling 側を実装します。parent が component slot に渡す child node に対して virtual TypeScript assertion を追加します。template codegen や runtime rendering は変えません。

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => [typeof TabItem, HTMLButtonElement];
    footer: () => [HTMLButtonElement];
  };
};
```

同じ public contract は SFC の `defineSlots` からも得られます。

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

flag が有効な場合、Vize は provided default slot を `__VizeProvidedSlotChildren<[typeof TabItem, HTMLButtonElement]>` と表現し、component の `__vizeSlots.default` return type と比較するよう TypeScript に渡します。mismatch した child は TypeScript diagnostic になります。

```vue
<template>
  <Tabs>
    <div />
  </Tabs>
</template>
```

child mapping:

| Provided child | Virtual TypeScript type |
| --- | --- |
| `<button>` などの native element | `HTMLButtonElement` |
| SVG または MathML element | 狭い DOM type が分からない場合は `SVGElement` または `MathMLElement` fallback |
| `<TabItem>` などの imported component | `typeof TabItem` |
| text または interpolation | `string` |
| named `<template #footer>` | named slot と比較 |
| comment と structural-only wrapper | child value を持たない場合は skip |

RFC #734 の return shape は TypeScript cardinality contract として保持します。

| Slot return contract | Vize virtual TS での意味 |
| --- | --- |
| `() => HTMLInputElement` | input child 1 つ。single-child shorthand を受け付ける |
| `() => HTMLInputElement[]` | input child 0 個以上 |
| `() => [HTMLInputElement, HTMLInputElement]` | tuple order どおり input child 2 つ |
| `() => (typeof TabItem)[]` | `TabItem` component child 0 個以上 |
| `() => [typeof TabItem, HTMLButtonElement]` | `TabItem` child 1 つ、その後に button child 1 つ |

`__VizeProvidedSlotChildren<__T>` は、slot contract の accepted child type が 1 つの場合、single child を `Only` または `[Only]` のどちらとしても受け付けます。multi-child contract では tuple/array shape を保ちます。open slot index signature と `any` は `any` に degrade するため、TypeScript は strict child diagnostic を出しません。built-in、dynamic component、`KeepAlive`、`Teleport`、`Transition`、`TransitionGroup`、`Suspense` は現時点では strict component child type に寄与しません。

slot contract が props に依存する advanced component library は `__vizeResolveSlots` を expose できます。`defineSlots` を使う component は parent のために `__vizeSlots` marker を export します。required slot-name check は別の仕組みです。この flag は既存の virtual-TS slot model に child node type check を追加します。

## Verification Checklist

新しい experimental behavior を document する前に、次の事実を test で固定してください。

- flag の default は off
- shared config と direct plugin option は document された precedence で解決される
- direct compiler field は final boolean
- low-level docs は実在 entry point の `compile`、`compileVapor`、`parseTemplate`、`compileSfc*` を名前で示す
- bad syntax は黙って compile されず diagnostic になる
- DOM、SSR、Vapor、SFC、WASM、または `vize check` coverage が surface matrix と一致する
- docs には good example、flag-off または invalid-placement example、implementation boundary がある
