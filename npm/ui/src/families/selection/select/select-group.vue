<script setup lang="ts">
import { computed } from "vue";

import {
  deriveDeterministicId,
  useDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { selectContext, selectGroupContext } from "./select-context.ts";

const { id = undefined, ariaLabel = undefined } = defineProps<{
  /**
   * Consumer-owned group id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Accessible group name used instead of a `SelectLabel`.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** A `SelectLabel` followed by the group's options. */
  default(): unknown;
}>();

const context = selectContext.use();
const groupId = useDeterministicId({ id: () => id, hint: "select-group" });
const labelId = computed(() => deriveDeterministicId(groupId.value, "label"));

selectGroupContext.provide({ labelId });
</script>

<template>
  <div
    :id="groupId"
    role="group"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabel === undefined ? labelId : undefined"
    :data-vize-ui="`${context.partPrefix}-group`"
    part="group"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
