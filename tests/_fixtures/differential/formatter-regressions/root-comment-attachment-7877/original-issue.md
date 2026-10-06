### Area

`vize fmt` (glyph), SFC root level

### Version

`vize` 0.432.0

### Minimal reproduction

`Example.vue`

```vue
<script setup lang="ts">
const label = "hello";
</script>

<!-- Explains the template below. -->
<template>
  <p>{{ label }}</p>
</template>
```

```sh
vize fmt --write Example.vue
```

### Actual

A blank line is inserted between the root-level comment and the block it documents:

```vue
<!-- Explains the template below. -->

<template>
  <p>{{ label }}</p>
</template>
```

### Expected

The comment stays attached to the following block, as written (Prettier 3 and Oxfmt keep it unchanged):

```vue
<!-- Explains the template below. -->
<template>
  <p>{{ label }}</p>
</template>
```
