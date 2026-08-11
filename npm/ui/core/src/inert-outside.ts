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
  readBranches,
  readEnabled,
  readMode,
  readRoot,
  validateOptions,
} from "./inert-outside-internal.ts";
import { attachToken, detachToken, refreshToken } from "./inert-outside-stack.ts";
import type { InertOutsideToken } from "./inert-outside-stack.ts";
import type { InertOutsideController, InertOutsideOptions } from "./inert-outside-types.ts";

const disposedDiagnostic = "VIZE_UI_INERT_OUTSIDE_DISPOSED";
const setupDiagnostic = "VIZE_UI_INERT_OUTSIDE_SETUP";

/** Isolate rendered content outside reactive allowed roots without owning visual styling. */
export function createInertOutside(options: InertOutsideOptions): InertOutsideController {
  validateOptions(options);
  const activeState = shallowRef(false);
  const affectedState = shallowRef<readonly Element[]>(Object.freeze([]));
  const token: InertOutsideToken = {
    document: null,
    root: null,
    readBranches: () => readBranches(options.branches, token.document),
    readEnabled: () => readEnabled(options.enabled),
    readMode: () => readMode(options.mode),
    setAffected: (elements) => {
      affectedState.value = elements;
    },
  };
  let disposed = false;
  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const refresh = (): void => {
    assertAlive();
    if (!activeState.value) return;
    const nextRoot = readRoot(options.root);
    if (nextRoot !== token.root) {
      if (nextRoot) {
        readBranches(options.branches, nextRoot.ownerDocument);
        readEnabled(options.enabled);
        readMode(options.mode);
        attachToken(token, nextRoot);
      } else detachToken(token);
    } else {
      readBranches(options.branches, token.document);
      readEnabled(options.enabled);
      readMode(options.mode);
      refreshToken(token);
    }
  };
  const activate = (): void => {
    assertAlive();
    if (activeState.value) return;
    activeState.value = true;
    try {
      refresh();
    } catch (error) {
      activeState.value = false;
      detachToken(token);
      throw error;
    }
  };
  const deactivate = (): void => {
    assertAlive();
    if (!activeState.value) return;
    try {
      detachToken(token);
    } finally {
      activeState.value = false;
    }
  };
  const stopWatch = watch(
    () => {
      const root = toValue(options.root);
      const branches = toValue(options.branches) ?? [];
      return [
        root,
        Array.isArray(branches) ? [...branches] : branches,
        toValue(options.enabled),
        toValue(options.mode),
      ];
    },
    () => {
      if (activeState.value) refresh();
    },
    { flush: "sync" },
  );

  return Object.freeze({
    isActive: shallowReadonly(activeState),
    affectedElements: shallowReadonly(affectedState),
    activate,
    deactivate,
    refresh,
    dispose: () => {
      if (disposed) return;
      try {
        detachToken(token);
      } finally {
        activeState.value = false;
        disposed = true;
        stopWatch();
      }
    },
  });
}

/** Create, mount-activate, and scope-dispose an outside-inerting controller. */
export function useInertOutside(options: InertOutsideOptions): InertOutsideController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createInertOutside(options);
  if (getCurrentInstance()) onMounted(controller.activate);
  else controller.activate();
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  InertOutsideController,
  InertOutsideMode,
  InertOutsideOptions,
} from "./inert-outside-types.ts";
