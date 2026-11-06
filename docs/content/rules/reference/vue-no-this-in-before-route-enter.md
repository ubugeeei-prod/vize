---
title: "vue/no-this-in-before-route-enter"
description: "Disallow `this` in `beforeRouteEnter`."
---

# `vue/no-this-in-before-route-enter`

Disallow `this` in `beforeRouteEnter`.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`beforeRouteEnter` runs before the component instance is available, so `this.load()` cannot read that instance.

```vue annotate="remove:4,5"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    this.load();
    next();
  },
  methods: { load() {} },
};
</script>
```

## Good

Use the instance passed to the `next` callback and call `vm.load()` after the component is created.

```vue annotate="add:4"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    next((vm) => vm.load());
  },
  methods: { load() {} },
};
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-this-in-before-route-enter": "warn"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Vue rules](../vue.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_this_in_before_route_enter.rs#L24)
