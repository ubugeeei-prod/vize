<!-- Mail list whose rows swipe to reveal archive and delete actions, with a full swipe deleting the row. -->
<script setup lang="ts">
import { ref } from "vue";

import {
  SwipeActions,
  SwipeActionsAction,
  SwipeActionsContent,
  SwipeActionsTray,
} from "../swipe-actions.ts";

const messages = ref(["Team lunch on Friday", "Invoice #1042", "Design review notes"]);
const lastAction = ref<string | null>(null);

function run(action: string, message: string): void {
  lastAction.value = `${action}: ${message}`;
  messages.value = messages.value.filter((candidate) => candidate !== message);
}
</script>

<template>
  <div>
    <ul aria-label="Inbox">
      <SwipeActions
        v-for="message in messages"
        :key="message"
        as="li"
        @full-swipe="() => run('Deleted', message)"
      >
        <SwipeActionsTray side="leading" aria-label="Quick actions">
          <SwipeActionsAction value="archive" @select="() => run('Archived', message)">
            Archive
          </SwipeActionsAction>
        </SwipeActionsTray>
        <SwipeActionsTray side="trailing" aria-label="Destructive actions">
          <SwipeActionsAction value="delete" @select="() => run('Deleted', message)">
            Delete
          </SwipeActionsAction>
        </SwipeActionsTray>
        <SwipeActionsContent>{{ message }}</SwipeActionsContent>
      </SwipeActions>
    </ul>
    <output>{{ lastAction ?? "No actions yet" }}</output>
  </div>
</template>
