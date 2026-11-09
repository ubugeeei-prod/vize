<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";

import { provideFocusScope, useFocus, useFocusManager } from "../composables/useFocus.js";
import { useInput } from "../composables/useInput.js";
import { Box } from "./Box.js";
import type { ModalProps } from "./modal-types.js";
import { Text } from "./Text.js";

const {
  visible = true,
  title,
  width = "50%",
  height = "auto",
  border = "rounded",
  titleFg = "white",
  borderFg,
  bg,
  focusId = `modal-${Math.random().toString(36).slice(2)}`,
  initialFocusId,
  trapFocus = true,
  restoreFocus = true,
  closeOnEscape = true,
  accessibilityLabel,
} = defineProps<ModalProps>();

const emit = defineEmits<{
  close: [];
}>();

defineSlots<{
  default?: () => unknown;
}>();

defineOptions({ name: "FrescoModal" });

const manager = useFocusManager();
const wasFocusedBeforeOpen = ref<string>();
const isVisible = computed(() => visible);
const trapsFocus = computed(() => visible && trapFocus);

if (visible) wasFocusedBeforeOpen.value = manager.activeId.value;
watch(
  isVisible,
  (next, previous) => {
    if (next && !previous) wasFocusedBeforeOpen.value = manager.activeId.value;
  },
  { flush: "sync" },
);

const modalFocus = useFocus({
  id: focusId,
  isActive: isVisible,
  autoFocus: true,
  delegateFocusTo: computed(() => initialFocusId),
  trapFocus: trapsFocus,
});
provideFocusScope(modalFocus.id);

function focusInitialTarget(): void {
  if (!visible || !initialFocusId || !manager.focusableIds.value.includes(initialFocusId)) return;
  manager.focus(initialFocusId);
}

watch(
  [isVisible, manager.focusableIds],
  () => {
    void nextTick(focusInitialTarget);
  },
  { deep: true, flush: "post" },
);
watch(
  isVisible,
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
  isActive: computed(() => visible && closeOnEscape),
  onEscape: () => emit("close"),
});

const accessibilityProps = computed(() => ({
  ariaRole: "dialog" as const,
  ariaLabel: accessibilityLabel ?? title ?? "Dialog",
  ariaHidden: !visible,
  ariaState: { expanded: visible },
}));
</script>

<template>
  <Box
    :display="visible ? 'flex' : 'none'"
    justify-content="center"
    align-items="center"
    width="100%"
    height="100%"
    :focus-id="modalFocus.id"
    v-bind="accessibilityProps"
  >
    <Box
      v-if="visible"
      flex-direction="column"
      :width
      :height
      :border
      :fg="borderFg"
      :bg
      :padding="1"
    >
      <Box v-if="title" justify-content="center" :padding-bottom="1">
        <Text :content="title" :fg="titleFg" bold />
      </Box>
      <Box :flex-grow="1">
        <slot />
      </Box>
    </Box>
  </Box>
</template>
