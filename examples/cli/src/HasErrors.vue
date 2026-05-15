<script setup lang="ts">
import { ref } from "vue";

const items = ref([
  { id: 1, name: "Apple" },
  { id: 2, name: "Banana" },
  { id: 3, name: "Cherry" },
]);

const users = ref([
  { id: 1, name: "Alice", active: true },
  { id: 2, name: "Bob", active: false },
  { id: 3, name: "Charlie", active: true },
]);
</script>

<template>
  <div class="container">
    <!-- vue/require-v-for-key: v-for without :key -->
    <ul>
      <li v-for="item in items">{{ item.name }}</li>
    </ul>

    <!-- vue/no-use-v-if-with-v-for: v-if with v-for on same element -->
    <div v-for="user in users" v-if="user.active" :key="user.id">
      {{ user.name }}
    </div>

    <!-- Correct usage for comparison -->
    <template v-for="product in items" :key="product.id">
      <div>{{ product.name }}</div>
    </template>
  </div>
</template>

<style scoped>
.container {
  padding: 20px;
}

ul {
  list-style: none;
  padding: 0;
}

li {
  padding: 8px;
  border-bottom: 1px solid #eee;
}
</style>
