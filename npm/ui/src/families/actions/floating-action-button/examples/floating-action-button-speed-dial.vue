<!-- Speed dial whose trigger opens a menu of related create actions and reports the one chosen. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  SpeedDial,
  SpeedDialAction,
  SpeedDialContent,
  SpeedDialTrigger,
} from "../floating-action-button.ts";
import type { SpeedDialSelectEvent } from "../floating-action-button.ts";

const created = ref<string | null>(null);

function onSelect(event: SpeedDialSelectEvent): void {
  created.value = event.value;
}
</script>

<template>
  <div>
    <SpeedDial direction="up" @select="onSelect">
      <SpeedDialTrigger aria-label="Create">+</SpeedDialTrigger>
      <SpeedDialContent>
        <SpeedDialAction value="note" label="New note">N</SpeedDialAction>
        <SpeedDialAction value="task" label="New task">T</SpeedDialAction>
        <SpeedDialAction value="event" label="New event">E</SpeedDialAction>
      </SpeedDialContent>
    </SpeedDial>
    <output>Created: {{ created ?? "nothing yet" }}</output>
  </div>
</template>
