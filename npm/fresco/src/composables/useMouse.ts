/** Mouse and wheel input handling for focused Fresco surfaces. */

import { computed, isRef, ref, watch, type Ref } from "vue";
import { lastMouseEvent } from "../app.js";
import type { MouseEvent } from "../protocol.js";

export interface UseMouseOptions {
  /** Whether the handler receives events. */
  isActive?: boolean | Readonly<Ref<boolean>>;
}

export interface UseMouseReturn {
  isActive: Readonly<Ref<boolean>>;
  /** Start delivering normalized mouse events. */
  enable: () => void;
  /** Stop delivering normalized mouse events. */
  disable: () => void;
}

/** Subscribe to normalized mouse actions, including wheel direction. */
export function useMouse(
  handler: (event: MouseEvent) => void,
  options: UseMouseOptions = {},
): UseMouseReturn {
  const externalIsActive = isRef(options.isActive) ? options.isActive : undefined;
  const locallyEnabled = ref(externalIsActive ? true : (options.isActive ?? true));
  const isActive = computed(() => locallyEnabled.value && (externalIsActive?.value ?? true));

  watch(
    lastMouseEvent,
    (event) => {
      if (event && isActive.value) handler(event);
    },
    { flush: "sync" },
  );

  return {
    isActive,
    enable: () => {
      locallyEnabled.value = true;
    },
    disable: () => {
      locallyEnabled.value = false;
    },
  };
}
