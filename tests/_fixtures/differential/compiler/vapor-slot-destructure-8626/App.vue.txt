<script setup lang="ts" vapor>
import SlotOwner from "./SlotOwner.vue";
</script>

<template>
  <div>
    <SlotOwner v-slot="{ props: trigger }">
      <button :id="trigger.id" type="button">{{ trigger.label }}</button>
    </SlotOwner>
    <SlotOwner v-slot="{ props: { label } }">
      <span id="nested-label">{{ label }}</span>
    </SlotOwner>
  </div>
</template>
