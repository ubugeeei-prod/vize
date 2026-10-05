<template>
  <div v-pre>
    <span>{{ this is not an expression }}</span>
    <span :class="notEvaluated">RAW_VPRE_TOKEN</span>
  </div>
  <p>{{ realBinding }}</p>
</template>

<script setup>
const realBinding = "ok";
</script>
