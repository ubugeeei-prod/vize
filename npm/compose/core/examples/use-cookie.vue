<script setup lang="ts">
import { useCookie } from "@vizejs/composable/use-cookie";

const { state: theme, remove } = useCookie("docs-example-theme", { path: "/", sameSite: "lax" });
</script>

<template>
  <div class="composable-example">
    <p>Remember a preference in a cookie so the server can read it on the next request.</p>
    <div class="example-actions">
      <button type="button" @click="() => (theme = 'light')">Use light theme</button>
      <button type="button" @click="() => (theme = 'dark')">Use dark theme</button>
      <button type="button" @click="() => remove()">Forget preference</button>
    </div>
    <output aria-live="polite">{{ theme ? `Saved theme: ${theme}` : "No saved theme." }}</output>
  </div>
</template>
