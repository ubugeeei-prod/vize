export const structureRendererFixtures = [
  {
    filename: "SplitterConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  SplitterGroup,
  SplitterHandle,
  SplitterPanel,
  useSplitterPersistence,
} from "./families/layout/splitter/splitter.ts";

const layout = useSplitterPersistence({ key: "consumer-layout" });
</script>

<template>
  <SplitterGroup v-model:layout="layout">
    <SplitterPanel :default-size="25" :min-size="10" collapsible>
      <template #default="{ size }">Navigation {{ size }}</template>
    </SplitterPanel>
    <SplitterHandle aria-label="Resize navigation">
      <template #default="{ state }">{{ state }}</template>
    </SplitterHandle>
    <SplitterPanel :default-size="75">
      <SplitterGroup orientation="vertical">
        <SplitterPanel :default-size="50">Editor</SplitterPanel>
        <SplitterHandle aria-label="Resize terminal" />
        <SplitterPanel :default-size="50">Terminal</SplitterPanel>
      </SplitterGroup>
    </SplitterPanel>
  </SplitterGroup>
</template>
`,
  },
  {
    filename: "TreeConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  TreeItem,
  TreeItemCheckbox,
  TreeItemToggle,
  TreeRoot,
  useTreeVirtualizer,
} from "./families/data/tree/tree.ts";

interface FileNode {
  readonly path: string;
  readonly children?: readonly FileNode[];
}

const files: readonly FileNode[] = [
  { path: "src", children: [{ path: "src/main.ts" }] },
  { path: "README.md" },
];
const virtualizer = useTreeVirtualizer({ itemSize: 24 });

function onAction(key: string): void {
  void key.length;
}
</script>

<template>
  <TreeRoot
    aria-label="Files"
    checkable
    :items="files"
    :get-key="(node) => node.path"
    :get-children="(node) => node.children"
    :virtualizer="virtualizer"
    @action="onAction"
  >
    <template #default="{ items }">
      <TreeItem v-for="item in items" :key="item.key" :item="item">
        <template #default="{ node, level }">
          <TreeItemToggle>
            <template #default="{ state }">{{ state }}</template>
          </TreeItemToggle>
          <TreeItemCheckbox>
            <template #default="{ checked }">{{ checked }}</template>
          </TreeItemCheckbox>
          <span :data-level="level">{{ node.path }}</span>
        </template>
      </TreeItem>
    </template>
  </TreeRoot>
</template>
`,
  },
] as const;
