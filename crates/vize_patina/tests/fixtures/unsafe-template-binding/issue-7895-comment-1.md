Two more binding shapes on component elements, with the same root cause as the call results above. A native element with the same expression is not reported, and the result is identical with `--type-aware` (0.432.0):

- a template literal that uses a typed prop: ``:title="`tone-${tone}`"``
- `:is` bound to an imported SFC: `<component :is="ChildPanel" …>`

`ChildPanel.vue`

```vue
<script setup lang="ts">
defineProps<{ title?: string }>();
defineEmits<{ close: [] }>();
</script>

<template>
  <section :title="title"><slot /></section>
</template>
```

`ParentPanel.vue`

```vue
<script setup lang="ts">
import ChildPanel from "./ChildPanel.vue";

const { tone = "neutral" } = defineProps<{ tone?: "neutral" | "accent" }>();
const emit = defineEmits<{ dismiss: [] }>();
</script>

<template>
  <div>
    <ChildPanel :title="`tone-${tone}`" @close="() => emit('dismiss')" />
    <component :is="ChildPanel" :title="`tone-${tone}`" />

    <!-- not reported: the same expressions on a native element -->
    <section :title="`tone-${tone}`" @click="() => emit('dismiss')"></section>

    <!-- not reported: plain bindings on the component -->
    <ChildPanel :title="tone" @close="emit('dismiss')" />
  </div>
</template>
```

```
ParentPanel.vue:10:25 error type/no-unsafe-template-binding Template binding resolves to an unsafe `any` or `unknown` type          <- `tone-${tone}` on ChildPanel
ParentPanel.vue:10:55 error type/no-unsafe-template-binding Template event handler calls a value with an unsafe `any` or `unknown` type   <- () => emit('dismiss')
ParentPanel.vue:11:21 error type/no-unsafe-template-binding Template binding resolves to an unsafe `any` or `unknown` type          <- :is="ChildPanel"
ParentPanel.vue:11:41 error type/no-unsafe-template-binding Template binding resolves to an unsafe `any` or `unknown` type          <- `tone-${tone}` on <component>
```

`vize check`: `No type errors found!`
