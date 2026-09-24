<script setup lang="ts">
import { computed } from "vue";
import type { ComputedRef } from "vue";

import { landmarkContext, useLandmarkNavigation } from "./landmark-runtime.ts";
import type { LandmarkInfo, LandmarkKeyBinding, LandmarkProviderExpose } from "./landmark-types.ts";

const {
  discover = false,
  disabled = false,
  nextKey = undefined,
  previousKey = undefined,
} = defineProps<{
  /**
   * Also cycle through native and `[role]` landmarks inside the provider that
   * were not rendered by `Landmark`.
   *
   * @default false
   */
  readonly discover?: boolean;

  /**
   * Disable keyboard landmark cycling while keeping registrations.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Key that moves to the next landmark. `null` disables it; `undefined` uses F6.
   *
   * @default undefined
   */
  readonly nextKey?: LandmarkKeyBinding | null;

  /**
   * Key that moves to the previous landmark. `null` disables it; `undefined` uses Shift+F6.
   *
   * @default undefined
   */
  readonly previousKey?: LandmarkKeyBinding | null;
}>();

const emit = defineEmits<{
  /** Fired after keyboard or programmatic cycling focuses a landmark. */
  navigate: [landmark: LandmarkInfo, nativeEvent: KeyboardEvent | null];
}>();

defineSlots<{
  /** Page subtree containing Landmark components. */
  default(props: { readonly landmarks: readonly LandmarkInfo[] }): unknown;
}>();

const navigation = useLandmarkNavigation({
  discover: () => discover,
  enabled: () => !disabled,
  nextKey: () => nextKey,
  previousKey: () => previousKey,
  onNavigate: (landmark, event) => emit("navigate", landmark, event),
});
landmarkContext.provide(navigation);
const landmarks = computed(() => navigation.landmarks.value);

type LandmarkProviderSetupExpose = Omit<LandmarkProviderExpose, "landmarks"> & {
  readonly landmarks: ComputedRef<readonly LandmarkInfo[]>;
};

const exposed = {
  focusLandmark: navigation.focusLandmark,
  focusNext: () => navigation.focusNext(),
  focusPrevious: () => navigation.focusPrevious(),
  landmarks,
  refresh: navigation.refresh,
} satisfies LandmarkProviderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <slot :landmarks="landmarks" />
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
