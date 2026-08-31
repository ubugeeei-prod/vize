import assert from "node:assert/strict";

import { nextTick } from "vue";

import type { FullscreenButtonController, FullscreenButtonOperation } from "./fullscreen-button.ts";

export interface FullscreenControllerRig {
  readonly controller: FullscreenButtonController;
  readonly requests: { readonly target: Element; readonly event: MouseEvent }[];
  readonly exits: MouseEvent[];
  readonly setFullscreenElement: (element: Element | null) => void;
}

export async function settle(): Promise<void> {
  await Promise.resolve();
  await nextTick();
}

export function createControllerRig(
  options: {
    readonly request?: (target: Element, event: MouseEvent) => void | Promise<void>;
    readonly exit?: (event: MouseEvent) => void | Promise<void>;
  } = {},
): FullscreenControllerRig {
  let fullscreenElement: Element | null = null;
  const requests: { readonly target: Element; readonly event: MouseEvent }[] = [];
  const exits: MouseEvent[] = [];
  const controller: FullscreenButtonController = {
    getFullscreenElement: () => fullscreenElement,
    async requestFullscreen(target, event) {
      requests.push({ event, target });
      await options.request?.(target, event);
      fullscreenElement = target;
    },
    async exitFullscreen(event) {
      exits.push(event);
      await options.exit?.(event);
      fullscreenElement = null;
    },
  };

  return {
    controller,
    exits,
    requests,
    setFullscreenElement: (element) => {
      fullscreenElement = element;
    },
  };
}

export function recordedOperation(
  recorded: readonly { readonly payload: readonly unknown[] }[],
  index: number,
): FullscreenButtonOperation {
  const operation = recorded[index]?.payload[0];
  assert.ok(typeof operation === "object" && operation !== null);
  return operation as FullscreenButtonOperation;
}
