<script setup lang="ts" generic="Item">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import AvatarGroupOverflow from "./avatar-group-overflow.vue";
import {
  avatarGroupSpacing,
  resolveAvatarGroupMessages,
  splitAvatarGroup,
} from "./avatar-group-state.ts";
import type { AvatarGroupSplit } from "./avatar-group-state.ts";
import type {
  AvatarGroupExpose,
  AvatarGroupItemSlotState,
  AvatarGroupMessageOverrides,
  AvatarGroupMessages,
  AvatarGroupOverflowSlotState,
  AvatarGroupSlotState,
  AvatarGroupState,
} from "./avatar-group-types.ts";

const {
  items,
  max = undefined,
  total = undefined,
  spacing = undefined,
  messages = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /** People rendered as avatar tiles, in order. @default required */
  readonly items: readonly Item[];

  /**
   * Maximum number of tiles, including the overflow tile. `undefined` never collapses.
   *
   * @default undefined
   */
  readonly max?: number;

  /**
   * Total number of people when `items` is a page of a larger set; the remainder
   * joins the overflow count.
   *
   * @default undefined
   */
  readonly total?: number;

  /**
   * Overlap or gap published as `--vize-ui-avatar-group-spacing`. Numbers are pixels.
   *
   * @default undefined
   */
  readonly spacing?: number | string;

  /**
   * Localized overflow strings.
   *
   * @default undefined
   */
  readonly messages?: AvatarGroupMessageOverrides | undefined;

  /**
   * Accessible list name, e.g. "Project members".
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the list.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** Renders one visible person, typically an `Avatar`. */
  item?(props: AvatarGroupItemSlotState<Item>): unknown;

  /** Replaces the default overflow tile. Receives the hidden items and count. */
  overflow?(props: AvatarGroupOverflowSlotState<Item>): unknown;
}>();

const element = useTemplateRef<HTMLUListElement>("element");
const split = computed<AvatarGroupSplit<Item>>(() => splitAvatarGroup(items, max, total));
const groupState = computed<AvatarGroupState>(() => split.value.state);
const visibleItems = computed<readonly Item[]>(() => split.value.visibleItems);
const hiddenItems = computed<readonly Item[]>(() => split.value.hiddenItems);
const overflowCount = computed<number>(() => split.value.overflowCount);
const visibleCount = computed<number>(() => split.value.visibleItems.length);
const resolved = computed<AvatarGroupMessages>(() => resolveAvatarGroupMessages(messages));
const spacingValue = computed<string | undefined>(() => avatarGroupSpacing(spacing));
const style = computed<string | undefined>(() =>
  spacingValue.value === undefined
    ? undefined
    : `--vize-ui-avatar-group-spacing: ${spacingValue.value}`,
);
const visibleEntries = computed<readonly AvatarGroupItemSlotState<Item>[]>(() =>
  visibleItems.value.map((item, index) => ({ item, index })),
);
const overflowState = computed<AvatarGroupOverflowSlotState<Item>>(() => ({
  count: split.value.overflowCount,
  hiddenItems: split.value.hiddenItems,
  label: resolved.value.overflow(split.value.overflowCount),
  text: resolved.value.overflowText(split.value.overflowCount),
}));

type AvatarGroupSetupExpose = Omit<
  AvatarGroupExpose<Item>,
  keyof AvatarGroupSlotState<Item> | "element"
> & {
  readonly element: typeof element;
  readonly hiddenItems: ComputedRef<readonly Item[]>;
  readonly overflowCount: ComputedRef<number>;
  readonly state: ComputedRef<AvatarGroupState>;
  readonly visibleItems: ComputedRef<readonly Item[]>;
};

const exposed = {
  element,
  hiddenItems,
  overflowCount,
  state: groupState,
  visibleItems,
} satisfies AvatarGroupSetupExpose;

defineExpose(exposed);
</script>

<template>
  <ul
    ref="element"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :style
    data-vize-ui="avatar-group"
    part="root"
    :data-state="groupState"
    :data-count="visibleCount"
    :data-overflow="overflowCount > 0 ? overflowCount : undefined"
  >
    <li
      v-for="entry in visibleEntries as readonly AvatarGroupItemSlotState<Item>[]"
      :key="entry.index"
      data-vize-ui="avatar-group-item"
      part="item"
      :data-index="entry.index"
    >
      <slot name="item" v-bind="entry" />
    </li>
    <li v-if="overflowCount > 0" data-vize-ui="avatar-group-overflow-item" part="overflow-item">
      <slot name="overflow" v-bind="overflowState">
        <AvatarGroupOverflow :count="overflowCount" :items="hiddenItems" :messages />
      </slot>
    </li>
  </ul>
</template>

<style scoped>
/* Headless by design. Consume --vize-ui-avatar-group-spacing, e.g.
   [data-vize-ui="avatar-group-item"] + * { margin-inline-start: var(--vize-ui-avatar-group-spacing); } */
</style>
