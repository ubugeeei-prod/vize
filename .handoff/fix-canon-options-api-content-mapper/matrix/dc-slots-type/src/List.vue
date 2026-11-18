<script lang="ts">
import { defineComponent, type PropType, type SlotsType } from 'vue'
export default defineComponent({
  props: { items: { type: Array as PropType<string[]>, required: true } },
  slots: Object as SlotsType<{ default: { item: string; index: number }; empty: {} }>,
})
</script>

<template>
  <ul>
    <li v-for="(item, index) in items" :key="index"><slot :item="item" :index="index" /></li>
    <slot v-if="!items.length" name="empty" />
  </ul>
</template>
