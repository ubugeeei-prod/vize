---
title: Type Checking
description: Type-check Vue SFCs with TypeScript 7.1 content mappers. tsrs can run the same project.
---

# Type Checking

Type-check `.vue` files with TypeScript 7.1 or newer and Vize's
[content mapper](./content-mapper.md). TypeScript reports the same `TS####`
diagnostics on the line in your `.vue` file.

[tsrs](https://github.com/oxc-project/tsrs) can check that same project. It is a
Rust compiler for TypeScript 7.1, so you change the command, not the `tsconfig`.

> [!NOTE]
> Type checking is part of the alpha. Compare the results with your current
> `vue-tsc` command before you replace the CI step. See [Stability](../stability.md).

## Recommended: TypeScript 7.1

Content mappers ship in TypeScript 7.1, starting with the 7.1 beta. Install
`vize` and the compiler, then name the mapper in `tsconfig.json`:

```bash
vp install -D vize typescript@beta
```

```json
{
  "compilerOptions": {
    "module": "preserve",
    "strict": true,
    "noEmit": true
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

A content mapper is an external program. TypeScript runs it only when you opt in:

```bash
tsc --runExternalCode --noEmit -p tsconfig.json
```

That replaces `vue-tsc --noEmit`. After TypeScript 7.1 is stable, install
`typescript` at 7.1 or newer and keep the same command. Mapper options, template
directives, and editor setup are in the [Content Mapper guide](./content-mapper.md).

## tsrs

Install tsrs and pass the same flag. The `contentMappers` entry does not change:

```bash
vp install -D tsrs
tsrs --runExternalCode --noEmit -p tsconfig.json
```

Pin the tsrs version while you evaluate it. tsrs follows a TypeScript 7.1
snapshot, and a newer snapshot can change which diagnostics you see. Upstream
does not support `--init` or `--showConfig`. tsrs 0.18.0 does not publish
Windows or Intel macOS packages. For the editor, start `tsrs --lsp`.

## Coming from vue-tsc

| You use today                                 | With Vize                                                          |
| --------------------------------------------- | ------------------------------------------------------------------ |
| `vue-tsc --noEmit`                            | `tsc --runExternalCode --noEmit` with the mapper above             |
| `vue-tsc -p tsconfig.app.json --noEmit`       | `tsc --runExternalCode --noEmit -p tsconfig.app.json`              |
| the same project on tsrs                      | `tsrs --runExternalCode --noEmit -p tsconfig.app.json`             |
| Vue - Official extension diagnostics          | a TypeScript 7.1 host with content mappers, or the [Vize language server](./language-server.md) |

## Also available: `vize check`

`vize check` type-checks `.vue`, `.ts`, `.tsx`, and `.d.ts` files with the
checker packaged in `vize`. New projects should use the TypeScript 7.1 content
mapper above. Use `vize check` when you want that packaged command, including
virtual TypeScript inspection and the check server.

In a Vite+ project that uses the [Vize helper](./vite-plus.md), the generated
task still runs this command:

```bash
vp run typecheck
```

Install `vize` and run it directly when you are not on Vite+:

```bash
vp install -D vize
vp exec vize check
```

With no arguments, `vize check` reads `files`, `include`, and `exclude` from the nearest
`tsconfig.json`. Pass paths or a different project to narrow the run:

```bash
vize check src
vize check --tsconfig tsconfig.app.json
```

`vize check --tsrs-path node_modules/.bin/tsrs` selects tsrs as the packaged
checker's runtime. The recommended way to use tsrs is the `tsrs --runExternalCode`
command above, which runs the content mapper itself.

## What the output looks like

Given this component:

```vue
<script setup lang="ts">
const props = defineProps<{ step: number }>();
const emit = defineEmits<{ change: [value: number] }>();
</script>

<template>
  <button @click="emit('change', props.step.toFixed())">{{ props.stepp }}</button>
</template>
```

`vize check` reports both template mistakes at their source position. A
TypeScript 7.1 or tsrs run reports the same codes in that compiler's format:

```text
src/Counter.vue
  error:7:34 [TS2345] Argument of type 'string' is not assignable to parameter of type 'number'.
  error:7:66 [TS2551] Property 'stepp' does not exist on type ... Did you mean 'step'?

✗ Type checked 1 files in 487.30ms
  2 error(s)
```

The exit code is non-zero when there are errors, so the command can gate CI as is. Use
`--format json` for machine-readable output and `--max-warnings 0` to fail on warnings too.

## What gets checked

| Area                                      | Checked by default | Turn off with                     |
| ----------------------------------------- | ------------------ | --------------------------------- |
| `<script>` and `<script setup>` code      | Yes                | Exclude the file in `tsconfig`    |
| Template expressions and bindings         | Yes                | `--no-check-template-bindings`    |
| Props passed to child components          | Yes                | `--no-check-props`                |
| Emitted events and their payloads         | Yes                | `--no-check-emits`                |
| Options API template bindings (`data`, `computed`, `methods`, ...) | Yes | `vue.typeChecker.optionsApi: false` |
| Plain `.ts` / `.tsx` / `.d.ts` files       | Yes                | Exclude the file in `tsconfig`    |

The same switches are available in `vite.config.ts`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    typeChecker: {
      checkProps: true,
      checkEmits: true,
      checkTemplateBindings: true,
      optionsApi: true,
    },
  },
});
```

Set `vue.typeChecker.enabled: false` to leave type checking to your current tool. The same `vue`
section works with the plain Vite plugin; see the
[configuration reference](./configuration-reference.md) for every key.

## Global types and auto-imports

Vize only sees what your TypeScript project sees. Globals added through `declare module "vue"`,
and generated files such as `auto-imports.d.ts` or `components.d.ts`, must be matched by the
`include` globs of the `tsconfig.json` you check:

```json
{
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue", "src/**/*.d.ts"]
}
```

> [!TIP]
> If `$t`, `$route`, or an auto-imported component shows up as "does not exist", the declaration
> file is usually outside `include`. Add it there, or point `--tsconfig` at the project that has it.

`vue-tsc` runs on the JavaScript compiler. TypeScript 7.1 and tsrs run on a native
compiler. The
[type checker benchmark](../architecture/performance.md#benchmark-type-checker) shows measured
differences for `vize check`. Diagnostics can still differ in edge cases, which is why the
[migration guide](./migration.md#replace-a-check-command-after-comparing-output) asks you to
compare before switching.

## Debugging a diagnostic

Vize turns each SFC into virtual TypeScript before checking it. When a diagnostic looks wrong,
look at that generated code:

```bash
vize check --show-virtual-ts src/components/Counter.vue
```

`--save-virtual-ts-for <file>` writes it next to the component instead, and `--profile` writes
timings and all virtual files under `node_modules/.vize`. Include the output when you
[report a bug](./testing.md).

## Speed on large projects

These controls apply to `vize check`. A TypeScript 7.1 or tsrs run uses that
compiler's own project, not these flags.

`vize check` splits independent parts of the import graph across several native TypeScript
processes and picks the count from your CPU and file count. If memory is tight or you need
stable ordering, use one process:

```bash
vize check --servers 1
```

[Type-check performance](./check-performance.md) covers profiling and tuning in detail. For fast
repeated runs during development, keep a server running (Unix only) and point checks at it:

```bash
vize check-server --socket /tmp/vize-check.sock
vize check --socket /tmp/vize-check.sock src
```

## Related

- [Content Mapper](./content-mapper.md) for the TypeScript 7.1 setup
- [CLI reference: `vize check`](./cli.md#check)
- [Language server](./language-server.md) for type errors in the editor
- [Type-aware lint rules](../rules/type-and-script.md), which reuse the same checker
- [JSX and TSX type checking](./jsx.md)
