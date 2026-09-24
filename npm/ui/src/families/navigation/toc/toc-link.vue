<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { tocContext, tocItemContext } from "./toc-context.ts";
import type { TocLinkSlotState } from "./toc-types.ts";

const { targetId } = defineProps<{
  /** Id of the heading or section this link points to. @default required */
  readonly targetId: string;
}>();

defineSlots<{
  /** Link text. Receives the target id and whether its section is in view. */
  default(props: TocLinkSlotState): unknown;
}>();

const context = tocContext.use();
const item = tocItemContext.useOptional();
const element = useTemplateRef<HTMLAnchorElement>("element");
const key = useDeterministicId({ hint: "toc-link" });
let registration: CollectionRegistration<string> | null = null;

watch(
  [key, () => targetId],
  ([nextKey, nextTarget]) => {
    registration?.unregister();
    registration = context.registerLink({ element, key: nextKey, targetId: nextTarget });
    item?.setTargetId(nextTarget);
  },
  { flush: "sync", immediate: true },
);
onScopeDispose(() => {
  registration?.unregister();
  registration = null;
  item?.setTargetId(null);
});

// The href is always an encoded same-document fragment.
const encodedTarget = computed(() => encodeURIComponent(targetId));
const active = computed(() => context.activeId.value === targetId);
const slotState = computed<TocLinkSlotState>(() => ({ active: active.value, targetId }));

function onClick(event: MouseEvent): void {
  context.navigate(targetId, event);
}

const exposed = {
  active,
  element,
} satisfies {
  readonly active: ComputedRef<boolean>;
  readonly element: typeof element;
};

defineExpose(exposed);
</script>

<template>
  <a
    ref="element"
    :href="`#${encodedTarget}`"
    :aria-current="active ? 'location' : undefined"
    data-vize-ui="toc-link"
    part="link"
    :data-active="active ? 'true' : undefined"
    :data-target="targetId"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </a>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
