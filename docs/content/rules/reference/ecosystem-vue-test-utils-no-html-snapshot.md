---
title: "ecosystem/vue-test-utils-no-html-snapshot"
---

# `ecosystem/vue-test-utils-no-html-snapshot`

Avoid snapshotting wrapper.html() in Vue Test Utils tests

Default severity: `warning`  
Presets: `ecosystem`  
Automatic fix: None; review the suggested change  
Applies to: JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form  
Options: No rule-specific options. Severity and preset selection are configurable.

## Configuration (Vite+)

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

```vue
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

## Good

```vue
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [All rules](../all.md)
