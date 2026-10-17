<script setup lang="ts">
import InvoiceCard from "./InvoiceCard.vue";

const invoice = {
  /**
   * Format the total shown on an invoice.
   *
   * Supports **JPY** and **USD** currencies.
   * @param amount Amount before currency formatting.
   * @param currency Currency used on the invoice.
   * @returns A localized invoice total.
   * @example
   * invoice.formatTotal(1200, 'JPY')
   */
  formatTotal(amount: number, currency: "JPY" | "USD" = "JPY"): string {
    return amount.toFixed() + currency;
  },
};
</script>

<template>
  <InvoiceCard heading="April" tone="muted" />
  <p>{{ invoice.formatTotal(1200) }}</p>
</template>
