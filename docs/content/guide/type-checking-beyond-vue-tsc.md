---
title: Type Checking Beyond vue-tsc
description: Errors and editor answers where Vize is more precise than vue-tsc and Volar. Each one is pinned by a test, so this page matches what Vize reports.
---

# Type Checking Beyond vue-tsc

> We are NOT aiming for full vue-tsc compatibility. We are aiming for something more correct and more precise.

vue-tsc, `@vue/content-mapper` and TypeScript are references, not the specification. When
Vize and vue-tsc disagree, the authored code decides:

- If Vize is right, it keeps the behavior, and the case is listed on this page.
- If Vize is wrong, the difference is a bug, and it gets fixed.

Vize runs differential checks against vue-tsc on real projects such as misskey and n8n, but
only to find differences. Each difference is then classified from the code itself. Positions,
codes and messages are chosen for accuracy, for example the exact attribute rather than the
whole tag, not to copy vue-tsc.

Each example below is backed by a test that runs the native TypeScript compiler on these exact
files. The output shown is what Vize reports, in the `vize check` format. A TypeScript 7.1 or
tsrs run reports the same codes and positions in its own format. The vue-tsc output was
recorded with vue-tsc 3.3.11, TypeScript 6.0.3 and Vue 3.5.44, using default
`vueCompilerOptions`.

## Diagnostics

### A prop keeps its type when a sibling prop's type is missing

When one prop's type can't be resolved, for example because a package isn't installed,
vue-tsc's `toRef(props, …)` turns every prop into `any`. Vize keeps each prop's own type, so a
read the declared type doesn't allow is still an error:

```vue
<!-- src/NodePanel.vue -->
<script setup lang="ts">
import { toRef } from "vue";
import type { RunData } from "missing-run-data";

interface PanelNode {
  name: string;
}

const props = withDefaults(defineProps<{ node?: PanelNode | null; run?: RunData }>(), {
  node: null,
});
const node = toRef(props, "node");
</script>

<template>
  <span>{{ node?.label }}</span>
</template>
```

Vize reports the missing module and the read:

```text
src/NodePanel.vue
  error:3:30 [TS2307] Cannot find module 'missing-run-data' or its corresponding type declarations.
  error:16:18 [TS2339] Property 'label' does not exist on type 'PanelNode'.
```

vue-tsc reports only the missing module:

```text
src/NodePanel.vue(3,30): error TS2307: Cannot find module 'missing-run-data' or its corresponding type declarations.
```

`PanelNode` declares `name`, not `label`, and that doesn't depend on `RunData`. Reading
`node?.name` instead leaves only the missing-module error. Found in n8n's `RunData.vue`,
where 16 property reads were hidden this way. Pinned by
[`unresolved-prop-type`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/type-checking-beyond-vue-tsc/unresolved-prop-type).

### A listener on a native element receives a DOM `Event`

A `v-on` listener on a native element is a DOM event listener, whatever the event's name.
Vize types its argument as `Event`. vue-tsc leaves a name it doesn't know untyped:

```vue
<!-- src/Accordion.vue -->
<script setup lang="ts">
function beforeEnter(el: HTMLElement) {
  el.style.height = "0";
}
</script>

<template>
  <div class="accordion" @before-enter="beforeEnter">
    <slot />
  </div>
</template>
```

```text
src/Accordion.vue
  error:8:41 [TS2345] Argument of type '(el: HTMLElement) => void' is not assignable to parameter of type '(_e: Event) => unknown'.
    Types of parameters 'el' and '_e' are incompatible.
    Type 'Event' is missing the following properties from type 'HTMLElement': accessKey, accessKeyLabel, autocapitalize, autocorrect, and 318 more.
```

vue-tsc reports nothing, with or without `strictTemplates`.

`before-enter` is a `<Transition>` hook. On a `<div>` the handler never receives the element,
and it only runs if something dispatches a DOM event with that name. The code meant a
transition group:

```vue
<!-- src/Accordion.vue (fixed) -->
<script setup lang="ts">
function beforeEnter(el: Element) {
  (el as HTMLElement).style.height = "0";
}
</script>

<template>
  <TransitionGroup tag="div" class="accordion" @before-enter="beforeEnter">
    <slot />
  </TransitionGroup>
</template>
```

Found in n8n's `ItemsRenderer.vue`, which used four transition hooks on a `<div>`. Pinned by
[`unknown-native-event`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/type-checking-beyond-vue-tsc/unknown-native-event).

### A slot the component doesn't declare

Vize reports slot content passed to a slot the component doesn't declare. The check is on by
default, with no option to turn on:

```vue
<!-- src/EmptyState.vue -->
<script setup lang="ts">
defineSlots<{ icon(): unknown; default(): unknown; actions(): unknown }>();
</script>

<template>
  <article>
    <slot name="icon" />
    <p><slot /></p>
    <slot name="actions" />
  </article>
</template>
```

```vue
<!-- src/Panel.vue -->
<script setup lang="ts">
import EmptyState from "./EmptyState.vue";
</script>

<template>
  <EmptyState>
    <template #description>Connect a node first.</template>
  </EmptyState>
</template>
```

```text
src/Panel.vue
  error:7:16 [TS2339] Property 'description' does not exist on type 'Partial<$Slots>'.
```

vue-tsc reports nothing, with or without `strictTemplates`.

`EmptyState` never renders a `description` slot, so this text never appears on the page.
Passing it to the default slot (`#default`) reports nothing. Found in n8n's
`InputPanel.vue`. Pinned by
[`unknown-slot-name`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/type-checking-beyond-vue-tsc/unknown-slot-name).

## Editor

The [language server](./language-server.md) uses the same checker. These answers come from
recorded editor responses, which Vize's tests compare after every change. The Volar answers were
observed with Vue - Official (`@vue/language-server` 3.3.11 and `@vue/typescript-plugin`) on the
same files.

### Hover on a component tag shows what it accepts

Hovering a component tag shows the component's authored props, emits, slots and model:

```typescript
const ShortComponent: VueComponent &
{
  props: { label: string; count?: number };
  emits: { save: [value: string] };
  slots: { default(props: { value: string }): unknown };
  model: { "modelValue": boolean };
}
```

Long components put one member per line. The same holds through `defineAsyncComponent` and
plain aliases. Volar shows only the import alias (`import LongComponent`), and through an async
alias it shows a `DefineComponent<__VLS_Props, …>` that names a generated type instead of the
props. Pinned by
[`component-hover-readability`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/component-hover-readability)
and
[`component-alias-hover-10787`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/component-alias-hover-10787).

### Go to definition reaches a component registered under an alias

For an Options API component registered as `components: { "lottie": Lottie }`, go to definition
on `<lottie />` opens `lottie.vue`. Volar stops at the `"lottie": Lottie` registration key, and
its hover there is `any`. Pinned by
[`registered-lowercase-component-alias`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/registered-lowercase-component-alias).

### Slot-name completion lists only the component's slots

Right after `#` or `v-slot:` on a `<template>` inside a component, completion offers exactly the
slots the component declares, each with a snippet that destructures its props, such as
`#item="{ item }"`. Volar offers 500 items there, mostly HTML attributes and directives.
Pinned by the
[`template-slot-completion-9907`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/template-slot-completion-9907)
inputs and `nested_template_slot_completion_only_lists_component_slots`.

## Related

- [Type checking](./type-checking.md#type-checking-policy) for the policy and setup
- [Language server](./language-server.md)
- [Migration guide](./migration.md) for comparing output before switching from vue-tsc
