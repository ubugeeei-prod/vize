<script setup lang="ts">
import { ref } from "vue";

const items = ref([1, 2, 3]);
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item">
      {{ item.toUpperCase() }}
    </li>
  </ul>
  <button
    @click="
      () => {
        const x: string = 123;
        console.log(x);
      }
    "
  >
    Bad inline
  </button>
</template>
