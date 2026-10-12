<script setup lang="ts">
const Child = {} as {new(): {$props: {}; $slots: {default: (p: {value: string}) => unknown}}};
</script>
<template>
<Child v-slot="{value}"><p :title="
 value
">safe</p></Child>
</template>
