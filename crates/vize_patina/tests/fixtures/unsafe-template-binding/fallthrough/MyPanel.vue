<script setup lang="ts">
import { useId } from "vue";

import MyButton from "./MyButton.vue";

const emit = defineEmits<{ close: [] }>();
const id = useId();
const compact = true;
</script>

<template>
  <div class="panel">
    <label :for="`${id}-action`">Action</label>
    <MyButton :id="`${id}-action`" :class="{ 'is-compact': compact }" color="primary" @click="() => emit('close')">
      Close
    </MyButton>
  </div>
</template>
