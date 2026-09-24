<!-- Presence-driven fade transition toggled by a disclosure button. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Transition as PresenceTransition } from "../transition.ts";

const visible = ref(true);
const panelId = useId();
</script>

<template>
  <div>
    <button
      type="button"
      :aria-expanded="visible"
      :aria-controls="panelId"
      @click="() => (visible = !visible)"
    >
      {{ visible ? "Hide" : "Show" }} shipping details
    </button>
    <PresenceTransition :id="panelId" :present="visible" motion="fade">
      <template #default="{ status }">
        <p>Orders ship within two business days.</p>
        <small>Transition status: {{ status }}</small>
      </template>
    </PresenceTransition>
  </div>
</template>
