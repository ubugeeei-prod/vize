<script setup lang="ts">
import { computed, onMounted, shallowRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import { useShortcutRegistry } from "../../interaction/shortcut/shortcut.ts";
// Import through the Dialog entry so the palette shares Dialog's packaged chunks.
import { DialogContent, DialogOverlay, DialogPortal, DialogRoot } from "../dialog/dialog.ts";
import { commandPaletteDialogContext } from "./command-palette-context.ts";
import type { CommandPaletteDialogExpose } from "./command-palette-types.ts";

const {
  open = undefined,
  defaultOpen = false,
  shortcut = "Mod+K",
  closeOnSelect = true,
  ariaLabel = "Command palette",
  to = "body",
  portalDisabled = false,
} = defineProps<{
  /**
   * Controlled dialog visibility. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial uncontrolled dialog visibility.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Global shortcut that toggles the dialog, resolved per platform (`Mod` is
   * Command on Apple platforms and Control elsewhere). `null` disables it.
   *
   * @default "Mod+K"
   */
  readonly shortcut?: string | null;

  /**
   * Close the dialog after an item is selected.
   *
   * @default true
   */
  readonly closeOnSelect?: boolean;

  /**
   * Accessible name of the dialog.
   *
   * @default "Command palette"
   */
  readonly ariaLabel?: string;

  /**
   * CSS selector or element the dialog layer is moved into.
   *
   * @default "body"
   */
  readonly to?: string | HTMLElement;

  /**
   * Render the dialog in place instead of teleporting it.
   *
   * @default false
   */
  readonly portalDisabled?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the dialog requests a controlled open value. */
  "update:open": [value: boolean];
}>();

defineSlots<{
  /** A CommandPaletteRoot and its parts. Receives the dialog state and a close function. */
  default?(props: { readonly open: boolean; readonly close: () => boolean }): unknown;
}>();

const openState = useControllableState({ value: () => open, defaultValue: () => defaultOpen });
const isOpen = openState.value;
const shortcutTarget = shallowRef<Document | null>(null);
const shortcuts = useShortcutRegistry({ target: shortcutTarget });
let releaseShortcut: (() => void) | null = null;

function setOpen(value: boolean): boolean {
  const changed = openState.set(value);
  if (changed) emit("update:open", value);
  return changed;
}

function toggle(): boolean {
  return setOpen(!isOpen.value);
}

function close(): boolean {
  return setOpen(false);
}

commandPaletteDialogContext.provide({
  didSelect: () => {
    if (closeOnSelect) setOpen(false);
  },
});

watch(
  () => shortcut,
  (next) => {
    releaseShortcut?.();
    releaseShortcut =
      next === null
        ? null
        : shortcuts.register({
            allowInEditable: true,
            description: "Toggle the command palette",
            handler: () => {
              toggle();
            },
            shortcut: next,
          });
  },
  { immediate: true },
);

onMounted(() => {
  shortcutTarget.value = document;
});

type CommandPaletteDialogSetupExpose = Omit<CommandPaletteDialogExpose, "open"> & {
  readonly open: ComputedRef<boolean>;
};

const exposed = {
  open: computed(() => isOpen.value),
  setOpen,
  toggle,
} satisfies CommandPaletteDialogSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    data-vize-ui="command-palette-dialog"
    part="dialog"
    :data-state="isOpen ? 'open' : 'closed'"
    :data-shortcut="shortcut ?? undefined"
  >
    <DialogRoot :open="isOpen" @update:open="setOpen">
      <DialogPortal :to :disabled="portalDisabled">
        <DialogOverlay />
        <DialogContent :aria-label>
          <slot :open="isOpen" :close="close" />
        </DialogContent>
      </DialogPortal>
    </DialogRoot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
