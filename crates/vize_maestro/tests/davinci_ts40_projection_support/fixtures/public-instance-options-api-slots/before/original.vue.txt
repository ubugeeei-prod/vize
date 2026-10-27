<script lang="ts">
export default {
  props: { message: { type: String, required: true } },
}
</script>
<template>
  <slot name="header" :message="message" />
  <p>{{ message }}</p>
</template>
