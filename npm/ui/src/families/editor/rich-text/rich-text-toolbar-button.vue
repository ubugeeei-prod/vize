<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { richTextContext, richTextToolbarContext } from "./rich-text-context.ts";
import type { RichTextCommand, RtState } from "./rich-text-state.ts";
import type { RichTextToolbarButtonSlotProps } from "./rich-text-types.ts";

const {
  command,
  active = undefined,
  ariaLabel = undefined,
  disabled = false,
} = defineProps<{
  /**
   * Command run on activation; the button disables itself while it cannot run.
   *
   * @default undefined
   */
  readonly command: RichTextCommand;

  /**
   * Toggle-state predicate; when provided the button publishes `aria-pressed`.
   *
   * @default undefined
   */
  readonly active?: (state: RtState) => boolean;

  /**
   * Accessible name when the content is an icon.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Force the disabled state.
   *
   * @default false
   */
  readonly disabled?: boolean;
}>();

defineSlots<{
  /** Button contents. Receives active and disabled state. */
  default?(props: RichTextToolbarButtonSlotProps): unknown;
}>();

const editor = richTextContext.use();
const toolbar = richTextToolbarContext.useOptional();
const element = useTemplateRef<HTMLButtonElement>("element");
const key = useDeterministicId({ hint: "rich-text-tool" });
const unavailable = computed(() => disabled || !editor.can(command));
const pressed = computed(() => (active ? active(editor.state.value) : null));
const registration = toolbar?.registry.register({
  key: key.value,
  value: null,
  element,
  disabled: () => unavailable.value,
});
onScopeDispose(() => registration?.unregister());

const tabStop = computed(() => {
  if (!toolbar) return true;
  const first = toolbar.registry.activeKey.value ?? toolbar.registry.navigableItems.value[0]?.key;
  return first === key.value;
});

function onClick(): void {
  if (unavailable.value) return;
  editor.run(command);
  editor.focus();
}

const buttonProps = computed(() => ({
  tabindex: tabStop.value ? 0 : -1,
  // Keep the editor selection: pressing a tool must not move focus first.
  onMousedown: (event: MouseEvent) => event.preventDefault(),
  onFocus: () => toolbar?.registry.setActiveKey(key.value),
  onClick,
}));
</script>

<template>
  <button
    ref="element"
    v-bind="buttonProps"
    type="button"
    :aria-label="ariaLabel"
    :aria-pressed="pressed === null ? undefined : pressed ? 'true' : 'false'"
    :aria-disabled="unavailable ? 'true' : undefined"
    data-vize-ui="rich-text-toolbar-button"
    part="toolbar-button"
    :data-active="pressed ? 'true' : undefined"
    :data-disabled="unavailable ? 'true' : undefined"
  >
    <slot :active="pressed === true" :disabled="unavailable" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
