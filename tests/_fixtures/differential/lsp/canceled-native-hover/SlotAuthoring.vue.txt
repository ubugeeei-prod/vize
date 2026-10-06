<!-- @inferTemplateDollarSlots true -->
<script setup lang="ts">
import { useSlots } from "vue";

defineSlots<{
  /** **Header** shown above the invoice.
   * @param props Customer-visible heading.
   * @example header({ title: 'Summary' })
   */
  header(props: { title: string }): any;
}>();
const ownSlots = useSlots();
ownSlots.header({ title: "Summary" });

declare const Invoice: (
  props: {},
  context: {
    slots: {
      summary(props: {
        /** **Invoice** total shown to the customer.
         * @example total.toFixed(2)
         */
        total: number;
      }): void;
    };
  },
) => {};
const names = {
  /** **Primary** invoice slot shown in the summary. */
  current: "summary" as const,
};
</script>

<template>
  <slot name="header" title="Summary" />
  {{ $slots.header({ title: "Summary" }) }}
  <Invoice>
    <template #[names.current]="invoice">
      {{ invoice.total.toFixed(2) }}
    </template>
  </Invoice>
</template>
