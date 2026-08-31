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
  createDismissEvent,
  createEscapeKeyDownEvent,
  createFocusOutsideEvent,
  createPointerDownOutsideEvent,
  readBoolean,
  readBranches,
  readRoot,
  validateBranches,
  validateOptions,
} from "./dismissable-layer-internal.ts";
import {
  attachDismissableLayer,
  detachDismissableLayer,
  refreshDismissableLayer,
} from "./dismissable-layer-stack.ts";
import type { DismissableLayerToken } from "./dismissable-layer-stack.ts";
import type {
  DismissableLayerBranchProps,
  DismissableLayerController,
  DismissableLayerDismissReason,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerOptions,
  DismissableLayerProps,
} from "./dismissable-layer-types.ts";

const disposedDiagnostic = "VIZE_UI_DISMISSABLE_LAYER_DISPOSED";
const setupDiagnostic = "VIZE_UI_DISMISSABLE_LAYER_SETUP";

function capture(errors: unknown[], callback: () => void): void {
  try {
    callback();
  } catch (error) {
    errors.push(error);
  }
}

function surfaceErrors(errors: readonly unknown[], message: string): void {
  if (errors.length === 1) throw errors[0];
  if (errors.length < 2) return;
  throw new AggregateError(errors, message);
}

/** Create an SSR-safe, document-scoped dismissal layer for overlays and popups. */
export function createDismissableLayer(
  options: DismissableLayerOptions,
): DismissableLayerController {
  validateOptions(options);
  const activeState = shallowRef(false);
  const topLayerState = shallowRef(false);
  const registeredBranches = new Set<Element>();
  const token: DismissableLayerToken = {
    document: null,
    root: null,
    readBranches: () => readAllBranches(token.document),
    readEnabled: () => readBoolean(options.enabled, "enabled", true),
    readEscapeKey: () => readBoolean(options.escapeKey, "escapeKey", true),
    readOutsideFocus: () => readBoolean(options.outsideFocus, "outsideFocus", true),
    readOutsidePointerDown: () =>
      readBoolean(options.outsidePointerDown, "outsidePointerDown", true),
    handleEscapeKeyDown: (originalEvent, target) => {
      const event = createEscapeKeyDownEvent(originalEvent, target);
      const errors: unknown[] = [];
      capture(errors, () => options.onEscapeKeyDown?.(event));
      dismissIfAllowed(errors, event.defaultPrevented, "escape-key", originalEvent, target);
      surfaceErrors(errors, "Dismissable layer Escape callbacks failed");
    },
    handleFocusOutside: (originalEvent, target) => {
      const event = createFocusOutsideEvent(originalEvent, target);
      const errors: unknown[] = [];
      capture(errors, () => options.onFocusOutside?.(event));
      notifyOutside(errors, event);
      dismissIfAllowed(errors, event.defaultPrevented, "focus-outside", originalEvent, target);
      surfaceErrors(errors, "Dismissable layer focus callbacks failed");
    },
    handlePointerDownOutside: (originalEvent, target) => {
      const event = createPointerDownOutsideEvent(originalEvent, target);
      const errors: unknown[] = [];
      capture(errors, () => options.onPointerDownOutside?.(event));
      notifyOutside(errors, event);
      dismissIfAllowed(
        errors,
        event.defaultPrevented,
        "pointer-down-outside",
        originalEvent,
        target,
      );
      surfaceErrors(errors, "Dismissable layer pointer callbacks failed");
    },
    setTopLayer: (value) => {
      topLayerState.value = value;
    },
  };
  let disposed = false;

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const readAllBranches = (document: Document | null): readonly Element[] => {
    const reactiveBranches = readBranches(options.branches, document);
    const imperativeBranches = validateBranches([...registeredBranches], document);
    return [...new Set([...reactiveBranches, ...imperativeBranches])];
  };
  const notifyOutside = (errors: unknown[], event: DismissableLayerInteractOutsideEvent): void => {
    capture(errors, () => options.onInteractOutside?.(event));
  };
  const dismissIfAllowed = (
    errors: unknown[],
    defaultPrevented: boolean,
    reason: DismissableLayerDismissReason,
    originalEvent: Event,
    target: Element | null,
  ): void => {
    if (errors.length > 0 || defaultPrevented) return;
    capture(errors, () => options.onDismiss?.(createDismissEvent(reason, originalEvent, target)));
  };
  const refresh = (): void => {
    assertAlive();
    if (!activeState.value) return;
    const nextRoot = readRoot(options.root);
    readBoolean(options.enabled, "enabled", true);
    readBoolean(options.escapeKey, "escapeKey", true);
    readBoolean(options.outsideFocus, "outsideFocus", true);
    readBoolean(options.outsidePointerDown, "outsidePointerDown", true);
    readAllBranches(nextRoot?.ownerDocument ?? null);
    if (nextRoot !== token.root) {
      if (nextRoot) attachDismissableLayer(token, nextRoot);
      else detachDismissableLayer(token);
    } else refreshDismissableLayer(token);
  };
  const activate = (): void => {
    assertAlive();
    if (activeState.value) return;
    activeState.value = true;
    try {
      refresh();
    } catch (error) {
      activeState.value = false;
      detachDismissableLayer(token);
      throw error;
    }
  };
  const deactivate = (): void => {
    assertAlive();
    if (!activeState.value) return;
    try {
      detachDismissableLayer(token);
    } finally {
      activeState.value = false;
    }
  };
  const stopWatch = watch(
    () => [
      toValue(options.root),
      toValue(options.branches),
      toValue(options.enabled),
      toValue(options.escapeKey),
      toValue(options.outsideFocus),
      toValue(options.outsidePointerDown),
    ],
    () => {
      if (activeState.value) refresh();
    },
    { flush: "sync" },
  );
  const registerBranch = (branch: Element): (() => void) => {
    assertAlive();
    validateBranches([branch], token.document);
    registeredBranches.add(branch);
    if (activeState.value) refresh();
    let released = false;
    return () => {
      if (released) return;
      released = true;
      registeredBranches.delete(branch);
      if (!disposed && activeState.value) refresh();
    };
  };
  const layerProps: Readonly<DismissableLayerProps> = Object.freeze({
    "data-vize-dismissable-layer": "",
  });
  const branchProps: Readonly<DismissableLayerBranchProps> = Object.freeze({
    "data-vize-dismissable-branch": "",
  });

  return Object.freeze({
    isActive: shallowReadonly(activeState),
    isTopLayer: shallowReadonly(topLayerState),
    layerProps,
    branchProps,
    registerBranch,
    activate,
    deactivate,
    refresh,
    dispose: () => {
      if (disposed) return;
      try {
        detachDismissableLayer(token);
      } finally {
        activeState.value = false;
        disposed = true;
        registeredBranches.clear();
        stopWatch();
      }
    },
  });
}

/** Create, mount-activate, and scope-dispose a dismissable overlay layer. */
export function useDismissableLayer(options: DismissableLayerOptions): DismissableLayerController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createDismissableLayer(options);
  if (getCurrentInstance()) onMounted(controller.activate);
  else controller.activate();
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  DismissableLayerBranchProps,
  DismissableLayerController,
  DismissableLayerDismissEvent,
  DismissableLayerDismissReason,
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerFocusOutsideEvent,
  DismissableLayerInteractOutsideEvent,
  DismissableLayerOptions,
  DismissableLayerOutsideReason,
  DismissableLayerPointerDownOutsideEvent,
  DismissableLayerPointerType,
  DismissableLayerProps,
} from "./dismissable-layer-types.ts";
