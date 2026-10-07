One more shape where the shorthand rename produces **overlapping / duplicated** edits rather than just the wrong name (vize 0.432.0, all three files open):

`Panel.vue`

```vue
<script setup lang="ts">
defineProps<{ isOpened: boolean; heading?: string }>();
</script>

<template>
  <section v-if="isOpened">{{ heading }}</section>
</template>
```

`Wrapper.vue`

```vue
<script setup lang="ts">
import Panel from "./Panel.vue";

const { isOpened } = defineProps<{ isOpened: boolean }>();
const heading = "Title";
</script>

<template>
  <Panel
    :is-opened
    :heading
  />
</template>
```

`App.vue`

```vue
<script setup lang="ts">
import Panel from "./Panel.vue";
import Wrapper from "./Wrapper.vue";
</script>

<template>
  <Panel :is-opened="true" />
  <Wrapper :is-opened="false" />
</template>
```

`rename` in `Wrapper.vue` on the destructured `isOpened` (3:10) → `visible` returns

```
App.vue     7:12-7:21 "is-opened" -> "visible"
Wrapper.vue 3:8-3:16  "isOpened"  -> "isOpened: visible"
Wrapper.vue 3:35-3:43 "isOpened"  -> "visible"
Wrapper.vue 9:5-9:14  "is-opened" -> "visible"
Wrapper.vue 9:14-9:14 ""          -> "visible"
```

The last two edits together produce `:visiblevisible`. `rename` from `App.vue` on `:is-opened` of `<Panel>` (6:11) gives `Wrapper.vue` two overlapping edits (`9:5-9:13` and `9:5-9:14`), which LSP clients reject or apply inconsistently, and it misses `v-if="isOpened"` in `Panel.vue` (5:17) although `references` returns that location. `references` itself contains `9:5-9:13 "is-opene"` (camelCase length applied to the kebab-case attribute) and a zero-length `9:14-9:14`.

This looks like the canonical/Corsa rename path not expanding shorthands, plus `merge_missing_authored_rename` appending the authored edit on top.
