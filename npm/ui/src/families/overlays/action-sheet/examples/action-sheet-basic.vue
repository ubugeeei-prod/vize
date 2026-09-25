<!-- Bottom action sheet offering photo actions, including a disabled and a destructive one. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  ActionSheet,
  ActionSheetCancel,
  ActionSheetContent,
  ActionSheetDescription,
  ActionSheetItem,
  ActionSheetMenu,
  ActionSheetTitle,
  ActionSheetTrigger,
} from "../action-sheet.ts";
import type { ActionSheetSelectEvent } from "../action-sheet.ts";

const open = ref(false);
const chosen = ref<string | null>(null);

function onSelect(event: ActionSheetSelectEvent): void {
  chosen.value = event.value;
}
</script>

<template>
  <div>
    <ActionSheet v-model:open="open">
      <ActionSheetTrigger>Photo options</ActionSheetTrigger>
      <ActionSheetContent>
        <ActionSheetTitle>Photo</ActionSheetTitle>
        <ActionSheetDescription>Choose what to do with this photo.</ActionSheetDescription>
        <ActionSheetMenu>
          <ActionSheetItem value="share" @select="onSelect">Share</ActionSheetItem>
          <ActionSheetItem value="copy" @select="onSelect">Copy link</ActionSheetItem>
          <ActionSheetItem value="archive" disabled @select="onSelect">Archive</ActionSheetItem>
          <ActionSheetItem value="delete" destructive @select="onSelect">Delete</ActionSheetItem>
        </ActionSheetMenu>
        <ActionSheetCancel>Cancel</ActionSheetCancel>
      </ActionSheetContent>
    </ActionSheet>
    <output>Last action: {{ chosen ?? "none" }}</output>
  </div>
</template>
