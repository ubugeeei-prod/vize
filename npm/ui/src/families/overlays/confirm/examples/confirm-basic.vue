<!-- Awaits a destructive confirmation through the provider's promise-based confirm API. -->
<script setup lang="ts">
import { ref, useTemplateRef } from "vue";

import { ConfirmProvider, type ConfirmProviderExpose } from "../confirm.ts";

const confirmRef = useTemplateRef<ConfirmProviderExpose>("confirm");
const files = ref(["invoice-march.pdf", "invoice-april.pdf"]);

async function deleteAll(): Promise<void> {
  const provider = confirmRef.value;
  if (provider === null) return;
  const accepted = await provider.confirm({
    title: "Delete all invoices?",
    description: "This permanently removes every invoice in this folder.",
    confirmLabel: "Delete invoices",
    destructive: true,
  });
  if (accepted) files.value = [];
}
</script>

<template>
  <ConfirmProvider ref="confirm">
    <ul>
      <li v-for="file in files" :key="file">{{ file }}</li>
    </ul>
    <button type="button" :disabled="files.length === 0" @click="deleteAll">Delete all</button>
  </ConfirmProvider>
</template>
