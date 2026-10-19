---
title: Type Checking
description: Type-check Vue SFCs, TypeScript, and TSX with vize check and TypeScript's native compiler.
---

# Type Checking

`vize check` type-checks `.vue`, `.ts`, `.tsx`, and `.d.ts` files in one project-wide pass. It is the
Vize counterpart of `vue-tsc --noEmit`: it reports the same TypeScript diagnostics (with the same
`TS####` codes), pointing at the line in your `.vue` file.

> [!NOTE]
> Type checking is part of the alpha. Run it next to your current `vue-tsc` command and compare the
> results on your project before you replace the CI step. See [Stability](../stability.md).

## Run it

In a Vite+ project that uses the [Vize helper](./vite-plus.md), run the generated task:

```bash
vp run typecheck
```

Without Vite+, install the `vize` package and run the command directly. The package already
includes TypeScript's native compiler for your platform, so there is nothing else to install.

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

`vize check` reports both template mistakes at their source position:

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
| Options API template bindings (`data`, `computed`, `methods`, ...) | Yes | `typeChecker.optionsApi: false` |
| Plain `.ts` / `.tsx` / `.d.ts` files       | Yes                | Exclude the file in `tsconfig`    |

The same switches are available in `vite.config.ts`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  typecheck: {
    checkProps: true,
    checkEmits: true,
    checkTemplateBindings: true,
    optionsApi: true,
  },
});
```

Set `typecheck: false` to leave type checking to your current tool. Outside the Vite+ helper the
same object is called `typeChecker`; see the
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

## Coming from vue-tsc

| You use today                              | With Vize                                         |
| ------------------------------------------ | ------------------------------------------------- |
| `vue-tsc --noEmit`                         | `vize check`                                      |
| `vue-tsc -p tsconfig.app.json --noEmit`    | `vize check --tsconfig tsconfig.app.json`         |
| `vue-tsc --declaration --emitDeclarationOnly` | `vize check --declaration --declaration-dir dist/types` |
| Vue - Official extension diagnostics       | [Vize language server](./language-server.md) with `typecheck` enabled |

The checker runs on TypeScript 7's native compiler (the Go port, also called Project Corsa), while
`vue-tsc` runs on the JavaScript compiler. The
[type checker benchmark](../architecture/performance.md#benchmark-type-checker) shows the measured
difference. Diagnostics can still differ in edge cases, which is why the
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

- [CLI reference: `vize check`](./cli.md#check)
- [Language server](./language-server.md) for type errors in the editor
- [Type-aware lint rules](../rules/type-and-script.md), which reuse the same checker
- [JSX and TSX type checking](./jsx.md)
