/** Mouse and wheel input handling for focused Fresco surfaces. */

import { isRef, ref, watch, type Ref } from "vue";
import { lastMouseEvent } from "../app.js";
import type { MouseEvent } from "../protocol.js";

export interface UseMouseOptions {
  /** Whether the handler receives events. */
  isActive?: boolean | Ref<boolean>;
}

export interface UseMouseReturn {
  isActive: Ref<boolean>;
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
  const isActive = isRef(options.isActive) ? options.isActive : ref(options.isActive ?? true);

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
      isActive.value = true;
    },
    disable: () => {
      isActive.value = false;
    },
  };
}
