<script setup lang="ts">
defineEmits({
  cancel: (reason: string) => reason.length > 0,
});
</script>

<template>
  <button type="button">Cancel</button>
</template>
