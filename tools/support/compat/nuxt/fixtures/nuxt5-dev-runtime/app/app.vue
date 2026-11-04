<script setup>
const count = ref(0);
useHead({
  title: "Stylesheet fixture",
  meta: [{ name: "description", content: "Controlled SSR stylesheet hook fixture" }],
  link: [{ rel: "icon", href: "data:," }],
});
</script>
<template>
  <button class="styled-counter" @click="count++">{{ count }}</button>
</template>
<style scoped>
.styled-counter {
  outline-color: rgb(44, 55, 66);
}
</style>
