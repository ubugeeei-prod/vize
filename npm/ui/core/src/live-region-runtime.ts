import {
  getCurrentScope,
  nextTick,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
} from "vue";

import type {
  LiveRegionController,
  LiveRegionOptions,
  LiveRegionPoliteness,
} from "./live-region-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_LIVE_REGION_OPTION";
const disposedDiagnostic = "VIZE_UI_LIVE_REGION_DISPOSED";
const setupDiagnostic = "VIZE_UI_LIVE_REGION_SETUP";
const politenessValues = new Set<LiveRegionPoliteness>(["assertive", "polite"]);

function readPoliteness(
  value: LiveRegionOptions["politeness"],
  fallback: LiveRegionPoliteness,
): LiveRegionPoliteness {
  const resolved = toValue(value);
  if (resolved === undefined) return fallback;
  if (!politenessValues.has(resolved)) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: politeness must resolve to polite or assertive`,
    );
  }
  return resolved;
}

function validateOptions(options: LiveRegionOptions): void {
  if (typeof options.politeness !== "function") readPoliteness(options.politeness, "polite");
}

/** Create an SSR-safe announcement queue for one live region. */
export function createLiveRegion(options: LiveRegionOptions = {}): LiveRegionController {
  validateOptions(options);
  const message = shallowRef("");
  const politeness = shallowRef(readPoliteness(options.politeness, "polite"));
  let disposed = false;
  let token = 0;

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  return Object.freeze({
    message: shallowReadonly(message),
    politeness: shallowReadonly(politeness),
    announce: (text: string, nextPoliteness?: LiveRegionPoliteness) => {
      assertAlive();
      if (typeof text !== "string") {
        throw new TypeError(`${invalidOptionDiagnostic}: announcement text must be a string`);
      }
      if (nextPoliteness !== undefined) politeness.value = readPoliteness(nextPoliteness, "polite");
      else politeness.value = readPoliteness(options.politeness, "polite");
      const generation = ++token;
      message.value = "";
      void nextTick(() => {
        if (disposed || generation !== token) return;
        message.value = text;
      });
    },
    clear: () => {
      assertAlive();
      token += 1;
      message.value = "";
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      token += 1;
      message.value = "";
    },
  });
}

/** Create a live-region announcer disposed with the current Vue effect scope. */
export function useLiveRegion(options: LiveRegionOptions = {}): LiveRegionController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createLiveRegion(options);
  onScopeDispose(controller.dispose);
  return controller;
}
