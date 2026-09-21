<script setup lang="ts">
defineProps<{ row: { name: string; price: string } }>();
</script>

<template>
  <tr>
    <td>{{ row.name }}</td>
    <td>{{ row.price }}</td>
  </tr>
</template>
