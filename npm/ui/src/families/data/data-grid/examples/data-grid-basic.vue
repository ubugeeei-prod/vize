<!-- Sortable, multi-select employee grid with typed columns and controlled sorting and selection. -->
<script setup lang="ts">
import { ref } from "vue";

import { createColumnHelper, DataGrid } from "../data-grid.ts";
import type { DataGridSort } from "../data-grid.ts";

interface Employee {
  readonly id: string;
  readonly name: string;
  readonly role: string;
  readonly salary: number;
}

const employees: readonly Employee[] = [
  { id: "e1", name: "Ada Lovelace", role: "Engineer", salary: 128000 },
  { id: "e2", name: "Grace Hopper", role: "Architect", salary: 154000 },
  { id: "e3", name: "Alan Turing", role: "Researcher", salary: 141000 },
  { id: "e4", name: "Katherine Johnson", role: "Analyst", salary: 119000 },
];

const column = createColumnHelper<Employee>();
const columns = [
  column.accessor("name", { header: "Name", width: 200 }),
  column.accessor("role", { header: "Role" }),
  column.accessor("salary", { header: "Salary", align: "end" }),
];

const rowId = (row: Employee): string => row.id;
const sorting = ref<readonly DataGridSort[]>([{ columnId: "name", direction: "ascending" }]);
const selection = ref<readonly string[]>([]);
</script>

<template>
  <div>
    <DataGrid
      v-model:sorting="sorting"
      v-model:selection="selection"
      :rows="employees"
      :columns
      :get-row-id="rowId"
      selection-mode="multiple"
      aria-label="Employees"
    />
    <output>{{ selection.length }} selected</output>
  </div>
</template>
