/**
 * usePaste - bracketed paste handling.
 */

import { isRef, onUnmounted, ref, watch, type Ref } from "@vue/runtime-core";
import { lastPasteEvent } from "../app.js";
import { useStreamsContext } from "./useStreams.js";

export interface UsePasteOptions {
  /** Whether the paste handler is active */
  isActive?: boolean | Ref<boolean>;
}

function toRef(value: boolean | Ref<boolean>): Ref<boolean> {
  return isRef(value) ? value : ref(value);
}

export function usePaste(handler: (text: string) => void, options: UsePasteOptions = {}) {
  const isActive = toRef(options.isActive ?? true);
  const streams = useStreamsContext();
  let bracketedPasteEnabled = false;

  const syncBracketedPasteMode = (isEnabled: boolean) => {
    if (bracketedPasteEnabled === isEnabled) return;
    streams.setBracketedPasteMode(isEnabled);
    bracketedPasteEnabled = isEnabled;
  };

  watch(lastPasteEvent, (event) => {
    if (!event || !isActive.value) return;
    handler(event.text);
  });

  watch(isActive, syncBracketedPasteMode, { immediate: true });
  onUnmounted(() => syncBracketedPasteMode(false));

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
