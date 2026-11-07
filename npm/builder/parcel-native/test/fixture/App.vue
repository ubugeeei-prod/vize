<script setup lang="ts">
const message: string = "from-native-parcel";
</script>

<template>
  <main class="message">
    <img src="./logo.svg" alt="Vize" />
    {{ message }}
  </main>
</template>

<style scoped>
.message {
  color: rebeccapurple;
}
</style>
