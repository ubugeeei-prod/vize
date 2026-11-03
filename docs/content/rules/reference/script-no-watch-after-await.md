---
title: "script/no-watch-after-await"
description: "Disallow watchers registered after an await in setup() without keeping their stop handle."
---

# `script/no-watch-after-await`

Disallow watchers registered after an await in setup() without keeping their stop handle.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

`watch()` is created after `await loadData()` and its stop handle is discarded, so the watcher is never stopped with the component.

```vue annotate="remove:5"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    await loadData()
    watch(() => props.id, reload)
  },
}
</script>
```

## Good

The watcher is created before the first `await`, so Vue stops it when the component unmounts.

```vue annotate="add:6"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    watch(() => props.id, reload)
    await loadData()
  },
}
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
        "script/no-watch-after-await": "error"
      }
    },
  },
});
```

Then run `vp run lint`.

## Related

- [Type and script rules](../type-and-script.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_watch_after_await.rs#L44)
