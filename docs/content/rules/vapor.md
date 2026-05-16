---
title: Vapor Rules
---

# Vapor Rules

These rules cover template constraints for Vapor-oriented components and apps. Composition API and
script-level Vapor guidance lives in [Type and script rules](./type-and-script.md).

## `vapor/no-vue-lifecycle-events`

Reports per-element lifecycle events such as `@vue:mounted`.

Default severity: `error`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <input @vue:mounted="focusInput" />
</template>
```

Good:

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

## `vapor/require-vapor-attribute`

Suggests adding `vapor` to `<script setup>` when the preset expects Vapor-compatible components.

Default severity: `warning`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const count = ref(0);
</script>
```

Good:

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `vapor/no-inline-template`

Reports the deprecated `inline-template` attribute.

Default severity: `error`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

Good:

```vue
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

## `vapor/prefer-static-class`

Reports dynamic `:class` bindings whose value is a static string literal.

Default severity: `warning`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

Good:

```vue
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```
