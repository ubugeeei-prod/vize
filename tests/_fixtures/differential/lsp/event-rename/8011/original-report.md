### Area

Language server (`vize lsp --stdio`): references / rename for event, slot, and type-alias prop declarations

### Version

`vize` 0.432.0

### Minimal reproduction

`package.json` with `vue@3.5`, `typescript@5`; `tsconfig.json` with `strict`, `moduleResolution: "Bundler"`, including `src/**/*.vue`.

**Events**: `src/Toggle.vue`

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

with a parent `src/App.vue` using `<Toggle @change="onChange" />`.

**Slots**: `src/Card.vue`

```vue
<script setup lang="ts">
defineSlots<{
  header(props: { title: string }): unknown;
}>();
</script>

<template>
  <section>
    <slot name="header" title="Hello" />
  </section>
</template>
```

`src/CardUser.vue`

```vue
<script setup lang="ts">
import Card from "./Card.vue";
</script>

<template>
  <Card>
    <template #header="{ title }">{{ title }}</template>
  </Card>
</template>
```

**Props from a local type alias**: `src/E.vue`

```vue
<script setup lang="ts">
type Props = {
  hidden?: boolean;
  tone?: "dark" | "light";
};
defineProps<Props>();
</script>

<template>
  <span v-if="!hidden" :class="tone">x</span>
</template>
```

### Actual

| file | request at | result |
| --- | --- | --- |
| `Toggle.vue` | `references` on the `change:` key (2:3) | only `2:2-2:8`; neither `emit("change", true)` nor the parent's `@change` |
| `Toggle.vue` | `rename` on the `change:` key / from the parent's `@change` | never includes `emit("change", true)` |
| `Card.vue` | `references` / `rename` on the `header` key (2:3) | declaration only; `<slot name="header">` and `#header` go stale |
| `CardUser.vue` | `rename` on `#header` | edits `#header` and `<slot name>` but not the `defineSlots` key |
| `E.vue` | `references` on `hidden` (2:3) | only `2:2-2:8` |
| `E.vue` | `rename` `hidden` → `concealed` (2:3) | only the type member; `!hidden` in the template is left, so the template references a prop that no longer exists |
| `E.vue` | `definition` on `hidden` in the template (9:16) | `5:0-5:11` = `defineProps` |

The same component with an inline type literal (`defineProps<{ hidden?: boolean; … }>()`) works: references, rename and definition all include the template use. `withDefaults(defineProps<Props>(), …)` and a partial destructure (`const { html = "" } = defineProps<Props>()`, with the other props used by their bare names in the template) fail the same way as the alias.

### Expected

Event keys, slot keys and props declared through a type alias are linked to their template / script uses (and to parent listeners and slot templates), so references are complete and rename keeps the code consistent.

### Notes

- #7917 covers the *definition* half for props interfaces; this is about references / rename, where the template is silently left out of the edit.
- For the alias case, `vize check --show-virtual-ts` shows the alias hoisted to module scope without a mapping back to the SFC.

### Environment

- OS: macOS 26.4.1
- Architecture: arm64
- Node.js: 26.8.1
- Package manager: npm 11.19.0
- vue 3.5.x, typescript 5.9.3
