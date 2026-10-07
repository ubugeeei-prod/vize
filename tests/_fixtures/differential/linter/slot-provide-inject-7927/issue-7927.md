## Summary

`vize lint --cross-file` again treats a component passed through a slot as unrelated to the component that renders the slot. The `inject()` is reported as unmatched and the `provide()` as unused, even though at runtime the injected component is a descendant of the provider.

This looks like a regression of #869: the exact reproduction from that issue fails again on 0.432.0.

## Reproduction

`src/Provider.vue`:

```vue
<script setup lang="ts">
import { provide, ref } from 'vue'

const count = ref(0)
provide('count', count)
</script>

<template>
  <slot />
</template>
```

`src/Consumer.vue`:

```vue
<script setup lang="ts">
import { inject } from 'vue'

const count = inject('count')
</script>

<template>
  <div>{{ count }}</div>
</template>
```

`src/App.vue`:

```vue
<script setup lang="ts">
import Consumer from './Consumer.vue'
import Provider from './Provider.vue'
</script>

<template>
  <Provider>
    <Consumer />
  </Provider>
</template>
```

```sh
vize lint --cross-file --cross-file-tree --format plain src
```

## Actual (0.432.0)

```text
src/Consumer.vue:4:15 error cross-file vize:croquis/cf/unmatched-inject: **Unmatched Inject**: `inject('count')` has no matching `provide()` in any ancestor component and returns `undefined` at runtime.
src/Provider.vue:5:1 warning cross-file vize:croquis/cf/unused-provide: ...
```

The same happens with an `InjectionKey` imported from another module, with `inject(KEY, null)` (reported as "Unmatched Inject Default"), and when the `<slot />` is wrapped in an element (`<div><slot /></div>`). The printed tree puts the slot content directly under `App` and the provider at the root:

```text
## Provide/Inject Tree
**Provider**
  provide("count") _unused_
**App**
  **Consumer**
    inject("count") _no provider_
```

## Expected

`Consumer` is a slot descendant of `Provider`, so `inject('count')` matches `provide('count', count)` and neither diagnostic is reported (as after #869).

## Environment

- vize / @vizejs/native 0.432.0
- vue 3.5.38
- macOS arm64, Node 26
