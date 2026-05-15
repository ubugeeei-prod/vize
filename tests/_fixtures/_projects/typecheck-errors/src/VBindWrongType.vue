<script setup lang="ts">
import { ref } from "vue";

const count = ref(42);
const items = ref([1, 2, 3]);
</script>

<template>
  <div>
    <input :value="count.toUpperCase()" />
    <ul :class="items.split('-')">
      <li>{{ count }}</li>
    </ul>
  </div>
</template>
