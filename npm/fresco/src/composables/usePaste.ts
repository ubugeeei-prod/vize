/**
 * usePaste - bracketed paste handling.
 */

import { isRef, onUnmounted, ref, watch, type Ref } from "@vue/runtime-core";
import { lastPasteEvent } from "../app.js";
import { useStreamsContext } from "./useStreams.js";

let activePasteHandlerCount = 0;

export interface UsePasteOptions {
  /** Whether the paste handler is active */
  isActive?: boolean | Ref<boolean>;
}

export function hasActivePasteHandlers(): boolean {
  return activePasteHandlerCount > 0;
}

function toRef(value: boolean | Ref<boolean>): Ref<boolean> {
  return isRef(value) ? value : ref(value);
}

export function usePaste(handler: (text: string) => void, options: UsePasteOptions = {}) {
  const isActive = toRef(options.isActive ?? true);
  const streams = useStreamsContext();
  let rawModeEnabled = false;
  let bracketedPasteEnabled = false;
  let pasteHandlerRegistered = false;

  const syncRawMode = (isEnabled: boolean) => {
    if (rawModeEnabled === isEnabled) return;
    streams.setRawMode(isEnabled);
    rawModeEnabled = isEnabled;
  };

  const syncBracketedPasteMode = (isEnabled: boolean) => {
    if (bracketedPasteEnabled === isEnabled) return;
    streams.setBracketedPasteMode(isEnabled);
    bracketedPasteEnabled = isEnabled;
  };

  const syncPasteRegistration = (isEnabled: boolean) => {
    if (pasteHandlerRegistered === isEnabled) return;
    activePasteHandlerCount += isEnabled ? 1 : -1;
    pasteHandlerRegistered = isEnabled;
  };

  const syncActiveState = (isEnabled: boolean) => {
    syncRawMode(isEnabled);
    syncBracketedPasteMode(isEnabled);
    syncPasteRegistration(isEnabled);
  };

  watch(lastPasteEvent, (event) => {
    if (!event || !isActive.value) return;
    handler(event.text);
  });

  watch(isActive, syncActiveState, { immediate: true });
  onUnmounted(() => syncActiveState(false));

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
