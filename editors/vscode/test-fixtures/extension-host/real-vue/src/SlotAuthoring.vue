<script setup lang="ts">
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
  <Invoice>
    <template #[names.current]="invoice">
      {{ invoice.total.toFixed(2) }}
    </template>
  </Invoice>
</template>
