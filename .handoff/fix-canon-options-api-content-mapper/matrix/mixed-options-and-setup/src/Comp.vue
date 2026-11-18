<script lang="ts">
export default {
  name: 'Mixed',
  inheritAttrs: false,
}
</script>

<script setup lang="ts">
const props = defineProps<{ label: string; count?: number }>()
const emit = defineEmits<{ pick: [n: number] }>()
</script>

<template>
  <div v-bind="$attrs" @click="emit('pick', 1)">
    {{ label.toUpperCase() }} {{ props.count?.toFixed() }} {{ $attrs.id }}
    {{ label.toFixed() }} {{ count.toFixed() }}
  </div>
</template>
