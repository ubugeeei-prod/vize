/** Data-view fixtures compiled by every supported renderer lane. */
export const dataViewRendererFixtures = [
  {
    filename: "DataGridConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { createColumnHelper, DataGrid } from "./families/data/data-grid/data-grid.ts";
import type { DataGridSort } from "./families/data/data-grid/data-grid.ts";

interface User {
  readonly id: string;
  readonly name: string;
  readonly age: number;
}

const column = createColumnHelper<User>();
const columns = [
  column.accessor("name", { header: "Name", editable: true }),
  column.accessor("age", { header: "Age", parse: (input) => Number(input) }),
  column.display("actions", { header: "Actions" }),
];
const rows = ref<readonly User[]>([{ id: "a", name: "Ada", age: 36 }]);
const sorting = ref<readonly DataGridSort[]>([]);
const selection = ref<readonly string[]>([]);
</script>

<template>
  <DataGrid
    v-model:sorting="sorting"
    v-model:selection="selection"
    aria-label="Users"
    :rows
    :columns
    :get-row-id="(row) => row.id"
    selection-mode="multiple"
  >
    <template #cell="cell">
      <button v-if="cell.columnId === 'actions'" type="button" tabindex="-1">Edit</button>
      <template v-else>{{ cell.text }}</template>
    </template>
  </DataGrid>
</template>
`,
  },
  {
    filename: "GridListConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { GridList } from "./families/data/grid-list/grid-list.ts";

const items = ref(["Alps", "Beach"]);
const selection = ref<readonly string[]>([]);
</script>

<template>
  <GridList v-model:items="items" v-model:selection="selection" aria-label="Photos" :items reorderable>
    <template #item="{ item, dragHandleProps }">
      {{ item }}
      <button v-if="dragHandleProps" v-bind="dragHandleProps" type="button">Move</button>
    </template>
  </GridList>
</template>
`,
  },
  {
    filename: "KanbanConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { Kanban } from "./families/data/kanban/kanban.ts";
import type { KanbanBoard } from "./families/data/kanban/kanban.ts";

type Lane = "todo" | "done";
const columns = [
  { id: "todo" as const, title: "To do" },
  { id: "done" as const, title: "Done" },
];
const board = ref<KanbanBoard<string, Lane>>({ todo: ["Write"], done: [] });
</script>

<template>
  <Kanban v-model="board" aria-label="Board" :columns :get-card-key="(card) => card">
    <template #card="{ card }">{{ card }}</template>
  </Kanban>
</template>
`,
  },
] as const;
