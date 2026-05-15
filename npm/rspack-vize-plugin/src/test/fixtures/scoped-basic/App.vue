<script setup lang="ts">
const msg: string = "scoped basic";
</script>

<template>
  <div class="container">
    <h1 class="title">{{ msg }}</h1>
    <p class="desc">Plain scoped CSS</p>
  </div>
</template>

<style scoped>
.container {
  padding: 1rem;
}

.title {
  color: royalblue;
}

.title:hover {
  color: darkblue;
}

.title::before {
  content: "» ";
}

h1,
h2,
.desc {
  margin: 0;
}
</style>
