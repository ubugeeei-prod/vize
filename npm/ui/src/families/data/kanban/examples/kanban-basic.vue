<!-- Sprint board bound with v-model: cards move between columns by pointer or keyboard, with a WIP limit. -->
<script setup lang="ts">
import { ref } from "vue";

import { Kanban, type KanbanBoard, type KanbanColumnDefinition } from "../kanban.ts";

type Lane = "todo" | "doing" | "done";

interface Task {
  readonly id: string;
  readonly title: string;
}

const columns: readonly KanbanColumnDefinition<Lane>[] = [
  { id: "todo", title: "To do" },
  { id: "doing", title: "In progress", limit: 2 },
  { id: "done", title: "Done" },
];

const board = ref<KanbanBoard<Task, Lane>>({
  todo: [
    { id: "t1", title: "Write release notes" },
    { id: "t2", title: "Audit color contrast" },
  ],
  doing: [{ id: "t3", title: "Fix date picker focus" }],
  done: [{ id: "t4", title: "Upgrade to Vue 3.5" }],
});
</script>

<template>
  <Kanban
    v-model="board"
    aria-label="Sprint board"
    :columns
    :get-card-key="(task: Task) => task.id"
    :get-card-label="(task: Task) => task.title"
  >
    <template #column="{ column, cards, full }">
      {{ column.title }} ({{ cards.length
      }}{{ column.limit === undefined ? "" : ` of ${column.limit}` }})
      <template v-if="full"> · full</template>
    </template>
    <template #card="{ card }">{{ card.title }}</template>
    <template #empty>No tasks</template>
  </Kanban>
</template>
