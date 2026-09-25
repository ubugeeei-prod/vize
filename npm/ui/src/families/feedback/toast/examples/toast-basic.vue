<!-- Toast provider with a shared store that queues notifications into the default viewport. -->
<script setup lang="ts">
import { createToastStore, ToastProvider, ToastViewport } from "../toast.ts";

const store = createToastStore({ duration: 4000 });

function saveProfile(): void {
  store.success({ title: "Profile saved" });
}

function archiveMessage(): void {
  store.toast({
    title: "Message archived",
    description: "You can find it in the Archive folder.",
    action: { label: "Undo", altText: "Restore the message from the Archive folder" },
  });
}
</script>

<template>
  <ToastProvider :store label="Notifications">
    <button type="button" @click="saveProfile">Save profile</button>
    <button type="button" @click="archiveMessage">Archive message</button>
    <ToastViewport />
  </ToastProvider>
</template>
