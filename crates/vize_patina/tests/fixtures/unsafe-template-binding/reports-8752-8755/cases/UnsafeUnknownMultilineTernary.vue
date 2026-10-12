<script setup lang="ts">
const flag = true; const text = "x"; const value: unknown = "y";
</script>
<template>
<p>{{ flag
 ? text
 : value }}</p>
</template>
