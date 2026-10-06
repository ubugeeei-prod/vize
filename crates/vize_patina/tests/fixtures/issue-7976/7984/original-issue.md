> **Possibly by design — filed as a question.** this is the same reasoning #7173 accepted for pseudo-elements and `@media` ("there is no element to put `v-show` on"), applied to `:deep()` / `::v-deep` targets. Whether the rule should skip them is a design call.

## Area

Linter, `css/no-display-none` (`opinionated`)

## Version

`vize` 0.432.0

## Minimal reproduction

`MyInput.vue`

```vue
<template>
  <div class="my-input">
    <input aria-label="value" />
    <div class="my-input__messages">message</div>
  </div>
</template>
```

`MyField.vue`

```vue
<script setup lang="ts">
import MyInput from "./MyInput.vue";
</script>

<template>
  <div class="my-field">
    <MyInput />
  </div>
</template>

<style scoped>
/* MyInput renders its own message row; this wrapper hides it */
.my-field :deep(.my-input__messages) {
  display: none;
}
</style>
```

`vize.config.json`

```json
{ "linter": { "preset": "incremental", "rules": { "css/no-display-none": "warn" } } }
```

```sh
vize lint -f plain --help-level none MyField.vue
```

## Actual

```
Patina lint report: 1 warning in 1 file

MyField.vue
  MyField.vue:14:3 warning css/no-display-none Consider using v-show directive instead of display: none
```

## Expected (proposal)

No diagnostic when the rule's selector reaches into a child component (`:deep(…)`, `::v-deep`,
`>>>`, `/deep/`), or into slotted content (`:slotted(…)`). The element being hidden is rendered by
another component's template, so there is no element in this template to put `v-show` on, which
is the same situation as the pseudo-element case fixed in #7173.
