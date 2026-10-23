---
title: TypeScript Content Mapper
---

# TypeScript Content Mapper

This is the recommended way to type-check Vue files. Use TypeScript 7.1 or newer.
[tsrs](https://github.com/oxc-project/tsrs) can run the same `tsconfig`. For editor setup,
see the [VS Code integration](../integrations/vscode.md). Day-to-day lint and format stay in
[Static Analysis](./static-analysis.md).

Content Mappers are TypeScript's plugin surface for checking file types the compiler cannot parse
itself — the [TypeScript 7.1 API roadmap](https://github.com/microsoft/typescript-go/issues/4830)
identifies them as the TS Server plugin replacement needed by Vue. The initial API merged in
[microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712).
That staging repository is now archived; native TypeScript development continues in
[microsoft/TypeScript](https://github.com/microsoft/TypeScript).
The [Content Mapper conformance workflow](https://github.com/ubugeeei-prod/vize/blob/main/.github/workflows/content-mapper-conformance.yml)
shows the exact checkout and build steps for the host described below.

Vize ships a conforming content mapper inside the `vize` npm package: a `tsgo` build with
content-mapper support spawns `vize content-mapper` and checks `.vue` files directly — hover,
go-to-definition, rename, completions, and diagnostics all map back to your authored SFC, with no
parallel `.vue.ts` project to materialize.

> [!NOTE]
> Content mappers run external programs. TypeScript 7.1 and later accept them with
> `tsc --runExternalCode`. [tsrs](https://github.com/oxc-project/tsrs) accepts the same
> flag and the same `contentMappers` entry. The conformance revision below qualifies one
> host build. It is not a requirement to install a private compiler before you type-check.

## Setup

Install `vize` and declare the mapper in your `tsconfig.json`:

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

Running external mapper processes requires explicit opt-in. TypeScript 7.1 and later:

```bash
tsc --runExternalCode --noEmit -p tsconfig.json
```

tsrs uses the same project and the same flag:

```bash
vp install -D tsrs
tsrs --runExternalCode --noEmit -p tsconfig.json
```

Vize's conformance workflow builds the native compiler entrypoint, `tsc/cmd/tsc`, at the
revision specified below and invokes that binary as `tsgo`. That run qualifies one host
revision. Day-to-day checks use the TypeScript 7.1 or tsrs command above.

In VS Code, when a trusted workspace uses a content-mapper-enabled TypeScript host, the Vize
extension registers `.vue` support automatically. The same mapper then powers the editor.
Turn on `js/ts.contentMappers.enabled` when the editor asks for it.

## Options

A mapper entry accepts an `options` object:

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

| Option       | Default | Purpose                                                    |
| ------------ | ------- | ---------------------------------------------------------- |
| `optionsApi` | `true`  | Resolve Vue Options API instance bindings inside templates |

Invalid options never fail the build: Vize reports them as option diagnostics positioned inside
your tsconfig (`vize1`–`vize3`) and continues with defaults. Vize also declares dependencies on
the project's `checkJs` and `noUnusedLocals` compiler options. Plain JavaScript `<script>` and
`<script setup>` blocks stay unchecked unless `checkJs` is enabled or a block opts in with a
leading `// @ts-check`; unused-local reporting follows each project's own configuration.

Vue checks belong in the selected tsconfig's top-level `vueCompilerOptions`:

```json
{
  "vueCompilerOptions": {
    "checkUnknownProps": true,
    "strictCssModules": true
  }
}
```

These options follow the existing tsconfig `extends` chain. `checkUnknownProps` defaults to
`strictTemplates` (otherwise `false`); an explicit `false` overrides that fallback.
`strictCssModules` defaults to `false`. Vize reports the configuration dependencies to the
TypeScript host so it can invalidate transformed files when an inherited setting changes.

## Template Directives

`@ts-expect-error` works as usual inside `<script>` blocks, which pass through verbatim. Template
expressions can't carry TS comments, so Vize maps the Vue-standard HTML comment directives through
the protocol instead:

```vue
<template>
  <!-- @vue-expect-error -->
  {{ count.toFixed(true) }}

  <!-- @vue-ignore -->
  {{ untypedThirdPartyValue.field }}
</template>
```

- `<!-- @vue-expect-error -->` suppresses TypeScript diagnostics on the next template node and
  reports `vize4: Unused '@vue-expect-error' directive` when nothing was suppressed.
- `<!-- @vue-ignore -->` suppresses silently.

A directive applies to the next template node, whether that node follows the comment on the same
line or after whitespace. On an element, `@vue-expect-error` and `@vue-ignore` cover the opening
tag, not its children. Put the comment directly before an interpolation or another child node to
target that child. `@vue-skip` covers the entire target-node subtree.

Placed directly before an SFC block, a directive covers that block's opening tag, such as a repeated
`<style module="name">` or a `generic` attribute; `<!-- @vue-skip -->` covers the whole block.

## Protocol

Vize uses the current Content Mapper initialization contract without numbered `protocolVersion`
negotiation. It selects UTF-8 position encoding, maintains a per-project
`openProject`/`closeProject` lifecycle, and emits `.tsx` virtual output so both TypeScript and
embedded JSX parse correctly.

The conformance workflow builds the native compiler from `microsoft/TypeScript` at exact revision
`d6c4afddb2c55f4a9dea7b59293a99a8fdea1799`, the merged
[Content Mappers round 2 change](https://github.com/microsoft/TypeScript/pull/63936).
It runs the full CLI, build-mode and LSP suites through packed npm artifacts, including authored
Vue declaration-map emission and navigation from a separately packed consumer back to `.vue`
source. This qualifies that exact host; it does not establish support in every published package.

Diagnostic codes reported under the `vize` source:

| Code    | Meaning                               |
| ------- | ------------------------------------- |
| `vize1` | Mapper options value is not an object |
| `vize2` | Unknown mapper option                 |
| `vize3` | Mapper option has the wrong type      |
| `vize4` | Unused `@vue-expect-error` directive  |

## Limitations

- Requires a host with the Content Mapper API and explicit external-code opt-in. The archived
  staging repository is not the current development source.
- The declaration-map request
  [microsoft/typescript-go#4860](https://github.com/microsoft/typescript-go/issues/4860)
  was completed by the round 2 change above. Vize's exact-host tests cover declaration-map
  emission and packed-consumer navigation; other host revisions require their own qualification.
- TypeScript 7.1 or newer is the recommended host. tsrs can run the same mapper config.
  Confirm that your installed compiler and editor expose content mappers before you rely on
  them in CI. [`vize check`](./cli.md#check) remains available for the packaged checker.
