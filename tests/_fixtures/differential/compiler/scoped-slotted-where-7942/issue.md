## Summary

Scoped CSS changes how **every rule** in a `<style scoped>` block is rewritten as soon as the block contains a `:slotted()` selector anywhere. With `:slotted()` present:

1. `:where(.x)` gets the scope attribute **outside** the `:where()` again (`[data-v]:where(.x)`, specificity (0,1,0)). This is what #6985 fixed for blocks without `:slotted()`.
2. `:slotted(:where(p))` becomes `[data-v-s]:where(p)` instead of `:where(p[data-v-s])`.
3. `.x > :slotted(p)` scopes `.x` (`.x[data-v] > p[data-v-s]`), which @vue/compiler-sfc does not do (see #6986).

The specificity changes are visible in real UIs: a rule written with `:where()` to stay below utility classes starts winning over them (in our case a `line-height` utility on slotted content was overridden, shifting list text by a few pixels).

Without any `:slotted()` in the block, the output matches @vue/compiler-sfc.

## Reproduction

Via `@vizejs/native`'s `compileCss(source, { scoped: true, scopeId: "data-v-1" })` (the Vite plugin and `compileSfc` produce the same selectors):

| input | @vue/compiler-sfc 3.5.38 | vize 0.432.0 |
| --- | --- | --- |
| `:where(.box) { color: red; }` | `:where(.box[data-v-1])` | `:where(.box[data-v-1])` ✅ |
| `:where(.box) { color: red; }`<br>`.x > :slotted(p) { margin: 0; }` | `:where(.box[data-v-1])`<br>`.x > p[data-v-1-s]` | `[data-v-1]:where(.box)` ❌<br>`.x[data-v-1] > p[data-v-1-s]` ❌ |
| `:where(.box) > :slotted(:where(p)) { line-height: 1.1; }` | `:where(.box[data-v-1]) > :where(p[data-v-1-s])` | `[data-v-1]:where(.box) > [data-v-1-s]:where(p)` ❌ |
| `:where(.box) { color: red; }`<br>`.x :deep(p) { margin: 0; }` | `:where(.box[data-v-1])`<br>`.x[data-v-1] p` | same ✅ |

A component using it:

```vue
<template>
  <div class="box"><slot /></div>
</template>

<style scoped>
:where(.box) {
  color: red;
}

:where(.box) > :slotted(:where(p)) {
  line-height: 1.1;
}
</style>
```

## Versions

Rows 2 and 3 start in 0.430.1 and are unchanged in 0.431.0 and 0.432.0. In 0.429.2 row 2 gave `:where(.box[data-v-1])` / `.x > p[data-v-1-s]` (same as Vue), and row 3 gave `:where(.box) > [data-v-1-s]:where(p)`.

## Expected

The same output as @vue/compiler-sfc, regardless of whether the block contains `:slotted()`: the scope attribute goes inside `:where()` / `:is()`, `:slotted(:where(x))` becomes `:where(x[data-v-s])`, and the compound before `> :slotted()` is not scoped.

## Environment

- @vizejs/native / @vizejs/vite-plugin 0.432.0
- @vue/compiler-sfc 3.5.38
- macOS arm64, Node 26
