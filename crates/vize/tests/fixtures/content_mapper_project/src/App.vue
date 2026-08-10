<script setup lang="ts">
import CallSignatureChild from "./CallSignatureChild.vue";
import Child from "./Child.vue";
import ConditionalGenericChild from "./ConditionalGenericChild.vue";
import DefaultModelChild from "./DefaultModelChild.vue";
import DynamicGenericChild from "./DynamicGenericChild.vue";
import GenericChild from "./GenericChild.vue";
import ModelChild from "./ModelChild.vue";
import NestedGenericChild from "./NestedGenericChild.vue";
import RuntimeChild from "./RuntimeChild.vue";
import SlotGenericChild from "./SlotGenericChild.vue";
import SlotProvider from "./SlotProvider.vue";
import { increment } from "./model";

defineProps<{ count: number }>();
const conditionalValue: "conditional" | null = Math.random() > 0.5 ? "conditional" : null;
const defaultModelValue = 1;
const handleCancel = (reason: string) => reason;
const handleChoose = (value: "top-level") => value;
const handleConfirm = (value: "conditional") => value;
const handleModelValue = (value: number) => value;
const handlePick = (value: string) => value;
const handleSelect = (value: "nested") => value;
const handleSaveItem = (id: string) => id;
const handleSlotActivate = (value: "slot") => value;
const handleSubmit = (accepted: boolean) => accepted;
const handleTitle = (title: string) => title;
const nestedValues = ["nested"] as const;
const topLevelValue = "top-level" as const;
</script>

<template>
  <Child :count="increment(count)" @save="increment" @save-item="handleSaveItem" />
  <ConditionalGenericChild
    v-if="conditionalValue"
    :value="conditionalValue"
    @confirm="handleConfirm"
  />
  <DefaultModelChild :model-value="defaultModelValue" @update:modelValue="handleModelValue" />
  <DynamicGenericChild :value="topLevelValue" @choose="handleChoose" />
  <CallSignatureChild @submit="handleSubmit" />
  <GenericChild value="chosen" @pick="handlePick" />
  <ModelChild title="chosen" @update:title="handleTitle" />
  <NestedGenericChild
    v-for="value in nestedValues"
    :key="value"
    :value="value"
    @select="handleSelect"
  />
  <RuntimeChild @cancel="handleCancel" />
  <SlotProvider v-slot="{ value }">
    <SlotGenericChild :value="value" @activate="handleSlotActivate" />
  </SlotProvider>
</template>
