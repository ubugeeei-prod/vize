## Summary

`css/no-display-none` suggests `v-show` for `display: none` on elements selected through `:deep()` and `:slotted()`. Those elements are rendered by a child component or passed in as slot content, so they are not in this component's template and there is nowhere to put `v-show`. Like the pseudo-element and `@media` cases fixed in #7173, these findings have no compliant form.

Since #7055 the rule also checks nested rules, so a `:deep()` nested under the component's root class is now reported as well.

## Reproduction

`DataCalendar.vue`

```vue
<template>
  <div class="calendar"><div class="calendar__week-numbers" /><div class="calendar__footer" /></div>
</template>
```

`ScheduleView.vue`

```vue
<script setup lang="ts">
import DataCalendar from "./DataCalendar.vue";
</script>

<template>
  <div class="schedule">
    <DataCalendar />
    <slot />
  </div>
</template>

<style scoped>
.schedule :deep(.calendar__week-numbers) {
  display: none;
}

.schedule {
  :deep(.calendar__footer) {
    display: none;
  }
}

:slotted(.hint) {
  display: none;
}
</style>
```

`vize.config.json`

```json
{ "linter": { "preset": "incremental", "rules": { "css/no-display-none": "warn" } } }
```

```sh
vize lint -f plain --help-level none ScheduleView.vue
```

```
ScheduleView.vue:14:3 warning css/no-display-none Consider using v-show directive instead of display: none
ScheduleView.vue:19:5 warning css/no-display-none Consider using v-show directive instead of display: none
ScheduleView.vue:24:3 warning css/no-display-none Consider using v-show directive instead of display: none
```

## Expected

No diagnostics. When the rule's subject is reached through `:deep()`, `:slotted()` (or `:global()`), the element is owned by another component, and `v-show` in this template cannot reach it.

## Why it matters

Hiding part of a third-party or shared child component with `:deep()` is the usual (and often the only) way to do it. With nested rules now checked, every such override is reported, and the only option is turning the rule off per file.

## Environment

- vize 0.432.0
- vue 3.5.42
- node 26.8.1, macOS arm64
