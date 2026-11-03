<script setup lang="tsx">
const count: number = 1
const vnode = <span>{count.toFixed(0)}</span>
</script>
<template><div :data-count="count">{{ vnode }}</div></template>
