---
title: "script/prefer-ref-over-reactive"
---

# `script/prefer-ref-over-reactive`

Recommend using ref() over reactive() for state management

[Bad](#bad) · [Good](#good)

Default severity: `warning`  
Presets: _none_  
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
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## Bad

The state is created with `reactive`, contrary to this opinionated rule’s preference for refs. The example illustrates a style preference, not an inherently invalid reactive object.

```vue
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
count: 0,
name: 'foo'
})
</script>
```

## Good

The examples create both scalar and object state with `ref`; related fields may also be split into separate refs. This satisfies the preferred state-construction form.

```vue
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.

[Implementation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [All rules](../all.md)
