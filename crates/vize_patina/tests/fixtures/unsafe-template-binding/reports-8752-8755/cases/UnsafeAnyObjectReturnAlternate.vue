<script setup lang="ts">
const flag: boolean = Math.random() > 0.5;
const text = "safe";
const t = (p: {text: string}): any => p.text;
</script>
<template>
<p>{{ flag
 ? text
 : t({text: 'unsafe'}) }}</p>
</template>
