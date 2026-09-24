<!-- Vertical stack of form actions nested with an inline row of buttons. -->
<script setup lang="ts">
import { Stack } from "../stack.ts";
</script>

<template>
  <Stack as="section" gap="0.75rem">
    <h2>Account settings</h2>
    <p>Update how your profile appears to other members.</p>
    <Stack axis="inline" gap="0.5rem" justify="end" align="center">
      <button type="button">Cancel</button>
      <button type="submit">Save changes</button>
    </Stack>
  </Stack>
</template>
