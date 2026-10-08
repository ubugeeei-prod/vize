## Summary

In a `<script lang="ts">` / `<script setup lang="ts">` block, `vize fmt` adds a trailing comma to the type parameters of a generic arrow function: `<T>(…) =>` becomes `<T,>(…) =>`. That form is only needed in TSX, where `<T>` would parse as a JSX tag. The same line in a plain `.ts` file is left alone, and Prettier / Oxfmt keep `<T>` in both.

## Reproduction

`Generic.vue`

```vue
<script setup lang="ts">
const first = <T>(items: readonly T[]): T | undefined => items[0];
const pick = <T = unknown>(value: T): T => value;

console.log(first([1]), pick(2));
</script>

<template>
  <p>x</p>
</template>
```

`plain.ts`

```ts
export const first = <T>(items: readonly T[]): T | undefined => items[0];
```

```sh
vize fmt --write --no-config Generic.vue plain.ts
```

`Generic.vue` becomes

```ts
const first = <T,>(items: readonly T[]): T | undefined => items[0];
const pick = <T = unknown,>(value: T): T => value;
```

`plain.ts` is unchanged.

## Expected

`Generic.vue` unchanged: the script block is `lang="ts"`, not `tsx`, so it should be printed like `plain.ts`. The comma only belongs in `lang="tsx"` (and `.tsx` files).

## Why it matters

Every generic arrow function in an SFC gets a diff against Prettier / Oxfmt output, and the SFC and a `.ts` file sharing the same helper end up formatted differently.

## Environment

- vize 0.432.0
- node 26.8.1, macOS arm64
