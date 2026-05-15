---
title: SSR Rules
---

# SSR Rules

These rules cover code and template patterns that can break server rendering or hydration. They are
documented separately from HTML and Vapor rules because the failure mode is the server/client
boundary.

## `ssr/no-browser-globals-in-ssr`

Reports browser-only globals in code that can run during SSR.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

Good:

```vue
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

Guard checks such as `typeof window === "undefined"` are allowed because the direct `typeof`
identifier form is safe during server rendering. Strings, comments, and regex literals are also
ignored when they contain names like `window` or `document`. Accessing a member such as
`typeof window.innerWidth` still reports, because it evaluates the browser global.

## `ssr/no-hydration-mismatch`

Reports non-deterministic template values that can differ between server render and client
hydration.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

Good:

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```
