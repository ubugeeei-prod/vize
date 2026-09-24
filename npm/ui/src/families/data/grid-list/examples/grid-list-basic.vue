<!-- Multi-select, reorderable task list bound with v-model for both items and selection. -->
<script setup lang="ts">
import { ref } from "vue";

import { GridList } from "../grid-list.ts";

interface Task {
  readonly id: string;
  readonly title: string;
  readonly due: string;
}

const tasks = ref<readonly Task[]>([
  { id: "draft", title: "Draft release notes", due: "Mon" },
  { id: "review", title: "Review pull requests", due: "Tue" },
  { id: "deploy", title: "Deploy to staging", due: "Wed" },
  { id: "retro", title: "Run the retrospective", due: "Fri" },
]);
const selection = ref<readonly string[]>(["review"]);

const taskKey = (task: Task): string => task.id;
const taskText = (task: Task): string => task.title;
</script>

<template>
  <div>
    <GridList
      v-model:items="tasks"
      v-model:selection="selection"
      :get-key="taskKey"
      :get-text-value="taskText"
      selection-mode="multiple"
      :disabled-keys="['retro']"
      reorderable
      aria-label="Tasks this week"
    >
      <template #item="{ item, selected }">
        <span>{{ item.title }}</span>
        <span>due {{ item.due }}</span>
        <span v-if="selected">(selected)</span>
      </template>
    </GridList>
    <output>{{ selection.length }} of {{ tasks.length }} tasks selected</output>
  </div>
</template>
