<script setup lang="ts">
interface Greeting {
  greet(name: string): string;
}
class Welcome implements Greeting {}
</script>

<template><p>Quick fix</p></template>
