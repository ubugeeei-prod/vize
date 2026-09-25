<script setup lang="ts" generic="Value = undefined">
import { computed, onMounted, onScopeDispose, onUpdated, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { commandPaletteContext, commandPaletteGroupContext } from "./command-palette-context.ts";
import type {
  CommandPaletteItemExpose,
  CommandPaletteItemSlotState,
  CommandPaletteItemState,
} from "./command-palette-types.ts";

const {
  value = undefined,
  textValue = undefined,
  keywords = [],
  command = undefined,
  shortcut = undefined,
  disabled = false,
  forceMount = false,
} = defineProps<{
  /**
   * Consumer data handed back through the `select` event.
   *
   * @default undefined
   */
  readonly value?: Value;

  /**
   * Searchable label. `undefined` uses the router command title, then the
   * rendered text after mount.
   *
   * @default undefined
   */
  readonly textValue?: string;

  /**
   * Extra search terms. Router command keywords are appended.
   *
   * @default []
   */
  readonly keywords?: readonly string[];

  /**
   * Router command id run with source `"palette"` when the item is selected.
   *
   * @default undefined
   */
  readonly command?: string;

  /**
   * Keyboard shortcut advertised through `aria-keyshortcuts`, for example `"Control+K"`.
   *
   * @default undefined
   */
  readonly shortcut?: string;

  /**
   * Refuse selection and skip the item during keyboard navigation.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Keep the item visible whatever the search.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the item is selected by click, Enter, or `select()`. */
  select: [value: Value | undefined, nativeEvent: Event | null];
}>();

defineSlots<{
  /** Item contents. Receives active, disabled, and label state. */
  default?(props: CommandPaletteItemSlotState): unknown;
}>();

const context = commandPaletteContext.use();
const group = commandPaletteGroupContext.useOptional();
const itemId = useDeterministicId({ hint: "command-palette-item" });
const element = useTemplateRef<HTMLDivElement>("element");
const elementRef = shallowRef<HTMLDivElement | null>(null);
const renderedText = shallowRef("");
const commandInfo = computed(() =>
  command === undefined ? undefined : context.findCommand(command),
);
const resolvedText = computed(() => textValue ?? commandInfo.value?.title ?? renderedText.value);
const resolvedKeywords = computed<readonly string[]>(() =>
  keywords.concat(commandInfo.value?.keywords ?? []),
);
const disabledState = computed(
  () => disabled || (commandInfo.value !== undefined && !commandInfo.value.isEnabled()),
);
const visible = computed(() => context.isVisible(itemId.value));
const active = computed(() => context.activeItemId.value === itemId.value);
const state = computed<CommandPaletteItemState>(() => {
  if (disabledState.value) return "disabled";
  return active.value ? "active" : "inactive";
});
const shortcutValue = computed(() => shortcut ?? null);
const slotState = computed<CommandPaletteItemSlotState>(() => ({
  active: active.value,
  disabled: disabledState.value,
  shortcut: shortcutValue.value,
  state: state.value,
  textValue: resolvedText.value,
}));

function select(nativeEvent: Event | null = null): boolean {
  if (disabledState.value || !visible.value) return false;
  if (command !== undefined) context.runCommand(command);
  emit("select", value, nativeEvent);
  context.didSelect(command ?? null, nativeEvent);
  return true;
}

const registration = context.registerItem({
  disabled: () => disabledState.value,
  element: elementRef,
  forceMount: () => forceMount,
  id: itemId.value,
  keywords: () => resolvedKeywords.value,
  select,
  textValue: () => resolvedText.value,
});
const releaseGroup = group?.addMember(itemId.value);

function syncElement(): void {
  elementRef.value = element.value;
  if (textValue === undefined && commandInfo.value?.title == null) {
    renderedText.value = element.value?.textContent?.trim() ?? "";
  }
}

onMounted(syncElement);
onUpdated(syncElement);
onScopeDispose(() => {
  registration.unregister();
  releaseGroup?.();
});

function onClick(event: MouseEvent): void {
  select(event);
}

function onPointerdown(event: PointerEvent): void {
  context.navigation.getItemProps(itemId.value).onPointerdown(event);
}

function onPointermove(): void {
  if (!disabledState.value && !active.value) context.setActive(itemId.value);
}

function onMousedown(event: MouseEvent): void {
  // Keep focus in the combobox input while pointing at options.
  event.preventDefault();
}

type CommandPaletteItemSetupExpose = Omit<
  CommandPaletteItemExpose,
  "active" | "disabled" | "element" | "id" | "shortcut" | "state" | "textValue" | "visible"
> & {
  readonly active: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly shortcut: ComputedRef<string | null>;
  readonly state: ComputedRef<CommandPaletteItemState>;
  readonly textValue: ComputedRef<string>;
  readonly visible: ComputedRef<boolean>;
};

const exposed = {
  active,
  disabled: disabledState,
  element,
  id: itemId,
  select,
  shortcut: shortcutValue,
  state,
  textValue: resolvedText,
  visible,
} satisfies CommandPaletteItemSetupExpose;

defineExpose(exposed);
// Options are reached through aria-activedescendant on the input, so they are
// intentionally not focusable; keyboard selection is owned by the input.
const optionProps = {
  role: "option",
  onClick,
  onMousedown,
  onPointerdown,
  onPointermove,
} as const;
</script>

<template>
  <div
    :id="itemId"
    ref="element"
    v-bind="optionProps"
    :aria-selected="active ? 'true' : 'false'"
    :aria-disabled="disabledState ? 'true' : undefined"
    :aria-keyshortcuts="shortcut"
    :hidden="visible ? undefined : true"
    data-vize-ui="command-palette-item"
    part="item"
    :data-state="state"
    :data-command="command"
  >
    <slot v-bind="slotState">{{ resolvedText }}</slot>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
