// CrossFile preset: Fallthrough Attrs
// $attrs, useAttrs(), and inheritAttrs patterns

import type { Preset } from "./crossfile";

export const FALLTHROUGH_ATTRS_PRESET: Omit<Preset, "icon"> = {
  id: "fallthrough-attrs",
  name: "Fallthrough Attrs",
  description: "$attrs, useAttrs(), and inheritAttrs patterns",
  files: {
    "App.vue": `<script setup lang="ts">
import BaseButton from './BaseButton.vue'
import MultiRootComponent from './MultiRootComponent.vue'
import UseAttrsComponent from './UseAttrsComponent.vue'
</script>

<template>
  <div>
    <h1>Fallthrough Attributes</h1>

    <!-- Passing class, style, and event to child -->
    <BaseButton
      class="custom-class"
      style="color: red"
      data-testid="main-button"
      @click="console.log('clicked')"
    >
      Click me
    </BaseButton>

    <!-- Multi-root needs explicit $attrs binding -->
    <MultiRootComponent
      class="passed-class"
      aria-label="Multiple roots"
    />

    <!-- Component using useAttrs() -->
    <UseAttrsComponent
      class="attrs-class"
      custom-attr="value"
    />
  </div>
</template>`,

    "BaseButton.vue": `<script setup lang="ts">
// Single root element - $attrs automatically applied

defineProps<{
  variant?: 'primary' | 'secondary'
}>()
</script>

<template>
  <!-- \u2713 $attrs (class, style, listeners) auto-applied to single root -->
  <button class="base-button">
    <slot />
  </button>
</template>`,

    "MultiRootComponent.vue": `<script setup lang="ts">
// \u274C Multiple root elements - $attrs not auto-applied!
// Need to explicitly bind $attrs to intended element
</script>

<template>
  <!-- \u274C Which element gets class="passed-class"? Neither! -->
  <header class="header">
    Header content
  </header>
  <main class="main">
    Main content
  </main>
  <footer class="footer">
    Footer content
  </footer>
</template>`,

    "MultiRootFixed.vue": `<script setup lang="ts">
// \u2713 Multiple roots with explicit $attrs binding
</script>

<template>
  <header class="header">
    Header content
  </header>
  <!-- \u2713 Explicitly bind $attrs to main element -->
  <main v-bind="$attrs" class="main">
    Main content
  </main>
  <footer class="footer">
    Footer content
  </footer>
</template>`,

    "UseAttrsComponent.vue": `<script setup lang="ts">
import { useAttrs, computed } from 'vue'

// \u2713 useAttrs() for programmatic access
const attrs = useAttrs()

// Access specific attributes
const customAttr = computed(() => attrs['custom-attr'])

// \u274C useAttrs() called but attrs not bound in template!
// This means passed attributes are lost
</script>

<template>
  <div>
    <p>Custom attr value: {{ customAttr }}</p>
    <!-- \u274C attrs not bound - class="attrs-class" is lost! -->
  </div>
</template>`,

    "UseAttrsFixed.vue": `<script setup lang="ts">
import { useAttrs, computed } from 'vue'

const attrs = useAttrs()
const customAttr = computed(() => attrs['custom-attr'])

// \u2713 Can filter/transform attrs
const filteredAttrs = computed(() => {
  const { class: _, ...rest } = attrs
  return rest
})
</script>

<template>
  <!-- \u2713 Explicitly bind attrs -->
  <div v-bind="attrs">
    <p>Custom attr: {{ customAttr }}</p>
  </div>
</template>`,

    "InheritAttrsFalse.vue": `<script setup lang="ts">
// \u274C inheritAttrs: false but $attrs not used!
// Passed attributes are completely lost

defineOptions({
  inheritAttrs: false
})
</script>

<template>
  <div class="wrapper">
    <input type="text" />
    <!-- $attrs should be bound to input, not wrapper -->
  </div>
</template>`,

    "InheritAttrsFixed.vue": `<script setup lang="ts">
// \u2713 inheritAttrs: false with explicit $attrs binding

defineOptions({
  inheritAttrs: false
})
</script>

<template>
  <div class="wrapper">
    <!-- \u2713 Bind $attrs to the actual input -->
    <input v-bind="$attrs" type="text" />
  </div>
</template>`,
  },
};
