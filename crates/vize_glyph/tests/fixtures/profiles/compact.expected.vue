<template>
  <article role="status" aria-live="polite" class="notice" data-component="save-result">{{ message }}</article>
  <button
    type="button"
    class="notice__dismiss"
    aria-label="Dismiss the saved result notification"
    data-tracking-id="save-result-dismiss">
    Dismiss
  </button>
</template>

<script lang="ts" setup>
defineProps<{ message: string }>();
</script>
