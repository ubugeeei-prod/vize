<script setup lang="ts">
const Child = {} as {new(): {$props: {}; $slots: {default: (p: {value: any}) => unknown}}};
</script>
<template>
<Child v-slot="{value}"><p :title="value" /></Child>
</template>
