<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { useResolvedDirection } from "../../i18n/direction/direction-runtime.ts";
import { useMenuRoot } from "../../menus/menu/menu-root-runtime.ts";
import type { MenuRootSetupExpose } from "../../menus/menu/menu-root-runtime.ts";
import type { MenuDirection } from "../../menus/menu/menu-types.ts";
import { splitButtonContext } from "./split-button-context.ts";
import type { SplitButtonContextValue } from "./split-button-context.ts";
import type { SplitButtonAnchor, SplitButtonSlotState } from "./split-button-types.ts";

const {
  id = undefined,
  open = undefined,
  defaultOpen = false,
  modal = true,
  dir = undefined,
  loop = false,
  disabled = false,
  anchor = "group",
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
} = defineProps<{
  /**
   * Consumer-owned base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled open state of the menu. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly open?: boolean;

  /**
   * Initial open state for uncontrolled use.
   *
   * @default false
   */
  readonly defaultOpen?: boolean;

  /**
   * Make outside content inert and lock document scroll while the menu is open.
   *
   * @default true
   */
  readonly modal?: boolean;

  /**
   * Reading direction: flips submenu arrow keys and submenu placement. `undefined` inherits `DirectionProvider`/`LocaleProvider`, then `"ltr"`.
   *
   * @default undefined
   */
  readonly dir?: MenuDirection;

  /**
   * Wrap arrow-key navigation from the last item to the first and back.
   *
   * @default false
   */
  readonly loop?: boolean;

  /**
   * Disable the primary action and the trigger, and close the menu when it becomes disabled.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Element the menu is positioned against: the whole group or only the trigger.
   *
   * @default "group"
   */
  readonly anchor?: SplitButtonAnchor;

  /**
   * Accessible name of the group when no `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the group.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the group.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}>();

const emit = defineEmits<{
  /** Fired with the requested open value (supports `v-model:open`). */
  "update:open": [value: boolean];
  /** Fired after any distinct open-state request with the previous value and native event. */
  openChange: [value: boolean, previous: boolean, nativeEvent: Event | null];
}>();

const dirState = useResolvedDirection(() => dir);

defineSlots<{
  /** Primary action, trigger, and content. Receives the menu open state and availability. */
  default(props: SplitButtonSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const disabledState = computed(() => disabled);
// Element that requested the current open state, when it was not the trigger.
const restoreElement = shallowRef<HTMLElement | null>(null);
let pendingOrigin: HTMLElement | null = null;

const menu = useMenuRoot({
  kind: "split-button",
  hint: "split-button",
  id: () => id,
  open: () => open,
  defaultOpen: () => defaultOpen,
  modal: () => modal,
  dir: () => dirState.value,
  loop: () => loop,
  disabled: () => disabled,
  reference: () => (anchor === "group" ? element.value : menu.level.triggerElement.value),
  restoreTarget: () => restoreElement.value ?? menu.level.triggerElement.value,
  onOpenChange: (value, previous, event) => {
    if (value) restoreElement.value = pendingOrigin;
    emit("update:open", value);
    emit("openChange", value, previous, event);
  },
});
const level = menu.level;

function openFromAction(origin: HTMLElement | null, event: Event): boolean {
  pendingOrigin = origin;
  try {
    return level.setOpen(true, event, "first");
  } finally {
    pendingOrigin = null;
  }
}

splitButtonContext.provide({
  disabled: disabledState,
  openFromAction,
} satisfies SplitButtonContextValue);
const slotState = computed<SplitButtonSlotState>(() => ({
  ...menu.slotState.value,
  disabled: disabledState.value,
}));

defineExpose({
  ...menu.expose,
  disabled: disabledState,
  element,
} satisfies MenuRootSetupExpose & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
});
</script>

<template>
  <div
    ref="element"
    role="group"
    :aria-label
    :aria-labelledby
    :aria-describedby
    :aria-disabled="disabledState ? 'true' : undefined"
    data-vize-ui="split-button"
    part="root"
    :data-state="level.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
