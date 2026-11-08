<script setup lang="ts">
import { computed } from "vue";

import { useIsScreenReaderEnabled } from "../composables/useIsScreenReaderEnabled.js";
import { stringifyChildren } from "../utils/text.js";
import type { TransformProps } from "./text-utility-types.js";

const props = defineProps<TransformProps>();
const slots = defineSlots<{ default?: () => unknown }>();

defineOptions({ name: "Transform" });

const isScreenReaderEnabled = useIsScreenReaderEnabled();

const content = computed(() => {
  const text = stringifyChildren(slots.default?.());
  return isScreenReaderEnabled && props.accessibilityLabel
    ? props.accessibilityLabel
    : text
        .split("\n")
        .map((line, index) => props.transform(line, index))
        .join("\n");
});
</script>

<template>
  <text :text="content" :aria-label="accessibilityLabel" />
</template>
