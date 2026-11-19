<script setup lang="ts">
const title = defineModel<string>("title", { required: true })
const modelValue = defineModel<number>({ default: 0 })
</script>
<template>
  <Field
    v-model:title="title"
    v-model="modelValue"
    @update:title="() => undefined"
    @update:modelValue="() => undefined"
  />
  <output>{{ title }} {{ modelValue }}</output>
</template>
