---
title: Template Rule Options
---

# Template Rule Options

These options use the names and defaults of
[eslint-plugin-vue](https://eslint.vuejs.org/rules/). Options alone do not enable a rule. A later
matching entry replaces the rule's full option object, and `{}` selects the defaults again. Unknown
fields and values of the wrong type are rejected.

## `vue/no-duplicate-attributes`

`allowCoexistClass` and `allowCoexistStyle` both default to `true`, so `class` may sit next to
`:class` and `style` next to `:style`. Set one to `false` to report the second spelling as a
duplicate. Two static `class` attributes are always duplicates.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-duplicate-attributes": "error" },
    ruleOptions: { "vue/no-duplicate-attributes": { allowCoexistClass: false } },
  },
});
```

Bad with this config:

```vue
<template>
  <div class="card" :class="{ active }" />
</template>
```

Good with this config:

```vue
<template>
  <div :class="['card', { active }]" />
</template>
```

## `vue/no-multiple-template-root`

`disallowComments` defaults to `false`. With `true`, a comment placed directly in the template
root, outside every root element, is reported too. Comments inside the root element stay allowed.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-multiple-template-root": "error" },
    ruleOptions: { "vue/no-multiple-template-root": { disallowComments: true } },
  },
});
```

Bad with this config:

```vue
<template>
  <!-- The card -->
  <div class="card" />
</template>
```

Good with this config:

```vue
<template>
  <div class="card">
    <!-- The card -->
  </div>
</template>
```

## `vue/no-lone-template`

`ignoreAccessible` defaults to `false`. With `true`, a nested `<template>` that has an `id` or a
`ref`, static or bound with a static argument, is not reported, because code can reach it.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-lone-template": "warn" },
    ruleOptions: { "vue/no-lone-template": { ignoreAccessible: true } },
  },
});
```

Bad with this config:

```vue
<template>
  <div>
    <template><span>Row</span></template>
  </div>
</template>
```

Good with this config:

```vue
<template>
  <div>
    <template ref="row"><span>Row</span></template>
  </div>
</template>
```

## `vue/no-static-inline-styles`

`allowBinding` defaults to `false`, which reports static values inside a `:style` binding as well
as the static `style` attribute. With `true`, only the `style` attribute is reported.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-static-inline-styles": "warn" },
    ruleOptions: { "vue/no-static-inline-styles": { allowBinding: true } },
  },
});
```

Bad with this config:

```vue
<template>
  <div style="color: red" />
</template>
```

Good with this config:

```vue
<template>
  <div :style="{ color: 'red' }" />
</template>
```

## `vue/no-unused-components`

`ignoreWhenBindingPresent` defaults to `true`: a template that binds `is` to anything but a string
literal, such as `<component :is="current">`, can render any registered component, so the file
reports nothing. With `false`, registrations are checked as if the binding were absent. A template
that HTML recovery could not parse is still treated as possibly using every registration.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-unused-components": "warn" },
    ruleOptions: { "vue/no-unused-components": { ignoreWhenBindingPresent: false } },
  },
});
```

Bad with this config:

```vue
<script>
import Card from "./Card.vue";
import Panel from "./Panel.vue";

export default { components: { Card, Panel }, data: () => ({ current: "Card" }) };
</script>

<template>
  <component :is="current" />
</template>
```

Good with this config:

```vue
<script>
import Card from "./Card.vue";

export default { components: { Card } };
</script>

<template>
  <Card />
</template>
```

## `vue/no-v-html`

`ignorePattern` has no default. A `v-html` whose expression matches the regular expression is not
reported. An identifier is matched by its name; any other expression by its source text without
enclosing parentheses. The pattern uses Rust regex syntax, which is Unicode-aware like the `u` flag
upstream applies. An empty pattern ignores nothing, and a pattern that does not compile ignores
nothing. The pattern applies to `v-html` only; bound `innerHTML` and `outerHTML` are still reported.

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/no-v-html": "warn" },
    ruleOptions: { "vue/no-v-html": { ignorePattern: "^sanitized" } },
  },
});
```

Bad with this config:

```vue
<template>
  <div v-html="article.body" />
</template>
```

Good with this config:

```vue
<template>
  <div v-html="sanitizedBody" />
</template>
```
