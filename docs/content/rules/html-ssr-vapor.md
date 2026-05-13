---
title: HTML SSR And Vapor Rules
---

# HTML, SSR, And Vapor Rules

These rules cover HTML validity, server-rendering hazards, and Vapor-only template constraints.
They are grouped together because they often protect rendering stability rather than component API
style.

## `html/id-duplication`

Reports duplicate static IDs inside one template.

Default severity: `error`  
Presets: `essential`, `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

Good:

```vue
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## `html/deprecated-element`

Reports deprecated HTML elements.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <center>Profile</center>
</template>
```

Good:

```vue
<template>
  <section class="profile">Profile</section>
</template>
```

## `html/deprecated-attr`

Reports deprecated HTML attributes.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <table border="1">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

Good:

```vue
<template>
  <table class="summary">
    <tr>
      <td>Total</td>
    </tr>
  </table>
</template>
```

## `html/no-consecutive-br`

Reports consecutive `<br>` elements used for layout.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

Good:

```vue
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## `html/require-datetime`

Requires machine-readable `datetime` values on `<time>`.

Default severity: `warning`  
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<template>
  <time>May 13, 2026</time>
</template>
```

Good:

```vue
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

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

## `vapor/no-suspense`

Reports `<Suspense>` in Vapor-only apps.

Default severity: `warning`  
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template>
  <Suspense>
    <AsyncPanel />
  </Suspense>
</template>
```

Good:

```vue
<script setup lang="ts" vapor>
const panel = await loadPanelData();
</script>

<template>
  <AsyncPanel :data="panel" />
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

## Additional HTML And Vapor Rules

`html/no-duplicate-dt` reports duplicate `<dt>` names in a definition list. Default: `warning`.
Presets: `happy-path`, `nuxt`, `opinionated`.

`html/no-empty-palpable-content` reports empty elements that should contain visible content. Default:
`warning`. Presets: `happy-path`, `nuxt`, `opinionated`.

`vapor/no-inline-template` disallows the deprecated `inline-template` attribute. Default: `error`.
Presets: `nuxt`, `opinionated`.

`vapor/prefer-static-class` prefers static `class` over dynamic bindings for string literals.
Default: `warning`. Presets: `nuxt`, `opinionated`.
