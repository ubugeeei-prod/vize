/** Command, history, shortcut, and action fixtures compiled by every supported renderer lane. */
export const commandRendererFixtures = [
  {
    filename: "CopyButtonConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { CopyButton } from "./families/actions/copy-button/copy-button.ts";

function onCopy(value: string) {
  void value;
}
</script>

<template>
  <CopyButton value="https://vize.dev/docs" idle-label="Copy docs link" @copy="onCopy" />
</template>
`,
  },
  {
    filename: "FullscreenButtonConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { FullscreenButton } from "./families/actions/fullscreen-button/fullscreen-button.ts";
import type { FullscreenButtonOperation } from "./families/actions/fullscreen-button/fullscreen-button.ts";

function onFullscreen(operation: FullscreenButtonOperation, event: MouseEvent) {
  void operation;
  void event;
}
</script>

<template>
  <FullscreenButton
    aria-label="Toggle fullscreen"
    enter-label="Enter fullscreen"
    @fullscreen="onFullscreen"
  />
</template>
`,
  },
  {
    filename: "PrintButtonConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { PrintButton } from "./families/actions/print-button/print-button.ts";

function onPrint(event: MouseEvent) {
  void event;
}
</script>

<template>
  <PrintButton aria-label="Print invoice" idle-label="Print invoice" @print="onPrint" />
</template>
`,
  },
  {
    filename: "ShareButtonConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ShareButton } from "./families/actions/share-button/share-button.ts";
import type { ShareButtonPayload } from "./families/actions/share-button/share-button.ts";

function onShare(payload: ShareButtonPayload, event: MouseEvent) {
  void payload;
  void event;
}
</script>

<template>
  <ShareButton
    aria-label="Share docs"
    idle-label="Share docs"
    text="Read the Vize docs"
    title="Vize docs"
    url="https://vize.dev/docs"
    @share="onShare"
  />
</template>
`,
  },
  {
    filename: "ToolbarConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { Toolbar, ToolbarItem } from "./families/actions/toolbar/toolbar.ts";

function run(value: string) {
  void value;
}
</script>

<template>
  <Toolbar aria-label="Editor actions" dir="rtl" @press="run">
    <ToolbarItem value="save">Save</ToolbarItem>
    <ToolbarItem value="publish">Publish</ToolbarItem>
  </Toolbar>
</template>
`,
  },
  {
    filename: "ButtonGroupConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ButtonGroup, ButtonGroupItem } from "./families/actions/button-group/button-group.ts";

function run(value: string) {
  void value;
}
</script>

<template>
  <ButtonGroup aria-label="Editor actions" role="toolbar" @press="run">
    <ButtonGroupItem value="save">Save</ButtonGroupItem>
    <ButtonGroupItem value="publish">Publish</ButtonGroupItem>
  </ButtonGroup>
</template>
`,
  },
  {
    filename: "CommandConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import { useCommandRouter } from "./families/foundations/command/command.ts";

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
import { useHistory } from "./families/interaction/history/history.ts";

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
import { formatShortcut, useShortcutRegistry } from "./families/interaction/shortcut/shortcut.ts";

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
];
