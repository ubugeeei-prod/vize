<script setup lang="ts">
const get: any = (p: {kind:string}) => p.kind;
</script>
<template>
<p v-if="get({kind: 'a'})">unsafe</p>
</template>
