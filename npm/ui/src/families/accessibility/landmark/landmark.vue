<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import {
  focusLandmarkElement,
  isNamedLandmarkRole,
  landmarkContext,
  landmarkElements,
} from "./landmark-runtime.ts";
import type { LandmarkExpose, LandmarkRole, LandmarkSlotState } from "./landmark-types.ts";

const {
  role,
  id = undefined,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Landmark role. Renders the matching native element: `banner` → `header`,
   * `navigation` → `nav`, `main` → `main`, `complementary` → `aside`,
   * `contentinfo` → `footer`, `region` → `section`, `form` → `form`, and
   * `search` → `search`.
   *
   * @default required
   */
  readonly role: LandmarkRole;

  /**
   * Consumer-owned element id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Accessible name. `navigation`, `region`, `complementary`, `form`, and
   * `search` landmarks need a name (or `ariaLabelledby`) to be distinguishable.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids of elements that name the landmark.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** Landmark contents. Receives the role and cycling focus state. */
  default(props: LandmarkSlotState): unknown;
}>();

const context = landmarkContext.useOptional();
const element = useTemplateRef<HTMLElement>("element");
const elementRef = shallowRef<HTMLElement | null>(null);
const landmarkId = useDeterministicId({ id: () => id, hint: "landmark" });
const tag = computed(() => landmarkElements[role]);
const roleState = computed(() => role);
const focused = shallowRef(false);
const slotState = computed<LandmarkSlotState>(() => ({ focused: focused.value, role }));
const focusProps = computed(() => ({
  onBlur: () => {
    focused.value = false;
  },
  onFocus: (event: FocusEvent) => {
    focused.value = event.target === element.value;
  },
}));
let unregister: (() => void) | null = null;

if (
  isNamedLandmarkRole(role) &&
  !ariaLabel &&
  !ariaLabelledby &&
  (typeof process === "undefined" || process.env.NODE_ENV !== "production")
) {
  console.warn(
    `VIZE_UI_LANDMARK_NAME: a "${role}" landmark needs ariaLabel or ariaLabelledby to be distinguishable`,
  );
}

onMounted(() => {
  elementRef.value = element.value;
  if (context) {
    unregister = context.register({ element: elementRef, id: landmarkId.value, role });
  }
});

onScopeDispose(() => {
  unregister?.();
  unregister = null;
});

function focus(): boolean {
  return element.value ? focusLandmarkElement(element.value) : false;
}

type LandmarkSetupExpose = Omit<LandmarkExpose, "element" | "id"> & {
  readonly element: typeof element;
  readonly id: ComputedRef<string>;
  readonly role: ComputedRef<LandmarkRole>;
  readonly focused: Readonly<ShallowRef<boolean>>;
};

const exposed = {
  element,
  focus,
  focused,
  id: landmarkId,
  role: roleState,
} satisfies LandmarkSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="tag"
    :id="landmarkId"
    ref="element"
    v-bind="focusProps"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    data-vize-ui="landmark"
    part="root"
    :data-landmark="role"
    :data-focused="focused ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
