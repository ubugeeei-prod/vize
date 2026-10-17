<script setup lang="ts">
defineProps<{
  /** Heading displayed above the invoice total. */
  heading: string;
  /** **Visual emphasis** for overdue invoices. */
  tone?: "muted" | "strong";
}>();
</script>

<template>
  <h2>{{ heading }}</h2>
</template>
