---
title: Experimentals
---

<!-- Generated translation; source: guide/experimentals.md -->

# Experimentals

`experimentals` は、提案段階の Vue 構文、ツール専用の型検査、まだ安定化していない backend route を明示的に opt-in するための共有設定です。安定した `compiler` オプションや Vue runtime の `features` とは分けています。すべてのキーはデフォルトで無効で、RFC 系の 機能は必ず個別に有効化し、同じ backend choice を安定オプションでも表せる場合は安定オプションを優先します。

このページは、次の機能を有効化するときの参照先です。

- Vue RFC [#823](https://github.com/vuejs/rfcs/pull/823) patterned templates
- Vue RFC [#831](https://github.com/vuejs/rfcs/pull/831) in-tag comments
- Vue RFC [#833](https://github.com/vuejs/rfcs/pull/833) `<Self>` component references
- Vue RFC [#734](https://github.com/vuejs/rfcs/pull/734) strict slot child checks
- server-script、SFC Vapor、JSX Vapor routing などの Vize backend experiments

下の表は、すべての public switch の off 時の挙動と precedence まで含めた契約です。RFC ごとの diagnostic、direct API entry point、implementation boundary、deferred syntax は [Vue RFC Experimental Details](./experimentals-vue-rfcs.md) に詳しくまとめています。

## 推奨設定

新しい TypeScript config では推奨 camelCase 名を使ってください。

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

JSON と Pkl config でも同じ値を使えます。`{}` は、機能を有効にしつつ将来の nested option のために object shape を残したい場合だけ使います。

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

## スイッチ値

各キーは switch として扱われます。

| 値 | 意味 |
| --- | --- |
| 省略 | 無効 |
| `false` | 無効。共有設定の opt-in を上書きするときも無効 |
| `null` | 無効。nullable な JSON 設定を生成する場合に便利 |
| `true` | 有効 |
| `{}` | 有効。将来の per-feature option 用に object shape を予約 |

Missing keys, `false`, and `null` are off. `true` と `{}` は有効です。

型付き TypeScript と Pkl config では `true`、`false`、`null`、`{}` だけを使ってください。 config loader は互換性のため、存在する non-`false` / non-`null` JSON 値を有効として扱いますが、 それは公開ドキュメント上の推奨 shape ではありません。

## 優先順位

共有 `vize.config.*` の値は、project config を読むツールのデフォルトです。直接の Vite plugin option は共有設定より優先され、明示的な opt-out も上書きとして扱われます。

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

この例では、`patternedTemplate: false` が共有 config の opt-in をこの plugin instance だけで無効化し、 `inTagComment: true` は同じ instance で parser feature を有効にします。

安定した compiler option がある場合は、そちらが優先されます。

- `vize({ vapor })` が `compiler.vapor` より優先され、その次に `experimentals.vapor` を見ます。
- `vize({ jsxMode })` が `compiler.jsxMode` より優先され、その次に `experimentals.jsxVapor` を見ます。
- `experimentalPatternedTemplate` のような resolved field は低レベル compiler switch です。
config resolution をすでに所有している integration だけが直接使ってください。

## Surface Matrix

| Flag | 有効時の挙動 | off 時の挙動 | enforce される場所 | boundary |
| --- | --- | --- | --- | --- |
| `patternedTemplate` | `v-match` / direct `v-when` branch を parse して lower | `v-match`、`v-when`、`v-case` は必要な opt-in を報告 | DOM, SSR, Vapor, SFC, WASM compile API | `vize check` は exhaustiveness をまだ certify しない |
| `inTagComment` | opening tag の `//` comment を parse し tooling 用に保持 | `//` は不正な tag syntax | Parser, DOM, SSR, Vapor, SFC, WASM compile API | runtime output なし、browser in-DOM support なし |
| `selfComponent` | exact `<Self>` を current component として扱う | `<Self>` は通常の component tag | DOM, SSR, Vapor, SFC, WASM compile API | render function と JSX は対象外 |
| `strictSlotChildren` | virtual TypeScript child assertion を出す | child-type assertion は生成されない | `vize check`, LSP/type-check project API | open slot と `any` は TypeScript の permissive behavior のまま |
| `serverScript` | native compiler integration に `experimentalServerScript` を渡す | native flag は `false` のまま | それを expose する native compiler integration | runtime が semantics を document するまでは host-defined |
| `vapor` | stable Vapor が未設定なら SFC Vapor に fallback | SFC は stable/default backend のまま | Vite plugin, package build config | `vize({ vapor })` と `compiler.vapor` が優先 |
| `jsxVapor` | stable JSX mode が未設定なら JSX/TSX Vapor に fallback | JSX/TSX は stable/default backend のまま | Vite plugin, package build config | `vize({ jsxMode })` と `compiler.jsxMode` が優先 |

entry-point coverage、最小の proof、low-level native API example は
[Experimentals Reference](./experimentals-reference.md) を参照してください。

## Flag Reference

| Flag | upstream / source | resolved field | compatibility names |
| --- | --- | --- | --- |
| `patternedTemplate` | Vue RFC [#823](https://github.com/vuejs/rfcs/pull/823) | `experimentalPatternedTemplate` | `pattenedTemplate` |
| `inTagComment` | Vue RFC [#831](https://github.com/vuejs/rfcs/pull/831) | `experimentalInTagComments` | `intagComment` |
| `selfComponent` | Vue RFC [#833](https://github.com/vuejs/rfcs/pull/833) | `experimentalSelfComponent` | `self_component` in JSON |
| `strictSlotChildren` | Vue RFC [#734](https://github.com/vuejs/rfcs/pull/734) | `experimentalStrictSlotChildren` | `strict_slot_children` in JSON |
| `serverScript` | Server script compiler experiment | `experimentalServerScript` | `"server script"`, `server_script` in JSON |
| `vapor` | SFC Vapor backend routing | `compiler.vapor` fallback | none |
| `jsxVapor` | JSX/TSX Vapor default | `compiler.jsxMode` fallback | none |

推奨 flag と alias を同時に設定しないでください。互換 alias は古い config を読み続けるためにありますが、混在すると compatibility resolution order に依存した挙動になります。新しい config では `patternedTemplate`、`inTagComment`、`selfComponent`、`strictSlotChildren`、`serverScript`、`vapor`、`jsxVapor` を使います。

## Patterned Templates

`patternedTemplate` は、Vue RFC #823 の `v-match` container と direct `v-when` branch child を有効化します。

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

match 対象の式は 1 回だけ評価されます。branch は上から順に検査され、最初に match した branch だけが render され、fallthrough はありません。`v-when="_"` は fallback pattern です。unguarded top-level fallback の場合は一意で、最後の branch である必要があります。

対応している pattern:

| Pattern | Example | Notes |
| --- | --- | --- |
| Literal | `v-when="'ready'"`, `v-when="404"` | strict equality。`NaN` は `Number.isNaN` |
| Value | `v-when="Status.Active"` | identifier と member expression を runtime value と比較 |
| Wildcard | `v-when="_"` | binding を導入せずに match |
| Const binding | `v-when="const value"` | `let` / `var` は rejected |
| Object | `v-when="{ kind: 'ok', value: const result }"` | 余分な object property は許容 |
| Object shorthand | `v-when="{ kind: 'ok', const data }"` | `{ const data }` は `{ data: const data }` |
| Object rest | `v-when="{ kind: 'error', ...const payload }"` | rest binding も `const` 必須。lone `...` も accepted |
| Array / tuple | `v-when="[const first, ...const rest]"` | `Array.isArray` が必要。rest があれば追加要素を許容 |
| Or | `v-when="'idle' | 'loading'"` | alternatives は左から右。alternative 内 binding は未対応 |
| As binding | `v-when="{ kind: 'ok', const data } as const entry"` | nested binding に加えて matched value も bind |
| Guard | `v-when="{ kind: 'error', error: const e } if (e.retriable)"` | pattern match 後に guard を実行 |

`v-case` は古い Vize 実験の互換 alias として残っていますが、新しい template では `v-when` を使います。 RFC #823 で議論されている shorthand candidate は Vize の public syntax ではありません。`?=`、`|=`、 `~=` は有効化されません。

不正な placement は黙って compile せず diagnostic になります。`v-when` branch は `v-match` container の direct child である必要があり、`v-match` には少なくとも 1 つ direct branch が必要です。`v-when` は directive argument や modifier を受け付けません。flag が無効な場合、Vize は `experimentals.patternedTemplate` が必要だと報告します。exhaustiveness と branch narrowing は tooling boundary なので、依存する前に RFC detail page を確認してください。

## In-Tag Comments

`inTagComment` は、Vue RFC #831 の opening tag attribute list 内 `//` comment を parse します。

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

comment は tag name の後、complete attribute/directive の間、最後の attribute の後かつ `>` / `/>` の前に置けます。trailing `//` の後は closing delimiter を次の行に置いてください。同じ行の `>` / `/>` は comment text として消費されます。Vize は source text を tooling 用に template root の `comments` list へ保持しますが、child node にはせず runtime output も生成しません。attribute value 内の `//` は通常の文字列です。

```vue
<template>
  <a href="https://example.test/path//segment">Link</a>
</template>
```

通常の template `comments` compiler option は、この構文の on/off には影響しません。flag が off の場合、 parser は opening tag 内の unexpected solidus として報告します。この構文は template parsing を所有する SFC/tooling pipeline 向けです。先に browser HTML parser を通す raw in-DOM template では前提にしないでください。

## Self Component

`selfComponent` は、Vue RFC #833 の exact `<Self>` tag を recursive component reference として予約します。

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

SFC build では、Vize は compiler metadata または filename から current component name を取り、 `<Self>` をその component に解決します。SFC filename がない direct template API では `componentName` を渡せます。`selfComponent` が無効なら `<Self>` は通常の component tag のままで、`Self` という local component binding も通常どおり扱われます。flag が on の場合、exact `<Self>` は local、imported、global component named `Self` を shadow します。

flag は DOM、SSR、Vapor compilation に通されます。有効かつ component name がある場合、emitted component resolution は current component name と Vue の maybe-self-reference hint を使います。reserved tag は exact かつ case-sensitive です。`<Self>` は特別ですが、`<self>` は特別ではありません。

## Strict Slot Children

`strictSlotChildren` は Vue RFC #734 の slot children 向け virtual-TypeScript checks を有効化します。 対象は `vize check`、LSP/type-check project API、declaration-aware virtual code です。通常の template compile に runtime behavior は追加されません。

```ts
export default defineConfig({
  experimentals: {
    strictSlotChildren: true,
  },
});
```

component は `__vizeSlots` で structural slot contract を公開できます。

```ts
import TabItem from "./TabItem.vue";

declare const Tabs: {
  readonly __vizeSlots?: {
    default: () => (typeof TabItem)[];
    footer: () => [HTMLButtonElement];
  };
};
```

Vize は提供された children をその contract と比較します。

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

flag が有効な場合、Vize は slot child assertion を合成し、TypeScript が provided node と slot return contract を比較できるようにします。native DOM child は対応する `HTML*Element`、component child は `typeof ComponentRef`、text/interpolation child は `string` に map されます。named slot と default slot はどちらも検査対象です。`defineSlots` contract も同じ marker shape を export します。値を持たない comment や structural wrapper node は skip されます。

これは tooling constraint なので、失敗は `vize check` または language server の TypeScript diagnostic として出ます。Vue template を compile するだけの build では enforce されません。open index signature や `any` を使う slot contract は TypeScript の permissive behavior に degrade します。

## Server Script

`serverScript` は server-script compiler experiment を明示的に検証する host だけで有効にしてください。

```ts
export default defineConfig({
  experimentals: {
    serverScript: true,
  },
});
```

この flag は native `experimentalServerScript` compiler field に解決されます。省略、`false`、`null` の場合、native field は disabled のままです。一部 runtime では、compiler stage が実験を support するまで switch を reserved のままにします。application code は安定した syntax contract として依存しないでください。framework がこの switch を消費し始めた場合は、その host 固有の documentation を優先してください。

## Vapor

`vapor` は、stable `compiler.vapor` と direct `vize({ vapor })` が未設定の場合に、SFC compilation を experimental Vapor backend に route します。

```ts
export default defineConfig({
  experimentals: {
    vapor: true,
  },
});
```

project が Vapor を安定した build choice として採用した場合は、`compiler.vapor` または direct `vize({ vapor })` を使ってください。stable/direct value は `experimentals.vapor` が `true` でも優先されるため、明示的な `vize({ vapor: false })` は per-plugin opt-out です。`experimentals.vapor` は trial run、compatibility probe、experimental fallback path のテスト用に残します。

## JSX Vapor

`jsxVapor` は、stable `compiler.jsxMode` と direct `vize({ jsxMode })` が未設定の場合に、JSX/TSX file の default output を Vapor にします。

```ts
export default defineConfig({
  experimentals: {
    jsxVapor: true,
  },
});
```

project-wide の backend choice として安定運用するなら `compiler.jsxMode: "vapor"` または direct `vize({ jsxMode: "vapor" })` を使ってください。stable/direct value は `experimentals.jsxVapor` より優先され、`jsxMode: "vdom"` は明示的な opt-out です。component-level の `"use vue:vapor"` と `"use vue:vdom"` directive は、その source file 自体の意図として扱われ、default mode を上書きできます。

## Direct API Fields

ほとんどの application は `experimentals` を設定します。低レベル integration は、解決済み boolean
field を `compile`、`compileVapor`、`parseTemplate`、`compileSfc`、`compileSfcBatch`、
`compileSfcBatchWithResults` に渡せます。これらの field は alias、`{}` switch object、
shared-config precedence を解釈しません。詳しくは
[Experimentals Reference](./experimentals-reference.md#direct-api-fields) を参照してください。
