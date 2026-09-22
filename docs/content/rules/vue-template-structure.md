---
title: "Vue Rules: Template Structure"
---

# Vue Rules: Template Structure

Loops, conditional branches, template scope, and template containers.
See [all Vue rules](./vue.md) for the complete reference and [Vue Rule Options](./options-vue.md)
for configurable settings.

## `vue/require-v-for-key`

Requires every `v-for` node to have a stable key.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

Good:

```vue
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

## `vue/no-use-v-if-with-v-for`

Reports a node that has `v-if` and `v-for` at the same time. Filtering in a computed value keeps the
list identity stable and makes the template easier to analyze.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

Good:

```vue
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## `vue/no-child-content`

Reports child content on elements that also use `v-html` or `v-text`. Vue replaces the children at
runtime, so the authored content is misleading.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p v-text="message">Fallback text</p>
</template>
```

Good:

```vue
<template>
  <p v-text="message" />
</template>
```

## `vue/no-dupe-v-else-if`

Reports repeated conditions in a `v-if` / `v-else-if` chain.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

Good:

```vue
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

## `vue/no-template-shadow`

Reports template variables that shadow variables from an outer scope. This prevents accidental
references to a different value than the reader expects.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const item = ref("selected");
</script>

<template>
  <p v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

Good:

```vue
<script setup lang="ts">
const selectedItem = ref("selected");
</script>

<template>
  <p v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

## `vue/no-lone-template`

Reports a `<template>` element that has no structural directive. `v-if`,
`v-else-if`, `v-else`, `v-for`, and `v-slot` justify the wrapper. The SFC's
own root `<template>` block is not this element.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <div>
    <template>
      <p>{{ message }}</p>
    </template>
  </div>
</template>
```

Good:

```vue
<template>
  <template v-if="ready">
    <p>{{ message }}</p>
  </template>
  <template v-for="item in items" :key="item.id">
    <p>{{ item.name }}</p>
  </template>
  <BaseCard>
    <template #header>
      <h2>{{ title }}</h2>
    </template>
  </BaseCard>
</template>
```

## `vue/no-template-key`

Reports `key` on a `<template>` element that is not a `v-for`. A `v-for`
template may carry the key. Put `key` on a real element otherwise.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <template :key="section">
    <div>{{ item }}</div>
  </template>
</template>
```

Good:

```vue
<template>
  <template v-for="item in items" :key="item.id">
    <div>{{ item.name }}</div>
  </template>
  <section :key="section">
    <div>{{ item }}</div>
  </section>
</template>
```

## `vue/no-unused-vars`

Reports a variable introduced by `v-for` or `v-slot` that the template never
reads. A name that starts with `_` is treated as intentionally unused.

Default severity: `warning`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

Good:

```vue
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

## `vue/no-useless-template-attributes`

Reports an attribute on `<template>` that Vue ignores. A `<template>` element
only keeps structural directives (`v-if`, `v-else-if`, `v-else`, `v-for`,
`v-slot`) and the `key` that belongs to a `v-for`.

Default severity: `error`\
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <BaseCard>
    <template v-if="show" class="panel"><div /></template>
    <template v-for="item in items" id="list"><div /></template>
    <template #header ref="header"><div /></template>
  </BaseCard>
</template>
```

Good:

```vue
<template>
  <BaseCard>
    <template v-if="show"><div /></template>
    <template v-for="item in items" :key="item.id"><div /></template>
    <template #header><div /></template>
  </BaseCard>
</template>
```
