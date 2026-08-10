<script setup lang="ts">
import CallSignatureChild from "./CallSignatureChild.vue";
import Child from "./Child.vue";
import RuntimeChild from "./RuntimeChild.vue";
import { increment } from "./model";

defineProps<{ count: number }>();
const handleCancel = (reason: string) => reason;
const handleSaveItem = (id: string) => id;
const handleSubmit = (accepted: boolean) => accepted;
</script>

<template>
  <Child :count="increment(count)" @save="increment" @save-item="handleSaveItem" />
  <CallSignatureChild @submit="handleSubmit" />
  <RuntimeChild @cancel="handleCancel" />
</template>
