---
title: "script/no-deprecated-events-api"
description: "Disallow the removed Vue 2 events API ($on / $off / $once)."
---

# `script/no-deprecated-events-api`

Disallow the removed Vue 2 events API ($on / $off / $once).

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The `$on`, `$once`, and `$off` calls use the instance event-bus methods removed in Vue 3.

```vue annotate="remove:4,5,6,7"
<script>
export default {
  mounted() {
    this.$on('event', handler)
    const vm = this
    vm.$once('event', handler)
    this.$off('event', handler)
  }
}
</script>
```

## Good

`$emit` remains valid, while event-bus subscription moves to the external emitter’s `on` method. The repair separates parent-directed emission from an external event bus.

```vue annotate="add:2,3,6,7"
<script>
import mitt from 'mitt'
const emitter = mitt()
export default {
  mounted() {
    this.$emit('event', payload)
    emitter.on('event', handler)
  }
}
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-events-api": "error"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L51)
