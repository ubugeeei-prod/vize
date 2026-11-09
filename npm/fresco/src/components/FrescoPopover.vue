<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";

import { provideFocusScope, useFocus, useFocusManager } from "../composables/useFocus.js";
import { useInput } from "../composables/useInput.js";
import { Box } from "./Box.js";
import type { PopoverPlacement, PopoverProps } from "./popover-types.js";

interface PopoverPanelPosition {
  readonly top?: string;
  readonly right?: string;
  readonly bottom?: string;
  readonly left?: string;
  readonly marginTop?: number;
  readonly marginRight?: number;
  readonly marginBottom?: number;
  readonly marginLeft?: number;
}

const {
  open = false,
  placement = "bottom",
  offset = 0,
  width,
  height,
  border = "rounded",
  fg,
  bg,
  focusId = `popover-${Math.random().toString(36).slice(2)}`,
  initialFocusId,
  trapFocus = true,
  restoreFocus = true,
  closeOnEscape = true,
  accessibilityRole = "dialog",
  accessibilityLabel = "Popover",
} = defineProps<PopoverProps>();

const emit = defineEmits<{
  "close": [];
}>();

defineOptions({ name: "FrescoPopover" });

const manager = useFocusManager();
const wasFocusedBeforeOpen = ref<string>();
const isOpen = computed(() => open);
const trapsFocus = computed(() => open && trapFocus);

if (open) wasFocusedBeforeOpen.value = manager.activeId.value;
watch(
  isOpen,
  (next, previous) => {
    if (next && !previous) wasFocusedBeforeOpen.value = manager.activeId.value;
  },
  { flush: "sync" },
);

const popoverFocus = useFocus({
  id: focusId,
  isActive: isOpen,
  autoFocus: true,
  delegateFocusTo: computed(() => initialFocusId),
  trapFocus: trapsFocus,
});
provideFocusScope(popoverFocus.id);

function focusInitialTarget(): void {
  if (!open || !initialFocusId || !manager.focusableIds.value.includes(initialFocusId)) return;
  manager.focus(initialFocusId);
}

watch(
  [isOpen, manager.focusableIds],
  () => {
    void nextTick(focusInitialTarget);
  },
  { deep: true, flush: "post" },
);
watch(
  isOpen,
  (next, previous) => {
    if (next || !previous || !restoreFocus) return;
    const target = wasFocusedBeforeOpen.value;
    if (target && manager.focusableIds.value.includes(target)) manager.focus(target);
    wasFocusedBeforeOpen.value = undefined;
  },
  { flush: "post" },
);
onMounted(() => void nextTick(focusInitialTarget));

useInput({
  isActive: computed(() => open && closeOnEscape),
  onEscape: () => emit("close"),
});

function panelPosition(selected: PopoverPlacement): PopoverPanelPosition {
  const gap = Number.isFinite(offset) ? Math.max(0, offset) : 0;
  switch (selected) {
    case "top":
      return { bottom: "100%", marginBottom: gap };
    case "right":
      return { left: "100%", marginLeft: gap };
    case "left":
      return { right: "100%", marginRight: gap };
    case "bottom":
    default:
      return { top: "100%", marginTop: gap };
  }
}

const position = computed(() => panelPosition(placement));
const accessibilityProps = computed(() => ({
  ariaRole: accessibilityRole,
  ariaLabel: accessibilityLabel,
  ariaState: { expanded: open },
}));
</script>

<template>
  <Box position="relative">
    <slot name="anchor" />
    <Box
      v-if="open"
      position="absolute"
      :top="position.top"
      :right="position.right"
      :bottom="position.bottom"
      :left="position.left"
      :margin-top="position.marginTop"
      :margin-right="position.marginRight"
      :margin-bottom="position.marginBottom"
      :margin-left="position.marginLeft"
      :width
      :height
      :border="border === 'none' ? undefined : border"
      :fg
      :bg
      :padding="1"
      :focus-id="popoverFocus.id"
      v-bind="accessibilityProps"
    >
      <slot />
    </Box>
  </Box>
</template>
