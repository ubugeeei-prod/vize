<!-- Data-driven file tree with expand toggles and selection bound through v-model:selected. -->
<script setup lang="ts">
import { ref } from "vue";

import { TreeItem, TreeItemToggle, TreeRoot } from "../tree.ts";

interface FileNode {
  readonly id: string;
  readonly name: string;
  readonly children?: readonly FileNode[];
}

const files: readonly FileNode[] = [
  {
    id: "src",
    name: "src",
    children: [
      { id: "app", name: "App.vue" },
      { id: "main", name: "main.ts" },
    ],
  },
  { id: "docs", name: "docs", children: [{ id: "guide", name: "guide.md" }] },
  { id: "readme", name: "README.md" },
];
const selected = ref<readonly string[]>(["app"]);
</script>

<template>
  <div>
    <TreeRoot
      v-model:selected="selected"
      aria-label="Project files"
      :items="files"
      :get-key="(node) => node.id"
      :get-children="(node) => node.children"
      :default-expanded="['src']"
    >
      <template #default="{ items }">
        <TreeItem v-for="item in items" :key="item.key" :item>
          <TreeItemToggle>
            <template #default="{ expandable, expanded }">
              <span v-if="expandable" aria-hidden="true">{{ expanded ? "-" : "+" }}</span>
            </template>
          </TreeItemToggle>
          {{ item.node.name }}
        </TreeItem>
      </template>
    </TreeRoot>
    <output>Selected: {{ selected.join(", ") || "nothing" }}</output>
  </div>
</template>
