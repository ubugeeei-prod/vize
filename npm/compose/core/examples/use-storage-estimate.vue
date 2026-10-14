<script setup lang="ts">
import { useMounted } from "@vizejs/composable/use-mounted";
import { useStorageEstimate } from "@vizejs/composable/use-storage-estimate";

const { supported, usage, quota, percentUsed, refresh } = useStorageEstimate();
const mounted = useMounted();
const megabytes = (bytes: number | null) =>
  bytes === null ? "?" : (bytes / 1024 / 1024).toFixed(1);
</script>

<template>
  <div class="composable-example">
    <p>Warn before offline data fills the browser's storage quota.</p>
    <button type="button" @click="() => refresh()">Refresh estimate</button>
    <output aria-live="polite">{{
      !mounted
        ? "Reading the storage estimate…"
        : !supported
          ? "Storage estimates are not available."
          : `Using ${megabytes(usage)} MB of ${megabytes(quota)} MB (${percentUsed?.toFixed(2) ?? "?"}%)`
    }}</output>
  </div>
</template>
