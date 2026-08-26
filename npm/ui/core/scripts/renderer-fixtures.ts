/** Headless component fixtures compiled by every supported renderer lane. */
export const rendererFixtures = [
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
] as const;
