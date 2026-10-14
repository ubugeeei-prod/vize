<script setup lang="ts">
import { useFetch } from "@vizejs/composable/use-fetch";

// Replace the data URL with your API endpoint.
const endpoint = `data:application/json,${encodeURIComponent('{"latest":"0.449.0"}')}`;
const { data, status, execute } = useFetch<{ latest: string }>(endpoint, { immediate: false });
</script>

<template>
  <div class="composable-example">
    <p>Fetch JSON when the reader asks, and show loading, result and error states.</p>
    <button type="button" :disabled="status === 'pending'" @click="() => execute()">
      Check for updates
    </button>
    <output aria-live="polite">{{
      status === "success"
        ? `Latest version: ${data?.latest}`
        : status === "error"
          ? "Could not check for updates."
          : status === "pending"
            ? "Checking…"
            : "Not checked yet."
    }}</output>
  </div>
</template>
