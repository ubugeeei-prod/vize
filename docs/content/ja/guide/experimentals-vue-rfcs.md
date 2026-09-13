---
title: Vue RFC Experimental Details
---

<!-- Generated translation; source: guide/experimentals-vue-rfcs.md -->

# Vue RFC Experimental Details

このページは [Experimentals](./experimentals.md) の Vue RFC 部分を詳しく説明します。ここで
document しているのは Vize が ship している opt-in contract と実装上の制限です。上流 PR は設計
source ですが、対応する Vize flag を明示的に on にしない限り RFC 機能は有効になりません。

## Opt-in Contract

RFC flag はすべて独立しています。1 つの proposal を有効にしても、別の proposal は有効になりません。

| RFC | Vize config flag | direct compiler field | 対象 | flag が off の場合 |
| --- | --- | --- | --- | --- |
| [#823](https://github.com/vuejs/rfcs/pull/823) patterned templates | `experimentals.patternedTemplate` | `experimentalPatternedTemplate` | DOM, SSR, Vapor, SFC, WASM compile API | `v-match`、`v-when`、`v-case` は opt-in が必要だと報告 |
| [#831](https://github.com/vuejs/rfcs/pull/831) in-tag comments | `experimentals.inTagComment` | `experimentalInTagComments` | Parser, DOM, SSR, Vapor, SFC, WASM compile API | opening tag 内の `//` は不正な tag syntax として parse |
| [#833](https://github.com/vuejs/rfcs/pull/833) self references | `experimentals.selfComponent` | `experimentalSelfComponent` | DOM, SSR, Vapor, SFC, WASM compile API | `<Self>` は通常の component tag |
| [#734](https://github.com/vuejs/rfcs/pull/734) strict slot children | `experimentals.strictSlotChildren` | `experimentalStrictSlotChildren` | `vize check`, LSP, virtual-TS project API | 追加の child-type assertion は生成しない |

共有 config と plugin config では `experimentals` を使います。direct compiler field は、alias、
precedence、`{}` switch object をすでに final boolean に解決した integration だけが使います。

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
vize({
  experimentals: {
    patternedTemplate: false,
    inTagComment: true,
  },
});
```

```ts
compileTemplate(source, {
  experimentalPatternedTemplate: true,
  experimentalInTagComments: true,
  experimentalSelfComponent: true,
});
```

2 つ目の例では、direct plugin value が shared config の patterned templates opt-in をその plugin
instance だけ無効化し、in-tag comments は有効のままにします。direct `compileTemplate` field は
`pattenedTemplate` や `intagComment` のような alias を解釈しません。

## Patterned Templates

`patternedTemplate` は RFC #823 の long-form `v-match` / `v-when` syntax を実装します。subject
expression は 1 回だけ評価され、direct branch child は source order で検査され、最初に match した
branch だけが render されます。

```vue
<template v-match="result">
  <ArticleView v-when="{ status: 'success', data: const article }" :article="article" />
  <RetryBanner
    v-when="{ status: 'error', error: const error } if (error.retriable)"
    :error="error"
  />
  <ErrorBanner v-when="{ status: 'error', error: const error }" :error="error" />
  <LoadingSpinner v-when="{ status: 'loading' }" />
  <p v-when="_">Unknown result.</p>
</template>
```

対応している branch pattern:

| Pattern | Example | Runtime check |
| --- | --- | --- |
| Literal | `v-when="'ready'"`, `v-when="404"` | strict equality。`NaN` は `Number.isNaN` |
| Value | `v-when="Status.Ready"` | identifier または member expression と比較 |
| Wildcard | `v-when="_"` | 常に match し、binding は導入しない |
| Const binding | `v-when="const value"` | 常に match し、この branch で `value` を bind |
| Object | `v-when="{ kind: 'ok', value: const data }"` | open structural object match |
| Object rest | `v-when="{ kind: 'error', ...const payload }"` | 残りの own enumerable property を bind |
| Array rest | `v-when="[const first, ...const rest]"` | array であることを要求し、追加 item を collect |
| Or | `v-when="'idle' | 'loading'"` | alternatives を左から右に試す |
| As binding | `v-when="{ kind: 'ok' } as const whole"` | matched value を `whole` として bind |
| Guard | `v-when="{ error: const e } if (e.retriable)"` | pattern 成功後に実行 |

binding は branch-local です。`v-when` を持つ element、attributes/directives、children、guard
expression から見えます。sibling branch からは見えません。

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

`let` と `var` binding は rejected です。現時点で Vize が受け付ける binding declaration は `const`
だけです。

```vue
<template v-match="entry">
  <p v-when="{ data: let article }">{{ article }}</p>
</template>
```

### Patterned Deferred Syntax

`v-case` は古い experiment との互換 alias としてだけ受け付けます。新しい code は `v-when` を使って
ください。RFC で議論されている shorthand candidate は Vize の public syntax ではありません。
`?=`、`|=`、`~=` は branch attribute として parse されません。or-pattern alternative 内の binding
も deferred なので、binding が必要な場合は branch を分けてください。

## In-Tag Comments

`inTagComment` は RFC #831 の opening tag attribute list 内 compile-time-only `//` comment を実装します。
Vize は parser/tooling fidelity のために source text を保持し、runtime code は emit しません。

```vue
<template>
  <LegacySelect
    :options="options"
    // @vue-expect-error legacy API accepts string IDs
    :selected-id="selectedId"
  />
</template>
```

置ける場所は tag name の後、complete attribute/directive の間、最後の attribute の後かつ `>` / `/>`
の前です。この comment は child node ではないので `<!-- ... -->` とは違い、template `comments`
option にも左右されません。

attribute value 内の `//` は通常の text として扱われます。

```vue
<template>
  <a href="https://example.test/a//b">Link</a>
</template>
```

この機能は template を自分で parse する SFC/tooling pipeline 向けです。raw in-DOM template は先に
browser に parse されるため、そこで in-tag comment に依存しないでください。

## Self Component

`selfComponent` は RFC #833 の exact tag `<Self>` を current component 用に予約します。

```vue
<template>
  <li>
    {{ node.id }}
    <ul>
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

tag は exact かつ case-sensitive です。flag が有効な場合だけ `<Self>` が予約されます。`<self>` は特別
ではありません。flag が off の場合、`Self` という local component binding は通常どおり扱われます。

## Strict Slot Children

`strictSlotChildren` は RFC #734 の tooling 側を実装します。parent が component slot に渡す child node
に対して virtual TypeScript assertion を追加します。template codegen や runtime rendering は変えません。

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => [typeof TabItem, HTMLButtonElement];
    footer: () => [HTMLButtonElement];
  };
};
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

flag が有効な場合、Vize は provided default slot を
`__VizeProvidedSlotChildren<[typeof TabItem, HTMLButtonElement]>` と表現し、component の
`__vizeSlots.default` return type と比較するよう TypeScript に渡します。

child mapping:

| Provided child | Virtual TypeScript type |
| --- | --- |
| `<button>` などの native element | `HTMLButtonElement` |
| `<TabItem>` などの imported component | `typeof TabItem` |
| text または interpolation | `string` |
| named `<template #footer>` | named slot と比較 |
| comment と structural-only wrapper | child value を持たない場合は skip |

slot contract が props に依存する advanced component library は `__vizeResolveSlots` を expose できます。
`defineSlots` を使う component は parent のために `__vizeSlots` marker を export します。required slot
name check は別の仕組みです。この flag は既存の virtual-TS slot model に child node type check を追加します。

## Verification Checklist

新しい experimental behavior を document する前に、次の事実を test で固定してください。

- flag の default は off
- shared config と direct plugin option は document された precedence で解決される
- direct compiler field は final boolean
- bad syntax は黙って compile されず diagnostic になる
- DOM、SSR、Vapor、SFC、WASM、または `vize check` coverage が surface matrix と一致する
- docs には good example と flag-off または invalid-placement example の両方がある
