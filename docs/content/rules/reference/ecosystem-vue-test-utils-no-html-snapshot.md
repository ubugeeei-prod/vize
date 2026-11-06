---
title: "ecosystem/vue-test-utils-no-html-snapshot"
description: "Avoid snapshotting wrapper.html() in Vue Test Utils tests."
---

# `ecosystem/vue-test-utils-no-html-snapshot`

Avoid snapshotting wrapper.html() in Vue Test Utils tests.

| Default severity | Presets | Automatic fix | Options |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | No | None |

Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form.

## Bad

The assertion snapshots the complete wrapper HTML instead of checking the expected behavior.

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

## Good

The assertion checks that the rendered text contains Saved.

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/vue-test-utils-no-html-snapshot": "warn"
    }
  },
});
```

Then run `vp run lint`.

## Related

- [Ecosystem and Nuxt rules](../ecosystem.md)
- [All rules](../all.md)
- [Source code](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15)
