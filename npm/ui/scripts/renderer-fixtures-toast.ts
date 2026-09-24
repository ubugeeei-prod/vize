export const toastRendererFixtures = [
  {
    filename: "ToastConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  ToastAction,
  ToastClose,
  ToastDescription,
  ToastProvider,
  ToastRoot,
  ToastTitle,
  Toaster,
  createToastStore,
} from "./families/feedback/toast/toast.ts";

const store = createToastStore<{ readonly orderId: number }>({ limit: 2 });
store.toast({ title: "Order placed", description: "Renderer toast", data: { orderId: 1 } });
</script>

<template>
  <ToastProvider :store label="Renderer notifications" :hotkey="['altKey', 'KeyT']">
    <Toaster v-slot="{ toast }">
      <ToastRoot :toast>
        <ToastTitle />
        <ToastDescription />
        <ToastAction alt-text="Open orders from the sidebar">View</ToastAction>
        <ToastClose />
      </ToastRoot>
    </Toaster>
  </ToastProvider>
</template>
`,
  },
] as const;
