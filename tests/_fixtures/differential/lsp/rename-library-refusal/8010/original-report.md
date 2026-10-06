### Area

Language server (`vize lsp --stdio`): rename on an event name

### Version

`vize` 0.432.0

### Minimal reproduction

`package.json` with `vue@3.5`, `typescript@5`; `tsconfig.json` with `"lib": ["ESNext", "DOM"]`, `strict`, `moduleResolution: "Bundler"`, including `src/**/*.vue`.

`src/Toggle.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{
  change: [value: boolean];
}>();

function flip() {
  emit("change", true);
}
</script>

<template>
  <button @click="flip">toggle</button>
</template>
```

`src/App.vue`

```vue
<script setup lang="ts">
import Toggle from "./Toggle.vue";

function onChange(value: boolean) {
  console.log(value);
}
</script>

<template>
  <Toggle @change="onChange" />
</template>
```

Request: `textDocument/rename` in `Toggle.vue` at `6:9` (inside `"change"` of `emit("change", true)`), `newName: "update"`.

### Actual

```
Toggle.vue 6:8-6:14 "change" -> "update"
node_modules/@typescript/typescript-darwin-arm64/lib/lib.dom.d.ts 10986:5-10986:11 "change" -> "update"
node_modules/@typescript/typescript-darwin-arm64/lib/lib.dom.d.ts 16666:5-16666:11 "change" -> "update"
node_modules/@typescript/typescript-darwin-arm64/lib/lib.dom.d.ts 24244:5-24244:11 "change" -> "update"
node_modules/@typescript/typescript-darwin-arm64/lib/lib.dom.d.ts 28243:5-28243:11 "change" -> "update"
node_modules/@typescript/typescript-darwin-arm64/lib/lib.dom.d.ts 34714:5-34714:11 "change" -> "update"
node_modules/@typescript/typescript-darwin-arm64/lib/lib.dom.d.ts 36810:5-36810:11 "change" -> "update"
```

An editor that applies the workspace edit rewrites TypeScript's bundled `lib.dom.d.ts` (the `change` keys of the DOM event maps). The `change:` key in `defineEmits` (2:2) and the parent's `@change` are not edited.

### Expected

Either rename the event consistently (the `defineEmits` key, every `emit("change")` and the listeners `@change` in parents) or refuse the rename. Never return edits for files outside the workspace / inside `node_modules` (TypeScript itself refuses to rename symbols declared in lib files).

### Likely cause

`crates/vize_maestro/src/ide/corsa_support/canonical/rename.rs`, `map_text_edit`: edits for non-virtual URIs are passed through unchanged, with no library / `node_modules` / outside-workspace filter. The string literal in `emit("change")` resolves to the union of event-map keys in the virtual TS, so Corsa returns every `change` key it knows.

### Environment

- OS: macOS 26.4.1
- Architecture: arm64
- Node.js: 26.8.1
- Package manager: npm 11.19.0
- vue 3.5.x, typescript 5.9.3
