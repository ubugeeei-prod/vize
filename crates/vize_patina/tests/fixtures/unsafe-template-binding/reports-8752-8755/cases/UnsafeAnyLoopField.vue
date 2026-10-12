<script setup lang="ts">
const rows = [] as {label:any}[];
</script>
<template>
<p v-for="item in rows" :title="
 item.label
" />
</template>
