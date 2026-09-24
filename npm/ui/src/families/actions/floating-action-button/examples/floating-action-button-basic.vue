<!-- Extended floating action button with visible text that counts how many drafts it has started. -->
<script setup lang="ts">
import { ref } from "vue";

import { FloatingActionButton } from "../floating-action-button.ts";

const drafts = ref(0);

function startDraft(): void {
  drafts.value += 1;
}
</script>

<template>
  <div>
    <FloatingActionButton placement="bottom-end" extended @click="startDraft">
      Compose
    </FloatingActionButton>
    <output>Drafts started: {{ drafts }}</output>
  </div>
</template>
