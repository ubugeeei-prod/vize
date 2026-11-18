<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  emits: {
    save: (payload: { id: number }) => payload.id > 0,
    close: null,
    'update:modelValue': (value: string) => typeof value === 'string',
  },
  props: { modelValue: { type: String, default: '' } },
})
</script>

<template>
  <div>
    <button @click="$emit('save', { id: 1 })">ok</button>
    <button @click="$emit('save', { id: 'x' })">bad</button>
    <button @click="$emit('close')">close</button>
    <button @click="$emit('missing')">missing</button>
    <input :value="modelValue" @input="$emit('update:modelValue', 1)" />
  </div>
</template>
