<script setup lang="ts">
import { computed, shallowRef, useTemplateRef, watch } from "vue";

import Positioner from "../../overlays/positioner/positioner.vue";
import type { Placement, VirtualElement } from "../../overlays/positioner/positioner.ts";
import { richTextContext } from "./rich-text-context.ts";
import { selectionRect } from "./rich-text-dom.ts";
import { isCollapsed } from "./rich-text-model.ts";
import { useRichTextToolbar } from "./rich-text-toolbar-runtime.ts";

const {
  ariaLabel = "Selection formatting",
  placement = "top",
  offset = 8,
  open = undefined,
} = defineProps<{
  /**
   * Accessible name of the floating toolbar.
   *
   * @default "Selection formatting"
   */
  readonly ariaLabel?: string;

  /**
   * Preferred placement relative to the selection.
   *
   * @default "top"
   */
  readonly placement?: Placement;

  /**
   * Gap between the selection and the menu in CSS pixels.
   *
   * @default 8
   */
  readonly offset?: number;

  /**
   * Force visibility; `undefined` shows the menu for a non-empty selection while editing.
   *
   * @default undefined
   */
  readonly open?: boolean;
}>();

defineSlots<{
  /** Toolbar buttons for the selection. */
  default?(): unknown;
}>();

const editor = richTextContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const toolbar = useRichTextToolbar(editor, element);
const menuFocused = shallowRef(false);
const visible = computed(
  () =>
    open ??
    (editor.editable.value &&
      (editor.focused.value || menuFocused.value) &&
      !isCollapsed(editor.state.value.selection)),
);
const anchor = shallowRef<VirtualElement | null>(null);

watch(
  [visible, () => editor.state.value.selection],
  () => {
    const root = editor.contentElement.value;
    const rect = visible.value && root ? selectionRect(root) : null;
    anchor.value = rect ? { getBoundingClientRect: () => rect } : null;
  },
  { flush: "post" },
);

const menuProps = computed(() => ({
  role: "toolbar",
  "aria-orientation": "horizontal" as const,
  onKeydown: toolbar.onKeydown,
  onFocusin: () => {
    menuFocused.value = true;
  },
  onFocusout: (event: FocusEvent) => {
    const next = event.relatedTarget;
    menuFocused.value = next instanceof Node && element.value?.contains(next) === true;
  },
}));

defineExpose({ element });
</script>

<template>
  <div
    data-vize-ui="rich-text-bubble-menu-host"
    part="bubble-menu-host"
    :hidden="visible ? undefined : true"
  >
    <Positioner v-if="visible" :reference="anchor" :placement :offset>
      <div
        ref="element"
        v-bind="menuProps"
        :aria-label="ariaLabel"
        :aria-controls="editor.contentId.value"
        data-vize-ui="rich-text-bubble-menu"
        part="bubble-menu"
      >
        <slot />
      </div>
    </Positioner>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
