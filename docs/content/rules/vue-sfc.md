---
title: "Vue Rules: SFC Blocks"
---

# Vue Rules: SFC Blocks

Block order, supported languages, external sources, and style scope.
See [all Vue rules](./vue.md) for the complete reference and [Vue Rule Options](./options-vue.md)
for configurable settings.

## `vue/no-preprocessor-lang`

Reports `lang="sass"`, `lang="scss"`, `lang="less"`, `lang="stylus"`, and
`lang="styl"` on a `<style>` block.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<style lang="scss">
.button {
  color: red;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

## `vue/no-script-non-standard-lang`

Reports a `<script>` language other than `js`, `jsx`, `ts`, `tsx`, or
`typescript`, matched without regard to case. Omitting `lang` is JavaScript
and is allowed.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<script lang="coffee">
# CoffeeScript
</script>
```

Good:

```vue
<script setup lang="ts">
const label = "Save";
</script>
```

## `vue/no-src-attribute`

Reports `src` on the SFC's `<template>`, `<script>`, or `<style>` block.
The component source should live in the `.vue` file.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

Good:

```vue
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

## `vue/no-template-lang`

Reports `lang="pug"`, `lang="jade"`, `lang="slm"`, or `lang="haml"` on the
`<template>` block.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<template lang="pug">
div.container
  h1 Hello
</template>
```

Good:

```vue
<template>
  <div class="container">
    <h1>Hello</h1>
  </div>
</template>
```

## `vue/require-scoped-style`

Reports a `<style>` block that is neither `scoped` nor `module`.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<style>
.button {
  color: red;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

```vue
<style module>
.button {
  color: red;
}
</style>
```

## `vue/sfc-element-order`

Enforces `vue/block-order`'s default: `<script>` and `<template>` may come in
either order, and `<style>` is last.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

Good:

```vue
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

```vue
<template>
  <p>{{ label }}</p>
</template>

<script setup lang="ts">
const label = "Save";
</script>

<style scoped></style>
```

## `vue/single-style-block`

Reports more than one `<style>` block when those blocks have the same
purpose. One `scoped` block and one unscoped block are allowed.

Default severity: `warning`\
Presets: `happy-path`, `nuxt`, `opinionated`

Bad:

```vue
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

Good:

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

## `vue/warn-custom-block`

Reports a top-level SFC block other than `<script>`, `<template>`, and
`<style>`. An element inside `<template>` is not a custom block. The finding
is a warning that the block needs a plugin, not a parse error.

Default severity: `warning`\
Presets: `nuxt`, `opinionated`

Bad:

```vue
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

Good:

```vue
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```
