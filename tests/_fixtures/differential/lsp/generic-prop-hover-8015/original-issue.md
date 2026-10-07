### Area

Language server (`vize lsp --stdio`): template completion and hover on generic component props

### Version

`vize` 0.432.0

### Minimal reproduction

`package.json` with `vue@3.5`, `typescript@5`; `tsconfig.json` with `strict`, `moduleResolution: "Bundler"`, including `src/**/*.vue`.

`src/Child.vue`

```vue
<script setup lang="ts">
defineProps<{
  label?: string;
  size?: "small" | "large";
}>();
</script>

<template>
  <button type="button">{{ label }}</button>
</template>
```

`src/Comp.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
const emit = defineEmits<{ done: [] }>();
const value = 3;
</script>

<template>
  <Child class="wide" size="large" data-role="x" />
  <div class="box" @click="$emit('done')">{{ String(value) }} {{ Math.max(value, 1) }}</div>
</template>
```

`src/List.vue`

```vue
<script setup lang="ts" generic="T extends { id: number }">
defineProps<{
  items: T[];
  title?: string;
}>();
defineEmits<{ pick: [item: T] }>();
</script>

<template>
  <ul :title="title">
    <li v-for="item in items" :key="item.id">{{ item.id }}</li>
  </ul>
</template>
```

`src/App.vue`

```vue
<script setup lang="ts">
import List from "./List.vue";

const rows = [{ id: 1, name: "a" }];
function onPick(row: { id: number; name: string }) {
  console.log(row.name);
}
</script>

<template>
  <List :items="rows" title="Rows" @pick="onPick" />
</template>
```

### Actual

**Completion in `Comp.vue`**

| position | items |
| --- | --- |
| `<Child cl\|ass=…` (7:11) | 0 (on a `<div>`, `class` is offered) |
| `<Child … dat\|a-role=…` (7:38) | 0 |
| `@click="$e\|mit(…)"` (8:29) | `$event`, `Child`, `emit`, `value` only; no `$emit`, `$attrs`, `$slots`, `$props` |
| `{{ St\|ring(value) }}` (8:47) | local bindings only; no `String` |
| `{{ Ma\|th.max(…) }}` (8:67) | local bindings only; no `Math` |

**Hover in `App.vue`** on the props of the generic component, `:items` (10:11) and `title` (10:24):

```
**items**
_Component prop_
unknown
```

Completion detail for the same props says `prop: T[]` / `string`, and hover on `@pick` shows the instantiated event type, so the information is available.

### Expected

- Global attributes (`class`, `style`, `id`, `data-*`, ARIA, …) are offered on component tags as on native elements (they fall through by default).
- `$emit`, `$attrs`, `$slots`, `$props`, `$el`, … and allowed template globals (`String`, `Math`, `Number`, `JSON`, `Array`, …) are offered in template expressions.
- Hover on a generic component's prop shows the instantiated type (`{ id: number; name: string }[]`) or at least the declared `T[]`.

### Environment

- OS: macOS 26.4.1
- Architecture: arm64
- Node.js: 26.8.1
- Package manager: npm 11.19.0
- vue 3.5.x, typescript 5.9.3
