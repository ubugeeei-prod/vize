<!-- Presence indicators exposed as labelled images next to each member name. -->
<script setup lang="ts">
import { StatusLight } from "../status-light.ts";
</script>

<template>
  <ul>
    <li>
      <StatusLight state="online" tone="success" aria-label="Online" />
      <span>Aiko Tanaka</span>
    </li>
    <li>
      <StatusLight state="away" tone="warning" aria-label="Away" />
      <span>Marcus Lee</span>
    </li>
    <li>
      <StatusLight state="offline" tone="neutral" aria-label="Offline" />
      <span>Priya Nair</span>
    </li>
  </ul>
</template>
