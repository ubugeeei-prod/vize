---
title: "script/consistent-type-imports"
description: "Enforce consistent usage of type imports, counting template uses as value uses."
---

# `script/consistent-type-imports`

Enforce consistent usage of type imports, counting template uses as value uses.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | None | Yes | [Rule options](../options.md) |

The rule checks Vue SFC scripts and standalone TypeScript (`.ts`) modules.

## What counts as a value use

Every use the component makes of an import counts, through the same binding analysis the compiler reads: a template expression, a component tag in any casing Vue resolves, a custom directive, a template ref, a `<style>` `v-bind()`, and the other script block.

A `v-for` or slot alias of the same name shadows the import. A type position, such as `(item as Foo)` in the template or the `generic` attribute, is a type use.

When a block cannot be read (an external `src` block, an expression that does not parse, direct `eval`, Musea `<variant>` markup), the rule reports nothing for that component.

A name imported with `import type` that a template expression or a `<style>` `v-bind()` evaluates is reported as well, because that binding does not exist at runtime. The template read is left to `vue/no-undefined-refs` when that rule is active. Component tags and directives are not reported: a registered or auto-imported component can render them.

## Options

`prefer`, `fixStyle` and `disallowTypeAnnotations` are the typescript-eslint options. `emitDecoratorMetadata` (default `true`) says whether the project emits decorator metadata, which lint does not read from `tsconfig.json`. With `true`, a type that a decorated signature serializes counts as a value use.

## The fix

A split that keeps a value import preserves its evaluation order. Converting the whole declaration to `import type` can remove module initialization under `verbatimModuleSyntax`. A declaration that must split and contains a comment or import attributes is reported without a fix.

## Bad

`Foo` is imported as a value, but its only use is the type argument of `defineProps`. `Bar` is not reported: the template reads `Bar.label`, which is a value use.

```vue annotate="remove:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## Good

`Foo` is imported with `import type`, so the import is erased at build time. `Bar` stays a value import because the template evaluates it.

```vue annotate="add:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import type { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-type-imports": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Differences from typescript-eslint

typescript-eslint needs `vue-eslint-parser` to see a template use, and oxlint skips `.vue` files for this rule. A decorated signature counts as a value use unless `emitDecoratorMetadata` is `false`; typescript-eslint reads `tsconfig.json` and then skips every file that contains a decorator. The fix never merges into another declaration. A type-only import that a template expression or a `<style>` `v-bind()` evaluates is reported.

## Related

- [Type and script rules](../type-and-script.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/consistent_type_imports.rs#L55)
