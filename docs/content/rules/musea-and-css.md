---
title: Musea & CSS Rules
---

# Musea & CSS Rules

Musea rules validate `<art>` and `<variant>` blocks. CSS rules inspect style content and recommend
patterns that keep component styles themeable, predictable, and compatible with Vue and Vapor.

## `musea/require-title`

Requires every `<art>` block to have a `title`.

Default severity: `error`

Bad:

```vue
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## `musea/require-component`

Requires every `<art>` block to name the component it documents.

Default severity: `warning`

Bad:

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## `musea/valid-variant`

Requires `<variant>` blocks to have a valid `name`.

Default severity: `error`

Bad:

```vue
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## `musea/unique-variant-names`

Requires variant names to be unique inside one art block.

Default severity: `error`

Bad:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

## `musea/no-empty-variant`

Reports empty variants that do not document props, slots, or visual state.

Default severity: `warning`

Bad:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

## `musea/prefer-design-tokens`

Prefers design token CSS variables over hardcoded primitive values in Musea examples.

Default severity: `warning`

Bad:

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button style="color: #d00">Delete</Button>
  </variant>
</art>
```

Good:

```vue
<art title="Button" component="./Button.vue">
  <variant name="danger">
    <Button class="danger">Delete</Button>
  </variant>
</art>

<style scoped>
.danger {
  color: var(--color-danger-text);
}
</style>
```

## `css/no-important`

Discourages `!important`.

Default severity: `warning`

Bad:

```vue
<style scoped>
.button {
  color: red !important;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

## `css/no-hardcoded-values`

Suggests CSS variables instead of hardcoded color, spacing, or size values.

Default severity: `warning`

Bad:

```vue
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

Good:

```vue
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

## `css/no-id-selectors`

Discourages ID selectors in component styles because they are hard to override and reuse.

Default severity: `warning`

Bad:

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

Good:

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

## `css/no-display-none`

Suggests using Vue visibility primitives instead of hiding component branches with CSS.

Default severity: `warning`

Bad:

```vue
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

Good:

```vue
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

## `css/no-v-bind-performance`

Warns about the runtime cost of CSS `v-bind()` in hot styles.

Default severity: `warning`

Bad:

```vue
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

Good:

```vue
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## `css/prefer-logical-properties`

Recommends logical properties for internationalized layouts.

Default severity: `warning`

Bad:

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

Good:

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

## `css/prefer-slotted`

Recommends `::v-slotted()` when styling slot content.

Default severity: `warning`

Bad:

```vue
<style scoped>
.content h2 {
  margin-block: 0;
}
</style>
```

Good:

```vue
<style scoped>
::v-slotted(h2) {
  margin-block: 0;
}
</style>
```

## `css/require-font-display`

Requires `font-display` in `@font-face` declarations.

Default severity: `warning`

Bad:

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

Good:

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

## Additional CSS Rules

`css/no-utility-classes` warns against implementing utility classes inside component styles. Default:
`warning`.

`css/prefer-nested-selectors` recommends CSS nesting for descendant selectors. Default: `warning`.
