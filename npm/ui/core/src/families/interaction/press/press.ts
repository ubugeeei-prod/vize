import { getCurrentScope, onScopeDispose } from "vue";

import { createPressHandlers } from "./press-handlers.ts";
import { PressLifecycle } from "./press-lifecycle.ts";
import type { PressController, PressOptions } from "./press-types.ts";

const setupDiagnostic = "VIZE_UI_PRESS_SETUP";

/**
 * Create an SSR-safe press normalizer for one host element.
 *
 * Spread the returned `pressProps` onto the host and call `dispose` when using
 * this factory outside a Vue effect scope. No DOM global is read at setup.
 */
export function createPress(options: PressOptions = {}): PressController {
  let handlers!: ReturnType<typeof createPressHandlers>;
  const lifecycle = new PressLifecycle(options, (document, source) =>
    handlers.installListeners(document, source),
  );
  handlers = createPressHandlers(lifecycle);
  const { installListeners: _, ...pressProps } = handlers;
  return lifecycle.toController(Object.freeze(pressProps));
}

/** Create a press normalizer disposed with the current Vue effect scope. */
export function usePress(options: PressOptions = {}): PressController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createPress(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  PressController,
  PressEvent,
  PressEventType,
  PressKeyboardBehavior,
  PressOptions,
  PressPointerType,
  PressProps,
} from "./press-types.ts";
