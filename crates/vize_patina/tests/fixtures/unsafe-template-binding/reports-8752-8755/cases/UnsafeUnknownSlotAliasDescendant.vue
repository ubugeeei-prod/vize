<script setup lang="ts">
const Child = {} as {new(): {$props: {}; $slots: {default: (p: {value: unknown}) => unknown}}};
</script>
<template>
<Child v-slot="{value: renamed}"><p :title="renamed" /></Child>
</template>
