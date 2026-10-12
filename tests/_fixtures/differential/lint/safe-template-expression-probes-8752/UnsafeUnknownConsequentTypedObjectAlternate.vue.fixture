<script setup lang="ts">
const flag: boolean = Math.random() > 0.5;
const t = (p: {text: string}): string => p.text;
const value: unknown = "x";
</script>
<template>
<p>{{ flag
 ? value
 : t({text: 'safe'}) }}</p>
</template>
