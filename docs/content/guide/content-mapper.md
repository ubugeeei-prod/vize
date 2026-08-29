---
title: TypeScript Content Mapper
---

# TypeScript Content Mapper

Content Mappers are TypeScript's plugin surface for checking file types the compiler cannot parse
itself — the [TypeScript 7.1 API roadmap](https://github.com/microsoft/typescript-go/issues/4830)
identifies them as the TS Server plugin replacement needed by Vue. The API merged into
`typescript-go` main in [microsoft/typescript-go#4712](https://github.com/microsoft/typescript-go/pull/4712).

Vize ships a conforming content mapper inside the `vize` npm package: a `tsgo` build with
content-mapper support spawns `vize content-mapper` and checks `.vue` files directly — hover,
go-to-definition, rename, completions, and diagnostics all map back to your authored SFC, with no
parallel `.vue.ts` project to materialize.

> **⚠️ Preview:** Content Mappers are merged upstream but not yet in the released TypeScript 7
> platform packages. Until a release ships the protocol, build a content-mapper-enabled native
> TypeScript binary from `typescript-go` main and keep [`vize check`](./cli.md#check) as the
> supported typecheck path.

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

Running external mapper processes requires explicit opt-in:

```bash
tsgo --runExternalCode --noEmit -p tsconfig.json
```

In VS Code, the Vize extension registers `.vue` support with the TypeScript 7
content-mapper host automatically in trusted workspaces — the same mapper then powers the editor.

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

| Option       | Default | Purpose                                                       |
| ------------ | ------- | ------------------------------------------------------------- |
| `optionsApi` | `true`  | Resolve Vue Options API instance bindings inside templates    |

Invalid options never fail the build: Vize reports them as option diagnostics positioned inside
your tsconfig (`vize1`–`vize3`) and continues with defaults. Vize also declares a dependency on
the project's `noUnusedLocals` compiler option, so unused-local reporting inside `<script setup>`
follows each project's own configuration.

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

- `<!-- @vue-expect-error -->` suppresses TypeScript diagnostics on the next template line and
  reports `vize4: Unused '@vue-expect-error' directive` when nothing was suppressed.
- `<!-- @vue-ignore -->` suppresses silently.

A directive applies to the rest of its own line when content follows the comment, otherwise to the
next non-empty line.

## Protocol

Vize speaks content-mapper protocol v1 as merged upstream: UTF-8 position encoding, per-project
`openProject`/`closeProject` lifecycle, and `.tsx` virtual output so both TypeScript and embedded
JSX parse correctly. Conformance is enforced in CI against a pinned `typescript-go` revision that
builds the exact upstream compiler and runs the full CLI, build-mode, and LSP suites through the
packed npm artifacts.

Diagnostic codes reported under the `vize` source:

| Code    | Meaning                                             |
| ------- | ---------------------------------------------------- |
| `vize1` | Mapper options value is not an object                |
| `vize2` | Unknown mapper option                                |
| `vize3` | Mapper option has the wrong type                     |
| `vize4` | Unused `@vue-expect-error` directive                 |

## Limitations

- Requires a `tsgo` built from `typescript-go` main until a TypeScript 7 release ships the API.
- Declaration maps for mapped inputs await
  [microsoft/typescript-go#4860](https://github.com/microsoft/typescript-go/issues/4860).
- `vize check` remains the supported production typecheck path while the upstream API is in
  preview.
