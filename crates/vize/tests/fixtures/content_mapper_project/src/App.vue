<script setup lang="ts">
import CallSignatureChild from "./CallSignatureChild.vue";
import Child from "./Child.vue";
import DefaultModelChild from "./DefaultModelChild.vue";
import GenericChild from "./GenericChild.vue";
import ModelChild from "./ModelChild.vue";
import RuntimeChild from "./RuntimeChild.vue";
import { increment } from "./model";

defineProps<{ count: number }>();
const defaultModelValue = 1;
const handleCancel = (reason: string) => reason;
const handleModelValue = (value: number) => value;
const handlePick = (value: string) => value;
const handleSaveItem = (id: string) => id;
const handleSubmit = (accepted: boolean) => accepted;
const handleTitle = (title: string) => title;
</script>

<template>
  <Child :count="increment(count)" @save="increment" @save-item="handleSaveItem" />
  <DefaultModelChild :model-value="defaultModelValue" @update:modelValue="handleModelValue" />
  <CallSignatureChild @submit="handleSubmit" />
  <GenericChild value="chosen" @pick="handlePick" />
  <ModelChild title="chosen" @update:title="handleTitle" />
  <RuntimeChild @cancel="handleCancel" />
</template>
