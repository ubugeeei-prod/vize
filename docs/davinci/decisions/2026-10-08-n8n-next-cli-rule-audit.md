# Latest n8n CLI rule audit (2026-10-08)

Source: `db6f2c09fe0b11f6660c8cda40a642ce5ba9e875`; actual Rust `Linter::lint_sfc` API.
All 51 requested rules are selected at error severity, including the exact three rule options. The independent ESLint config uses the explicit 51-row mapping with no preset or provider filtering. Both implementations were called twice and complete raw observations match within each provider.

Actual source probe binary SHA256: `bef69ae861119f69e4e30b888d8d76079521ddac684ee3577ad93f0beef07e82`.
Providers: ESLint 10.4.1; eslint-plugin-vue 10.9.2; vue-eslint-parser 10.4.1; TypeScript parser 8.65.0; Node 24.14.0.

## Ref factory identity correction

Track imported Vue ref-factory identity, aliases, namespaces, and lexical shadowing instead of matching arbitrary callee names. Preserve .value and genuine direct-import controls. The focused producer is tracked in [#8275](https://github.com/ubugeeei-prod/vize/issues/8275) and the [ref identity decision](./2026-10-08-n8n-ref-factory-identity.md). This audit preserves the original source-before observations.

### ref-import-alias

```vue
<script setup lang="ts">
import { ref as makeRef } from "vue";
const value = makeRef(0);
console.log(value + 1);
</script>
<template><div /></template>
```

Independent rules: vue/no-ref-as-operand
Actual Vize rules: none

### ref-namespace

```vue
<script setup lang="ts">
import * as Vue from "vue";
const value = Vue.ref(0);
console.log(value + 1);
</script>
<template><div /></template>
```

Independent rules: vue/no-ref-as-operand
Actual Vize rules: none

### ref-local-factory

```vue
<script setup lang="ts">
function ref(value: number) {
  return value;
}
const value = ref(0);
console.log(value + 1);
</script>
<template><div /></template>
```

Independent rules: none
Actual Vize rules: script/no-ref-as-operand

### ref-other-package

```vue
<script setup lang="ts">
import { ref } from "other";
const value = ref(0);
console.log(value + 1);
</script>
<template><div /></template>
```

Independent rules: none
Actual Vize rules: script/no-ref-as-operand

### ref-import-shadow

```vue
<script setup lang="ts">
import { ref } from "vue";
function calc(ref: (v: number) => number) {
  const value = ref(0);
  return value + 1;
}
</script>
<template><div /></template>
```

Independent rules: none
Actual Vize rules: script/no-ref-as-operand

### ref-import-direct

```vue
<script setup lang="ts">
import { ref } from "vue";
const value = ref(0);
console.log(value + 1);
</script>
<template><div /></template>
```

Independent rules: vue/no-ref-as-operand
Actual Vize rules: script/no-ref-as-operand

### ref-value-control

```vue
<script setup lang="ts">
import { ref } from "vue";
const value = ref(0);
console.log(value.value + 1);
</script>
<template><div /></template>
```

Independent rules: none
Actual Vize rules: none

### registration-draggable

```vue
<template><draggable /></template>
```

Independent rules: vue/no-undef-components
Actual Vize rules: vue/component-name-in-template-casing

### registration-nuxt-link

```vue
<template><nuxt-link /></template>
```

Independent rules: vue/no-undef-components
Actual Vize rules: none

### slot-modifier

```vue
<script setup lang="ts">
import MyWidget from "./MyWidget.vue";
const name = "header";
</script>
<template>
  <MyWidget><template #header.foo>ready</template></MyWidget>
</template>
```

Independent rules: vue/valid-v-slot
Actual Vize rules: none

### slot-dynamic-scope

```vue
<script setup lang="ts">
import MyWidget from "./MyWidget.vue";
const name = "header";
</script>
<template>
  <MyWidget
    ><template #[slot.name]="slot">{{ slot.name }}</template></MyWidget
  >
</template>
```

Independent rules: vue/valid-v-slot
Actual Vize rules: none

### slot-default-component-no-value

```vue
<script setup lang="ts">
import MyWidget from "./MyWidget.vue";
const name = "header";
</script>
<template><MyWidget v-slot>ready</MyWidget></template>
```

Independent rules: vue/valid-v-slot
Actual Vize rules: none

### no-child-comment

```vue
<script setup lang="ts">
const html = "";
</script>
<template>
  <div v-html="html"><!--comment--></div>
</template>
```

Independent rules: vue/no-v-html, vue/no-child-content
Actual Vize rules: vue/no-v-html

Official semantics: [no-ref-as-operand](https://eslint.vuejs.org/rules/no-ref-as-operand.html), [no-undef-components](https://eslint.vuejs.org/rules/no-undef-components), [valid-v-slot](https://eslint.vuejs.org/rules/valid-v-slot.html).

## Evidence and scope

The [retained corpus](../../../tests/_fixtures/differential/lint/ref-factory-identity-8275/) contains complete inputs, messages, fixes, suggestions, labels, counts, provider/source identities and repeat observations. `source-before.json` replays all 58 owned audit controls on an isolated target; the original 26 entire packets match the initial research. `source-after.json` and `independent.json` retain 37 focused ref-identity controls, including peer-reported scope boundaries. The normal Rust integration and live independent tooling tests rerun those controls on Actions. Both providers keep their full packets; only the independent recording filename is normalized.

This proves authored library API discrepancies. It does not claim native CLI execution, original n8n file semantic parity, Oxlint host adoption, full workspace adoption, merged fixes, or published release success. Requirement branch source was not copied, run, changed, commented on, or redistributed. The registered-only casing work remains a separate lane.
