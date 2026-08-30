import { getCurrentScope, onScopeDispose, toValue, watch } from "vue";

import { createPresence } from "./families/overlays/presence/presence-runtime.ts";
import type { TransitionController, TransitionOptions } from "./transition-types.ts";

const invalidOptionDiagnostic = "VIZE_UI_TRANSITION_OPTION";
const disposedDiagnostic = "VIZE_UI_TRANSITION_DISPOSED";
const setupDiagnostic = "VIZE_UI_TRANSITION_SETUP";

function readPadding(value: TransitionOptions["timeoutPadding"]): number {
  const resolved = toValue(value);
  if (resolved === undefined) return 0;
  if (typeof resolved !== "number" || !Number.isFinite(resolved) || resolved < 0) {
    throw new TypeError(
      `${invalidOptionDiagnostic}: timeoutPadding must resolve to a non-negative number`,
    );
  }
  return resolved;
}

function parseTimes(value: string): readonly number[] {
  return value.split(",").map((part) => {
    const trimmed = part.trim();
    if (trimmed.endsWith("ms")) return Number.parseFloat(trimmed) || 0;
    if (trimmed.endsWith("s")) return (Number.parseFloat(trimmed) || 0) * 1_000;
    return 0;
  });
}

function pairedMax(durations: readonly number[], delays: readonly number[]): number {
  const count = Math.max(durations.length, delays.length, 1);
  let max = 0;
  for (let index = 0; index < count; index += 1) {
    const duration = durations[index] ?? durations[0] ?? 0;
    const delay = delays[index] ?? delays[0] ?? 0;
    max = Math.max(max, duration + delay);
  }
  return max;
}

/** Longest animation or transition time on `element`, including delay. */
export function motionDurationMs(element: Element): number {
  if (typeof globalThis.getComputedStyle !== "function") return 0;
  const style = globalThis.getComputedStyle(element);
  return Math.max(
    pairedMax(parseTimes(style.transitionDuration), parseTimes(style.transitionDelay)),
    pairedMax(parseTimes(style.animationDuration), parseTimes(style.animationDelay)),
  );
}

function validateOptions(options: TransitionOptions): void {
  if (typeof options.timeoutPadding !== "function") readPadding(options.timeoutPadding);
}

/** Create an SSR-safe transition that auto-completes from CSS motion duration. */
export function createTransition(options: TransitionOptions = {}): TransitionController {
  validateOptions(options);
  const presence = createPresence(options);
  let disposed = false;
  let element: Element | null = null;
  let timer = 0;

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };

  const clearTimer = (): void => {
    if (timer === 0) return;
    globalThis.clearTimeout(timer);
    timer = 0;
  };

  const schedule = (): void => {
    clearTimer();
    const status = presence.status.value;
    if (status !== "entering" && status !== "exiting") return;
    const duration =
      (element === null ? 0 : motionDurationMs(element)) + readPadding(options.timeoutPadding);
    timer = globalThis.setTimeout(() => {
      timer = 0;
      if (!disposed) presence.completeAnimation();
    }, duration) as unknown as number;
  };

  const stopWatch = watch(() => presence.status.value, schedule, { flush: "sync" });

  return Object.freeze({
    completeAnimation: () => {
      assertAlive();
      clearTimer();
      presence.completeAnimation();
    },
    dispose: () => {
      if (disposed) return;
      disposed = true;
      clearTimer();
      stopWatch();
      presence.dispose();
    },
    isPresent: presence.isPresent,
    presenceProps: presence.presenceProps,
    setElement: (node: Element | null) => {
      assertAlive();
      element = node;
      schedule();
    },
    status: presence.status,
  });
}

/** Create a transition controller disposed with the current Vue effect scope. */
export function useTransition(options: TransitionOptions = {}): TransitionController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createTransition(options);
  onScopeDispose(controller.dispose);
  return controller;
}
