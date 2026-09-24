export const layoutPatternRendererFixtures = [
  {
    filename: "DashboardGridConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import {
  DashboardGrid,
  DashboardGridItem,
  type DashboardLayout,
} from "./families/layout/dashboard-grid/dashboard-grid.ts";

const layout = ref<DashboardLayout>([{ id: "sales", x: 0, y: 0, w: 2, h: 2 }]);
</script>

<template>
  <DashboardGrid v-model:layout="layout" :columns="6" label="Metrics">
    <template #default="{ rows }">
      <DashboardGridItem id="sales" label="Sales">
        <template #default="{ item, handleProps, resizeHandleProps, dragging }">
          <h3 v-bind="handleProps">{{ item.w }} {{ dragging }} {{ rows }}</h3>
          <span v-bind="resizeHandleProps"></span>
        </template>
      </DashboardGridItem>
    </template>
  </DashboardGrid>
</template>
`,
  },
  {
    filename: "MasonryConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Masonry } from "./families/layout/masonry/masonry.ts";

interface Photo {
  readonly id: string;
  readonly height: number;
}

const photos: readonly Photo[] = [
  { id: "dunes", height: 120 },
  { id: "harbor", height: 80 },
];
</script>

<template>
  <Masonry
    :items="photos"
    :columns="2"
    :gap="8"
    :estimate-height="(photo) => photo.height"
    :get-key="(photo) => photo.id"
  >
    <template #item="{ item, column }">
      <figure :data-column="column">{{ item.id }}</figure>
    </template>
  </Masonry>
</template>
`,
  },
  {
    filename: "MasterDetailConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";

import { MasterDetail } from "./families/layout/master-detail/master-detail.ts";

const selected = ref<"inbox" | "sent" | null>(null);
</script>

<template>
  <MasterDetail v-model:selected="selected" :ssr-width="1024" master-label="Mailboxes">
    <template #master="{ select }">
      <button type="button" @click="select('inbox')">Inbox</button>
    </template>
    <template #detail="{ selected: current, back }">
      <h2>{{ current }}</h2>
      <button type="button" @click="back">Back</button>
    </template>
    <template #empty>Pick a mailbox</template>
  </MasterDetail>
</template>
`,
  },
  {
    filename: "WindowManagerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  FloatingWindow,
  WindowDock,
  WindowManager,
  useWindowLayoutPersistence,
} from "./families/layout/window-manager/window-manager.ts";

const layout = useWindowLayoutPersistence({ key: "desktop" });

function onClose(): void {
  void layout.value;
}
</script>

<template>
  <WindowManager v-model:layout="layout" :snap-threshold="12">
    <FloatingWindow id="editor" title="Editor" :default-rect="{ x: 16, y: 16, width: 320, height: 200 }" @close="onClose">
      <template #default="{ handleProps, minimize, toggleMaximize, close, active }">
        <header v-bind="handleProps">Editor {{ active }}</header>
        <button type="button" @click="minimize">Minimize</button>
        <button type="button" @click="toggleMaximize">Maximize</button>
        <button type="button" @click="close">Close</button>
      </template>
    </FloatingWindow>
    <WindowDock label="Taskbar" filter="minimized" />
  </WindowManager>
</template>
`,
  },
  {
    filename: "ResponsiveConsumer.vue",
    source: String.raw`<script setup lang="ts">
import {
  ResponsiveShow,
  ResponsiveSwitch,
  useBreakpoint,
} from "./families/layout/responsive/responsive.ts";

const breakpoint = useBreakpoint({ ssrWidth: 1024 });
</script>

<template>
  <ResponsiveSwitch :ssr-width="1024">
    <template #base="{ width }">Phone {{ width }}</template>
    <template #lg="{ active }">Desktop {{ active }}</template>
  </ResponsiveSwitch>
  <ResponsiveShow above="md" :ssr-width="1024" hide-mode="hidden">
    <template #default="{ visible }">{{ visible }} {{ breakpoint.active.value }}</template>
  </ResponsiveShow>
</template>
`,
  },
  {
    filename: "StickyStackConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { StickyStack, StickyStackItem } from "./families/layout/sticky-stack/sticky-stack.ts";

function onStuck(stuck: boolean): void {
  void stuck;
}
</script>

<template>
  <StickyStack :offset="8">
    <template #default="{ total }">
      <StickyStackItem as="header" :estimated-height="48" @stuck-change="onStuck">
        <template #default="{ top, stuck }">App bar {{ top }} {{ stuck }}</template>
      </StickyStackItem>
      <StickyStackItem as="nav" :estimated-height="32">Tabs</StickyStackItem>
      <p>{{ total }}</p>
    </template>
  </StickyStack>
</template>
`,
  },
] as const;
