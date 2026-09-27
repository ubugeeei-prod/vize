<!-- @inferTemplateDollarSlots true -->
<script setup lang="ts">
import { useSlots } from 'vue';
const slots = useSlots();
slots.single?.({ str: 'different', num: 2 });
</script>
<template>
  <slot name="single" str="str" :num="1" />
  {{ $slots.single?.({ str: 'different', num: 2 }) }}
</template>
