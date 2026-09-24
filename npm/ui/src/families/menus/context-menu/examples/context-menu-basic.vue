<!-- Right-click (or Shift+F10) menu for a file row with actions and a sort radio group. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuItemIndicator,
  ContextMenuLabel,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "../context-menu.ts";

const lastAction = ref("none");
const sortBy = ref<string | null>("name");
</script>

<template>
  <div>
    <ContextMenu>
      <ContextMenuTrigger>report-2026.pdf</ContextMenuTrigger>
      <ContextMenuContent aria-label="File actions">
        <ContextMenuItem @select="() => (lastAction = 'Open')">Open</ContextMenuItem>
        <ContextMenuItem @select="() => (lastAction = 'Rename')">Rename</ContextMenuItem>
        <ContextMenuItem disabled>Share</ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuLabel>Sort by</ContextMenuLabel>
        <ContextMenuRadioGroup v-model="sortBy">
          <ContextMenuRadioItem value="name">
            <ContextMenuItemIndicator>•</ContextMenuItemIndicator>
            Name
          </ContextMenuRadioItem>
          <ContextMenuRadioItem value="modified">
            <ContextMenuItemIndicator>•</ContextMenuItemIndicator>
            Date modified
          </ContextMenuRadioItem>
        </ContextMenuRadioGroup>
      </ContextMenuContent>
    </ContextMenu>
    <output>Last action: {{ lastAction }}; sorted by {{ sortBy }}</output>
  </div>
</template>
