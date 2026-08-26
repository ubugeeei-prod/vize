/** Headless component fixtures compiled by every supported renderer lane. */
export const rendererFixtures = [
  {
    filename: "NestedPortalConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { usePortalStack } from "./portal.ts";

const target = ref("body");
const stack = usePortalStack();
</script>

<template>
  <Teleport :to="target" defer>
    <div data-portalled="outer">
      Outer layer
      <Teleport :to="target" defer>
        <div data-portalled="inner" :data-layers="stack.value.length">Inner layer</div>
      </Teleport>
    </div>
  </Teleport>
</template>
`,
  },
  {
    filename: "NestedPresenceConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import Presence from "./presence.vue";

const outerOpen = ref(true);
const innerOpen = ref(true);
</script>

<template>
  <Presence :present="outerOpen">
    <div>
      Outer layer
      <Presence :present="innerOpen">
        <div>Inner layer</div>
      </Presence>
    </div>
  </Presence>
</template>
`,
  },
  {
    filename: "DragAndDropConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useDragAndDrop } from "./drag-and-drop.ts";

const handle = ref<HTMLElement | null>(null);
const zone = ref<HTMLElement | null>(null);
const dropped = ref(0);
const controller = useDragAndDrop<{ id: number }>();
const source = controller.registerSource({
  key: "card",
  element: handle,
  payload: { kind: "card", data: { id: 1 } },
});
controller.registerTarget({
  key: "zone",
  element: zone,
  onDrop: () => {
    dropped.value += 1;
  },
});
</script>

<template>
  <button
    ref="handle"
    v-bind="source.sourceProps"
    type="button"
    :data-dragging="source.isDragging.value || undefined"
  >
    Drag me
  </button>
  <section ref="zone" :data-over="controller.targetKey.value === 'zone' || undefined">
    <output>{{ dropped }}</output>
  </section>
</template>
`,
  },
  {
    filename: "SortableConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useSortable } from "./sortable.ts";

const items = ref(["alpha", "bravo"]);
const first = ref<HTMLElement | null>(null);
const second = ref<HTMLElement | null>(null);
const sortable = useSortable({
  onSortCommit(event) {
    const next = [...items.value];
    const [moved] = next.splice(event.fromIndex, 1);
    if (moved !== undefined) next.splice(event.toIndex, 0, moved);
    items.value = next;
  },
});
const alpha = sortable.registerItem({ key: "alpha", element: first });
const bravo = sortable.registerItem({ key: "bravo", element: second });
</script>

<template>
  <ul :data-sorting="sortable.isSorting.value || undefined">
    <li ref="first" v-bind="alpha.itemProps" tabindex="0">{{ items[0] }}</li>
    <li ref="second" v-bind="bravo.itemProps" tabindex="0">{{ items[1] }}</li>
  </ul>
</template>
`,
  },
  {
    filename: "CommandConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useCommandRouter } from "./command.ts";

const output = ref("");
const router = useCommandRouter<"save">();
router.register({
  id: "save",
  title: "Save Document",
  run: () => {
    output.value = "saved";
  },
});
</script>

<template>
  <button type="button" :disabled="!router.isEnabled('save')" @click="router.execute('save')">
    Save
  </button>
  <output>{{ output }}</output>
</template>
`,
  },
  {
    filename: "HistoryConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useHistory } from "./history.ts";

const value = ref(0);
const history = useHistory();
function increment() {
  const before = value.value;
  value.value += 1;
  history.pushSnapshot({
    before,
    after: value.value,
    apply: (next) => {
      value.value = next;
    },
    label: "Increment",
  });
}
</script>

<template>
  <button type="button" @click="increment">Increment</button>
  <button type="button" :disabled="!history.canUndo.value" @click="history.undo()">Undo</button>
  <button type="button" :disabled="!history.canRedo.value" @click="history.redo()">Redo</button>
  <output>{{ value }}</output>
</template>
`,
  },
  {
    filename: "ShortcutConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { formatShortcut, useShortcutRegistry } from "./shortcut.ts";

const host = ref<HTMLElement | null>(null);
const count = ref(0);
const registry = useShortcutRegistry({ target: host, platform: "standard" });
registry.register({
  shortcut: "Mod+K",
  description: "Open the palette",
  handler: () => {
    count.value += 1;
  },
});
</script>

<template>
  <div ref="host" tabindex="0" :data-pending="registry.pendingSequence.value.length || undefined">
    <kbd>{{ formatShortcut("Mod+K", { platform: "standard" }) }}</kbd>
    <output>{{ count }}</output>
  </div>
</template>
`,
  },
  {
    filename: "DismissableLayerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useDismissableLayer } from "./dismissable-layer.ts";

const root = ref<HTMLElement | null>(null);
const branch = ref<HTMLElement | null>(null);
const layer = useDismissableLayer({
  root,
  branches: () => (branch.value ? [branch.value] : []),
  onDismiss(event) {
    void event.reason;
  },
});
</script>

<template>
  <section
    ref="root"
    v-bind="layer.layerProps"
    :data-active="layer.isActive.value || undefined"
    :data-top-layer="layer.isTopLayer.value || undefined"
  >
    <button type="button">Inside</button>
  </section>
  <aside ref="branch" v-bind="layer.branchProps">
    Portalled branch
  </aside>
</template>
`,
  },
  {
    filename: "FocusGuardsConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { focusGuardPreset, useFocusGuards } from "./focus-guards.ts";

const root = ref<HTMLElement | null>(null);
const guards = useFocusGuards({ root });
</script>

<template>
  <span v-bind="guards.beforeProps" :style="focusGuardPreset"></span>
  <div ref="root"><button type="button">Inside</button></div>
  <span v-bind="guards.afterProps" :style="focusGuardPreset"></span>
</template>
`,
  },
  {
    filename: "ScrollLockConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useScrollLock } from "./scroll-lock.ts";

const root = ref<HTMLElement | null>(null);
const ownerDocument = ref<Document | null>(null);
const lock = useScrollLock({ document: ownerDocument, strategy: "auto" });
onMounted(() => {
  ownerDocument.value = root.value?.ownerDocument ?? null;
});
</script>

<template>
  <div ref="root" :data-locked="lock.isLocked.value || undefined">
    Modal content
  </div>
</template>
`,
  },
  {
    filename: "InertOutsideConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useInertOutside } from "./inert-outside.ts";

const root = ref<HTMLElement | null>(null);
const isolation = useInertOutside({ root, mode: "both" });
</script>

<template>
  <div ref="root" :data-active="isolation.isActive.value || undefined">
    Modal content
  </div>
</template>
`,
  },
  {
    filename: "FocusScopeConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useFocusScope } from "./focus-scope.ts";

const root = ref<HTMLElement | null>(null);
const scope = useFocusScope({
  root,
  contain: true,
  autoFocus: true,
  restoreFocus: true,
});
</script>

<template>
  <div ref="root" :data-active="scope.isActive.value || undefined">
    <button type="button">Inside</button>
  </div>
</template>
`,
  },
  {
    filename: "PointerGraceConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { usePointerGrace } from "./pointer-grace.ts";

const grace = usePointerGrace({
  delay: 300,
  onGraceEnd() {},
});
</script>

<template>
  <div
    :data-pending="grace.isPending.value || undefined"
    @pointermove="grace.handleMove({ x: $event.clientX, y: $event.clientY })"
  >
    Grace target
  </div>
</template>
`,
  },
  {
    filename: "MeasureConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useSizeObserver, useVisibilityObserver } from "./measure.ts";

const host = ref<HTMLElement | null>(null);
const sizes = useSizeObserver({
  onResize(entries) {
    void entries.length;
  },
});
const visibility = useVisibilityObserver({
  onVisibilityChange(entries) {
    void entries.length;
  },
});
onMounted(() => {
  if (host.value) {
    sizes.observe(host.value);
    visibility.observe(host.value);
  }
});
</script>

<template>
  <div ref="host" :data-observed="sizes.observedCount.value || undefined">
    Measured content
  </div>
</template>
`,
  },
  {
    filename: "VirtualizerConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useVirtualizer } from "./virtualizer.ts";

const viewport = ref<HTMLElement | null>(null);
const virtualizer = useVirtualizer({
  count: 10000,
  estimateItemSize: 32,
  initialRect: { width: 320, height: 480 },
});
onMounted(() => {
  virtualizer.setViewport(viewport.value);
});
</script>

<template>
  <div ref="viewport" style="overflow-y: auto">
    <div :style="{ height: virtualizer.totalSize.value + 'px', position: 'relative' }">
      <div
        v-for="item in virtualizer.virtualItems.value"
        :key="item.key"
        :data-index="item.index"
        :data-sticky="item.isSticky || undefined"
        :style="{ position: 'absolute', top: item.start + 'px' }"
      >
        Row {{ item.index }}
      </div>
    </div>
  </div>
</template>
`,
  },
] as const;
