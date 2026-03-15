<script setup lang="ts">
const html = "<strong>Hello</strong>";
const items = ["a", "b", "c"];
console.log(html);
</script>

<template>
  <section>
    <div v-html="html" />
    <ul>
      <li v-for="item in items">{{ item }}</li>
    </ul>
    <button class="primary" class="duplicate">Click</button>
  </section>
</template>
