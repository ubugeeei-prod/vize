<!-- Menu button with actions, a disabled item, and a checkbox item, opened via v-model:open. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  MenuCheckboxItem,
  MenuContent,
  MenuItem,
  MenuItemIndicator,
  MenuRoot,
  MenuSeparator,
  MenuTrigger,
} from "../menu.ts";

const open = ref(false);
const pinned = ref(false);
const lastAction = ref("none");
</script>

<template>
  <div>
    <MenuRoot v-model:open="open">
      <MenuTrigger>Document actions</MenuTrigger>
      <MenuContent aria-label="Document actions">
        <MenuItem @select="() => (lastAction = 'Rename')">Rename</MenuItem>
        <MenuItem @select="() => (lastAction = 'Duplicate')">Duplicate</MenuItem>
        <MenuItem disabled>Move to shared drive</MenuItem>
        <MenuSeparator />
        <MenuCheckboxItem v-model="pinned">
          <MenuItemIndicator>✓</MenuItemIndicator>
          Pin to sidebar
        </MenuCheckboxItem>
      </MenuContent>
    </MenuRoot>
    <output>Last action: {{ lastAction }}; pinned: {{ pinned ? "yes" : "no" }}</output>
  </div>
</template>
