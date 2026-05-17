/**
 * usePaste - bracketed paste handling.
 */

import { isRef, ref, watch, type Ref } from "@vue/runtime-core";
import { lastPasteEvent } from "../app.js";

export interface UsePasteOptions {
  /** Whether the paste handler is active */
  isActive?: boolean | Ref<boolean>;
}

function toRef(value: boolean | Ref<boolean>): Ref<boolean> {
  return isRef(value) ? value : ref(value);
}

export function usePaste(handler: (text: string) => void, options: UsePasteOptions = {}) {
  const isActive = toRef(options.isActive ?? true);

  watch(lastPasteEvent, (event) => {
    if (!event || !isActive.value) return;
    handler(event.text);
  });

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
