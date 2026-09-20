<script setup lang="ts">
import { defineComponent } from "vue";

interface Invoice {
  /** **Invoice total** before refunds.
   * @example
   * invoiceTotal.toFixed(2)
   */
  invoiceTotal: number;
  /** **Refund total** returned to the customer.
   * @example
   * refundTotal.toFixed(2)
   */
  refundTotal: number;
}
const invoice: Invoice = { invoiceTotal: 42, refundTotal: 5 };
const Native = defineComponent({ emits: { "row-picked": (_invoice: Invoice) => true } });
declare const Generic: new <T>(props: { value: T }) => {
  $props: { value: T };
  $emit: {
    (event: "rowPicked", value: T): void;
    (event: "reset"): void;
  };
};
</script>

<template>
  <Native @row-picked="$event.invoiceTotal.toFixed()" />
  <Generic :value="invoice" @row-picked="$event.refundTotal.toFixed()" />
</template>
