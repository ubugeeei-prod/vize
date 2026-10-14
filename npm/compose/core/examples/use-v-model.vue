<script setup lang="ts">
import { useVModel } from "@vizejs/composable/use-v-model";

const props = defineProps<{ modelValue?: string }>();
const emit = defineEmits<{ "update:modelValue": [value: string | undefined] }>();
const nickname = useVModel(props, "modelValue", emit, { passive: true, defaultValue: "Ada" });
</script>

<template>
  <div class="composable-example">
    <p>
      Write a component that works with <code>v-model</code> and also keeps its own value when no
      parent binds one.
    </p>
    <label>
      Nickname
      <input v-model="nickname" />
    </label>
    <output aria-live="polite">Hello, {{ nickname }}!</output>
  </div>
</template>
