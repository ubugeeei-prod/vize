<!-- Icon-only buttons that carry their accessible name through aria-label. -->
<script setup lang="ts">
import { ref } from "vue";

import { IconButton } from "../icon-button.ts";

const count = ref(0);

function addItem(): void {
  count.value += 1;
}
</script>

<template>
  <div>
    <IconButton v-bind="{ ariaLabel: 'Add item' }" variant="solid" tone="accent" @press="addItem">
      <svg
        viewBox="0 0 24 24"
        width="1em"
        height="1em"
        fill="none"
        stroke="currentColor"
        aria-hidden="true"
      >
        <path d="M12 5v14M5 12h14" />
      </svg>
    </IconButton>
    <IconButton v-bind="{ ariaLabel: 'Delete item' }" tone="danger" disabled>
      <svg
        viewBox="0 0 24 24"
        width="1em"
        height="1em"
        fill="none"
        stroke="currentColor"
        aria-hidden="true"
      >
        <path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13" />
      </svg>
    </IconButton>
    <output>{{ count }} items</output>
  </div>
</template>
