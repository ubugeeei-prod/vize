---
title: Experimentals Reference
---

<!-- Generated translation; source: guide/experimentals-reference.md -->

# Experimentals Reference

このページは [Experimentals](./experimentals.md) と
[Vue RFC Experimental Details](./experimentals-vue-rfcs.md) の dense reference です。どの
entry point が flag を解決するか、変更に必要な proof は何か、どの low-level API field がすでに
final boolean なのかを確認するときに使います。

## Entry Points

その判断を所有している最も高レベルの entry point を使ってください。project config は Vize config
を読むツール向け、direct plugin option は 1 つの Vite plugin instance 向け、native compiler
field は project policy をすでに解決した integration 向けです。

| Entry point | 受け付ける shape | 解決する主体 | Notes |
| --- | --- | --- | --- |
| `vize.config.*` | `experimentals: { ... }` switch value | config loader | npm command、`vize check`、LSP session、project config を読む Vite plugin instance の shared default |
| `vize({ experimentals })` | 同じ switch value | Vite plugin option resolver | direct value が shared config より優先される。`false` と `null` は per-plugin opt-out |
| `vize({ vapor })`, `vize({ jsxMode })`, `compiler.vapor`, `compiler.jsxMode` | stable compiler option | compiler option resolver | これらの stable choice は `experimentals.vapor` と `experimentals.jsxVapor` の fallback routing より優先 |
| `compile`, `compileVapor`, `parseTemplate` | `CompilerOptions` の `experimental*` boolean field | caller | alias なし、`{}` switch object なし、shared-config precedence なし |
| `compileSfc`, `compileSfcBatch`, `compileSfcBatchWithResults` | SFC/batch option の `experimental*` boolean field | caller | config resolution 後の native SFC / WASM integration で使う |
| `vize check` と LSP/type-check project API | project `experimentals` と解決済み type-checker flag | config loader と project session | `strictSlotChildren` はここで virtual TypeScript check を生成する。runtime codegen ではない |

## Flag Contracts

| Flag | 有効化する場面 | 最小の proof | flag の外側に残るもの |
| --- | --- | --- | --- |
| `patternedTemplate` | project が RFC [#823](https://github.com/vuejs/rfcs/pull/823) の `v-match` / `v-when` branch を authored template で使うと決めたとき | direct `v-match` child を持つ flagged component と、flag-off で `experimentals.patternedTemplate` を報告する component を compile | type-checker exhaustiveness、unreachable-branch diagnostic、or-pattern binding |
| `inTagComment` | `@vue-expect-error` などの line-local annotation を opening tag の対象 prop 近くに置きたい tooling | tagged component を parse し、comment が `root.comments` に保持されつつ output code が変わらないことを assert | browser in-DOM template、runtime comment、通常の `comments` compiler option |
| `selfComponent` | recursive SFC が RFC [#833](https://github.com/vuejs/rfcs/pull/833) self-reference を `name` option や filename inference だけに依存せず持ちたいとき | `componentName` または SFC metadata 付きで `<Self />` を compile し、local import named `Self` が target ではないことを確認 | JSX、render function、lowercase `<self>` |
| `strictSlotChildren` | library / application が RFC [#734](https://github.com/vuejs/rfcs/pull/734) typed slot-child contract を公開し、`vize check` や LSP diagnostic が欲しいとき | valid な default/named slot tuple と TypeScript が reject する invalid child を type-check | runtime rendering、open `any` slot contract、built-in/dynamic component child typing |
| `serverScript` | host integration が server-script compiler experiment を所有して test しているとき | switch が有効な場合だけ native `experimentalServerScript` boolean が forward されることを assert | host が document するまで public server-script syntax semantics |
| `vapor` | stable `compiler.vapor` へ昇格する前に SFC Vapor を試すとき | `compiler.vapor` と direct `vize({ vapor })` が未設定の場合だけ `experimentals.vapor: true` が Vapor を選ぶことを確認 | stable project-wide Vapor policy |
| `jsxVapor` | stable `compiler.jsxMode` へ昇格する前に JSX/TSX Vapor を試すとき | `jsxMode` が未設定の場合だけ `experimentals.jsxVapor: true` が JSX output を Vapor default にすることを確認 | per-file `"use vue:*"` directive と stable JSX backend policy |

RFC [#831](https://github.com/vuejs/rfcs/pull/831) も他の RFC flag と同じ opt-in shape です。ただし挙動は parser/tooling-only で、attribute 近くの `//` annotation を保持し、runtime comment は emit しません。

## Direct API Fields

ほとんどの application は `experimentals` を設定します。config resolution を自前で済ませる低レベル
integration だけが native compiler field を直接渡してください。

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

これらの field は解決済み boolean です。alias、`{}` switch object、shared-config precedence
は解釈しません。caller がその解決ルールを所有する integration boundary でだけ使います。

## Config Recipes

意図した behavior を証明する最小の flag set だけを有効にします。無関係な RFC switch を 1 つの
project toggle にまとめないでください。また、feature が stable な `compiler` option または tool
option に移るまでは、experimental flag を shared config で default-on に昇格しません。

shared config で 1 つの RFC proposal だけを有効にする例:

```ts
import { defineConfig } from "vize";

export default defineConfig({
  experimentals: {
    inTagComment: true,
  },
});
```

shared config は有効のまま、一時的に plugin instance だけ opt-out する例:

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

config resolution 後の低レベル integration:

```ts
compile(templateSource, {
  experimentalInTagComments: resolvedExperimentals.inTagComments,
  experimentalPatternedTemplate: resolvedExperimentals.patternedTemplate,
});
```

native API call には `intagComment` のような compatibility alias や `{}` のような switch object を
渡しません。compiler を呼ぶ前に解決してください。

## Failure Examples

`patternedTemplate` は構造が違うと fail-closed です。

```vue
<template v-match="status"></template>

<template v-match="status">
  <p v-when:ready="'ready'">Ready</p>
  <p v-when.once="'ready'">Ready again</p>
</template>
```

`inTagComment` は attribute 用の第 2 の expression grammar ではありません。

```vue
<template>
  <LegacySelect :label="'// this is an attribute value, not an in-tag comment'" />

  <LegacySelect
    :// not a directive argument comment
    selected-id="abc"
  />
</template>
```

## Slot Cardinality

RFC #734 の return shape は TypeScript cardinality contract として保持します。

| Slot return contract | Vize virtual TS での意味 |
| --- | --- |
| `() => HTMLInputElement` | input child 1 つ。single-child shorthand を受け付ける |
| `() => HTMLInputElement[]` | input child 0 個以上 |
| `() => [HTMLInputElement, HTMLInputElement]` | tuple order どおり input child 2 つ |
| `() => (typeof TabItem)[]` | `TabItem` component child 0 個以上 |
| `() => [typeof TabItem, HTMLButtonElement]` | `TabItem` child 1 つ、その後に button child 1 つ |

## Implementation Coverage

| Concern | docs を変える前に必要な proof |
| --- | --- |
| Default-off behavior | flag-off の parse、compile、check、LSP path が documented off behavior を報告または保持する |
| Config resolution | shared config、direct Vite plugin value、alias、`false`、`null`、`{}` を native boolean と分けて cover する |
| Native template API | `compile`、`compileVapor`、`parseTemplate` が document された `experimental*` boolean field だけを受け付ける |
| Native SFC API | `compileSfc`、`compileSfcBatch`、`compileSfcBatchWithResults` が per-file / batch で同じ boolean を forward する |
| Type-checking API | `strictSlotChildren` は runtime output snapshot ではなく virtual TypeScript diagnostic で証明する |
| Boundary test | deferred RFC behavior は negative assertion または documented absence を持ち、近い syntax から support を推測させない |

## Release Safety Checklist

release note で experimental surface を claim する前に、shipped entry point に対して以下を確認します。

- public config key と direct native field の default-off behavior が検証されている
- alias は recommended name ではなく compatibility input として記載されている
- direct Vite plugin value が shared config を有効化でき、かつ明示的に opt-out できる
- `false`、`null`、`true`、`{}` が documented switch semantics を維持する
- `vapor` や `jsxVapor` のような backend fallback flag は stable `compiler` option より弱い
- RFC example には enabled example と flag-off または invalid-shape diagnostic example の両方がある
- boundary が patterned-template exhaustiveness や dynamic component に対する strict slot support などの deferred behavior を明記している
