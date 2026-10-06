## Version

- `vize` 0.432.0 (npm, `node_modules/.bin/vize lsp --stdio`), serverInfo `vize-maestro 0.432.0`
- `vue` 3.5.41, Node 26.6.0, macOS (arm64)

Follow-up to #7194 (tag-name completion now offers imported components and HTML elements — thanks).

## Minimal reproduction

`tsconfig.json`: as in #7194 (`moduleResolution: "Bundler"`, `include: ["src/**/*.ts", "src/**/*.vue"]`).

`vize.config.json`:

```json
{ "languageServer": { "typecheck": true, "editor": true } }
```

`src/MyButton.vue`:

```vue
<template>
  <button type="button"><slot /></button>
</template>
```

`src/App.vue`:

```vue
<script setup lang="ts">
import { ref } from "vue";
import MyButton from "./MyButton.vue";

const SORT_OPTIONS = ["asc", "desc"] as const;
const DialogState = { None: "None", Open: "Open" } as const;
const count = ref(0);
function increment(): void {
  count.value += 1;
}
</script>

<template>
  <MyButton @click="increment">{{ count }} {{ SORT_OPTIONS[0] }} {{ DialogState.None }}</MyButton>
  <
</template>
```

## Steps

`vize lsp --stdio`: `initialize`, `initialized`, `didOpen` `src/App.vue`, then `textDocument/completion` at line 15 after the typed prefix (the `<` line is replaced by `didChange` with each prefix below).

## Actual

Script-setup names in the result (all are `kind: 7` with `detail: "Component in script setup"`):

```
"<"    -> 199 items; ["DialogState","MyButton","SORT_OPTIONS","dialog-state","my-button"]
"<Di"  -> 1 item;    ["DialogState"]
"<di"  -> 4 items;   ["dialog-state"]          (next to dialog, div, …)
"<SO"  -> 1 item;    ["SORT_OPTIONS"]
"<co"  -> 5 items;   []                        (count is not offered)
```

`SORT_OPTIONS` (a readonly tuple) and `DialogState` (a plain object) are offered as components, including the kebab-case form `dialog-state`, which sorts between `dialog` and `div`. Accepting one produces `<SORT_OPTIONS` / `<dialog-state>`, which Vue resolves as an unknown element. It looks like any script-setup binding whose name starts with an upper-case letter is treated as a component.

In a real page with a dozen `UPPER_CASE` option arrays and `as const` enums, these make up a noticeable share of the `<` list.

## Expected

Only bindings that can be used as a component are offered as tag names: imported `.vue` default exports, `defineComponent(...)` / `defineAsyncComponent(...)` results, and other values whose type is a component (the checker is available here: `typecheck` is on). Plain objects, arrays, strings and functions are not offered. When the type is unknown, keeping the name-based guess is fine, but values whose type is known not to be a component should be excluded.

## Why

A `<script setup>` binding only works as a tag when its value is a component (Vue's compiler resolves `<SORT_OPTIONS>` to `$setup.SORT_OPTIONS` and renders the array as a component, which fails at runtime). The candidates added in #7194 are documented as "script-setup values"; the distinction between values and components is what this asks for. The `component-required-props` / hover code already knows which bindings are components (`hover` on `<MyButton` shows `const MyButton: VueComponent`).

If the name-based guess is intentional (it is cheap and works without the checker), filtering by type when `typecheck` is on would still remove these.
