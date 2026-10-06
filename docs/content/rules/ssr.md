---
title: SSR Rules
---

# SSR Rules

These rules cover code and template patterns that can break server rendering or hydration. They are
documented separately from HTML and Vapor rules because the failure mode is the server/client
boundary.

## `ssr/no-browser-globals-in-ssr`

Reports browser-only globals in code that can run during SSR, including top-level reads in
`<script>` and `<script setup>`. The script check uses resolved AST references so local bindings,
parameters and type-only syntax do not become browser globals.

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

Script function bodies remain deferred unless directly invoked or passed to a named import of
`onServerPrefetch`, `watchSyncEffect`, or `watchEffect` with its default options from `vue`. Import
aliases keep that identity; unrelated functions with the same name do not. Mounted and event
callbacks are not executed by SSR. Script branches guarded by an exact `typeof name` comparison
with `"undefined"` may use that global on the branch where it exists. The probe must
refer to the runtime global; a local variable with the same name does not protect an outer
function's browser access. Normal script and setup keep separate local scopes while setup
imports remain visible in the module.

This is a bounded heuristic: external callback implementations, namespace-imported hooks,
reassigned functions, generator iteration, parameter default evaluation, instance construction
and Options API methods are not traced.
Existing template checks keep their established behavior, including directive expressions.

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
