---
title: "vue/v-on-event-hyphenation"
description: "Enforce hyphenation of custom event names in v-on on components."
---

# `vue/v-on-event-hyphenation`

Enforce hyphenation of custom event names in v-on on components.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | Yes | [Rule options](../options.md) |

Applies to: Vue SFC templates and blocks, with script context where the rule requires it.

## Bad

The custom component listener uses `@myEvent` instead of a hyphenated event name.

```vue annotate="remove:2,3,4"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
<my-component @update:modelValue="handler" />
</template>
```

## Good

`@my-event` uses the required custom-event spelling. Native-element listeners and dynamic event arguments shown below are outside this check.

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-event-hyphenation": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [Rule options](../options.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L40)
