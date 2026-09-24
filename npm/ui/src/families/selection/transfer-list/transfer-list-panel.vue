<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { useCompositeNavigation } from "../../foundations/composite-navigation/composite-navigation.ts";
import { transferListContext, transferListPanelContext } from "./transfer-list-context.ts";
import type { TransferListSide } from "./transfer-list-model.ts";
import type { TransferListPanelSlotState } from "./transfer-list-types.ts";

const {
  side,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /** Which list this panel shows: available (`source`) or chosen (`target`) items. @default required */
  readonly side: TransferListSide;

  /**
   * Accessible name, e.g. "Available" or "Selected".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids of a visible panel heading.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /**
   * Render one `TransferListItem` per visible item. Values are untyped here;
   * the root slot exposes the same lists typed as `T`.
   */
  default?(props: TransferListPanelSlotState<unknown>): unknown;
}>();

/** Registered option data. */
interface PanelEntry {
  readonly id: ComputedRef<string>;
  readonly value: unknown;
}

const root = transferListContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const state = root.side(side);
const registry = createCollectionRegistry<string, PanelEntry>();
const navigation = useCompositeNavigation({
  registry,
  focusStrategy: "active-descendant",
  getItemId: (item) => item.value.id.value,
  isDisabled: root.disabled,
  typeahead: {},
});
const activeKey = computed(() => registry.activeKey.value);
const empty = computed(() => state.visible.value.length === 0);
const slotState = computed<TransferListPanelSlotState<unknown>>(() => ({
  checked: state.checked.value,
  items: state.visible.value,
  query: state.query.value,
  side,
  total: state.all.value.length,
}));
const containerProps = computed(() => navigation.getContainerProps());
const interactiveProps = computed(() => ({
  role: "listbox" as const,
  ...(root.disabled.value ? {} : { tabindex: 0 as const }),
  onFocus: (event: FocusEvent) => containerProps.value.onFocus(event),
  onKeydown,
}));

function focus(): void {
  element.value?.focus();
}

const release = root.registerPanel(side, focus);
onScopeDispose(release);

function activeValue(): { readonly found: boolean; readonly value: unknown } {
  const key = registry.activeKey.value;
  const item = key === null ? undefined : registry.getItem(key);
  return item === undefined
    ? { found: false, value: undefined }
    : { found: true, value: item.value.value };
}

function onKeydown(event: KeyboardEvent): void {
  if (root.disabled.value || event.defaultPrevented || event.target !== event.currentTarget) return;
  if (event.key === " ") {
    const active = activeValue();
    if (!active.found) return;
    event.preventDefault();
    root.toggleChecked(side, active.value);
    return;
  }
  if (event.key === "Enter") {
    const checked = state.checked.value;
    const active = activeValue();
    const values = checked.length > 0 ? checked : active.found ? [active.value] : [];
    if (values.length === 0) return;
    event.preventDefault();
    root.moveValues(side, values, event);
    return;
  }
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "a") {
    event.preventDefault();
    root.setChecked(side, state.visible.value);
    return;
  }
  const before = registry.activeKey.value;
  containerProps.value.onKeydown(event);
  const after = activeValue();
  if (
    event.shiftKey &&
    event.defaultPrevented &&
    after.found &&
    registry.activeKey.value !== before &&
    !root.isChecked(side, after.value)
  ) {
    root.toggleChecked(side, after.value);
  }
}

transferListPanelContext.provide({
  activate: (key) => {
    if (registry.navigableItems.value.some((item) => item.key === key)) registry.setActiveKey(key);
  },
  activeKey,
  empty,
  focus,
  query: state.query,
  register: ({ disabled, element: itemElement, id, textValue, value }) => {
    const registration = registry.register({
      disabled,
      element: itemElement,
      key: id.value,
      textValue,
      value: { id, value },
    });
    return () => {
      registration.unregister();
    };
  },
  side,
});
</script>

<template>
  <div
    :id="root.panelId(side)"
    ref="element"
    v-bind="interactiveProps"
    aria-multiselectable="true"
    :aria-activedescendant="
      root.disabled.value ? undefined : containerProps['aria-activedescendant']
    "
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-disabled="root.disabled.value ? 'true' : undefined"
    data-vize-ui="transfer-list-panel"
    part="panel"
    :data-side="side"
    :data-empty="empty ? 'true' : undefined"
    :data-count="state.all.value.length"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
