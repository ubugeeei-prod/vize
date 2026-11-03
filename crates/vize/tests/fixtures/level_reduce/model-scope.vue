<script setup>
const items = []
const query = ""
</script>

<template>
  <form class="list">
    <h1>Items</h1>
    <input v-for="item in items" :key="item" v-model="item">
    <input v-model:value="query">
  </form>
</template>
