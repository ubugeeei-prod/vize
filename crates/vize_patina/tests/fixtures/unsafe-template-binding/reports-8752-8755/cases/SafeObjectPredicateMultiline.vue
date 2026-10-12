<script setup lang="ts">
const isA = (o: {kind:string}): boolean => o.kind === "a";
</script>
<template>
<p v-if="
 isA({
 kind: 'a'
 })
">safe</p>
</template>
