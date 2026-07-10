<script setup lang="ts">
import { ref } from "vue";

const textColor = ref("royalblue");
const bgColor = ref("#f0f0f0");
const fontSize = ref("16px");
</script>

<template>
  <div class="dynamic">
    <p>v-bind in CSS</p>
  </div>
</template>

<style scoped>
.dynamic {
  color: v-bind(textColor);
  background-color: v-bind(bgColor);
  font-size: v-bind(fontSize);
}
</style>
