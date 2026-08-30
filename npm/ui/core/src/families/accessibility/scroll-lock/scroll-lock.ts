import {
  getCurrentInstance,
  getCurrentScope,
  onMounted,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
  watch,
} from "vue";

import {
  readBoolean,
  readDocument,
  readStrategy,
  validateScrollLockOptions,
} from "./scroll-lock-internal.ts";
import { attachScrollLock, detachScrollLock, refreshScrollLock } from "./scroll-lock-stack.ts";
import type { ScrollLockToken } from "./scroll-lock-stack.ts";
import type { ScrollLockController, ScrollLockOptions } from "./scroll-lock-types.ts";

const disposedDiagnostic = "VIZE_UI_SCROLL_LOCK_DISPOSED";
const setupDiagnostic = "VIZE_UI_SCROLL_LOCK_SETUP";

/** Lock a reactive document viewport without suppressing browser zoom gestures. */
export function createScrollLock(options: ScrollLockOptions): ScrollLockController {
  validateScrollLockOptions(options);
  const activeState = shallowRef(false);
  const lockedState = shallowRef(false);
  const gapState = shallowRef(0);
  const strategyState = shallowRef<"fixed" | "overflow" | null>(null);
  const token: ScrollLockToken = {
    document: null,
    readEnabled: () => readBoolean(options.enabled, "enabled", true),
    readPreserveGap: () => readBoolean(options.preserveScrollbarGap, "preserveScrollbarGap", true),
    readRestoreScroll: () => readBoolean(options.restoreScroll, "restoreScroll", true),
    readStrategy: () => readStrategy(options.strategy),
    setState: (locked, gap, strategy) => {
      lockedState.value = locked;
      gapState.value = gap;
      strategyState.value = strategy;
    },
  };
  let disposed = false;
  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const refresh = (): void => {
    assertAlive();
    if (!activeState.value) return;
    const document = readDocument(options.document);
    readBoolean(options.enabled, "enabled", true);
    readBoolean(options.preserveScrollbarGap, "preserveScrollbarGap", true);
    readBoolean(options.restoreScroll, "restoreScroll", true);
    readStrategy(options.strategy);
    if (document && document !== token.document) attachScrollLock(token, document);
    else if (!document) detachScrollLock(token);
    else refreshScrollLock(token);
  };
  const activate = (): void => {
    assertAlive();
    if (activeState.value) return;
    activeState.value = true;
    try {
      refresh();
    } catch (error) {
      activeState.value = false;
      detachScrollLock(token);
      throw error;
    }
  };
  const deactivate = (): void => {
    assertAlive();
    if (!activeState.value) return;
    try {
      detachScrollLock(token);
    } finally {
      activeState.value = false;
    }
  };
  const stopWatch = watch(
    () => [
      toValue(options.document),
      toValue(options.enabled),
      toValue(options.preserveScrollbarGap),
      toValue(options.restoreScroll),
      toValue(options.strategy),
    ],
    () => {
      if (activeState.value) refresh();
    },
    { flush: "sync" },
  );

  return Object.freeze({
    isActive: shallowReadonly(activeState),
    isLocked: shallowReadonly(lockedState),
    scrollbarGap: shallowReadonly(gapState),
    resolvedStrategy: shallowReadonly(strategyState),
    activate,
    deactivate,
    refresh,
    dispose: () => {
      if (disposed) return;
      try {
        detachScrollLock(token);
      } finally {
        activeState.value = false;
        disposed = true;
        stopWatch();
      }
    },
  });
}

/** Create, mount-activate, and scope-dispose a document scroll lock. */
export function useScrollLock(options: ScrollLockOptions): ScrollLockController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createScrollLock(options);
  if (getCurrentInstance()) onMounted(controller.activate);
  else controller.activate();
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  ScrollLockController,
  ScrollLockOptions,
  ScrollLockStrategy,
} from "./scroll-lock-types.ts";
