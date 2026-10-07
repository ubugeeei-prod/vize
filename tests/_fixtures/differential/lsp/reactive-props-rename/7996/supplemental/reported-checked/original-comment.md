**Same root cause, starting from the parent (Vize 0.432.0): lsp(rename): renaming a prop from the parent's attribute does not update the child's `defineProps` destructure**

## Version

- `vize` 0.432.0 (npm, `node_modules/.bin/vize lsp --stdio`), serverInfo `vize-maestro 0.432.0`
- `vue` 3.5.41, Node 26.6.0, macOS (arm64)

## Minimal reproduction

`tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "jsx": "preserve",
    "skipLibCheck": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts", "src/**/*.vue"]
}
```

`vize.config.json`:

```json
{ "languageServer": { "typecheck": true, "editor": true } }
```

`src/Child.vue`:

```vue
<script setup lang="ts">
const { checked } = defineProps<{
  checked: boolean;
}>();
</script>

<template>
  <span>{{ checked ? "on" : "off" }}</span>
</template>
```

`src/Parent.vue`:

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>

<template>
  <Child :checked="true" />
</template>
```

## Steps

`vize lsp --stdio`: `initialize` (project as `rootUri`), `initialized`, `didOpen` both files (so #7824 does not apply), then `textDocument/rename` with `newName: "active"`:

1. on `checked` in `Child.vue`'s `defineProps` type (line 3, character 2)
2. on `checked` in `Parent.vue`'s `:checked` attribute (line 6, character 11)

## Actual

From the declaration (1), the edit is complete — the destructure keeps its local name:

```
src/Child.vue:  [["2:9-2:16","active: checked"],["3:3-3:10","active"]]
src/Parent.vue: [["6:11-6:18","active"]]

const { active: checked } = defineProps<{
  active: boolean;
}>();
```

From the attribute (2), the destructure is left alone:

```
src/Child.vue:  [["3:3-3:10","active"]]
src/Parent.vue: [["6:11-6:18","active"]]

const { checked } = defineProps<{
  active: boolean;
}>();
```

After applying it, the server itself reports the result as broken:

```
Child.vue 2:9  error  vize/types(2339) Property 'checked' does not exist on type '__DefineProps<__LooseRequired<{ active: boolean; }>, "active">'.
Child.vue 3:3  warn   vize/lint(vue/no-unused-properties) Prop 'active' is defined but never used
```

## Expected

Renaming the prop gives the same `WorkspaceEdit` whichever occurrence the rename starts from (the declaration in `defineProps`, or a `:checked` / `checked` attribute in a parent): the destructuring pattern becomes `{ active: checked }` (or `{ active }` plus the local uses) in both cases.

## Why

Rename is defined on the symbol, not on the cursor position: `textDocument/references` from the same attribute already returns the declaration and the attribute, and the declaration-side rename already knows to rewrite the shorthand binding. #3892 set the same rule ("renaming the prop — from the attribute name or Child's `defineProps` — rewrites attribute names plus Child's `defineProps`; both directions should hold"); the destructured form is the case that still diverges. The current attribute-side result destructures a prop that no longer exists, so `checked` silently becomes `undefined` at runtime.

Related but different: #7824 (files that are not open are skipped). Here both files are open.
