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
import type { CSSProperties } from "vue";

import {
  containsComposed,
  focusableElements,
  focusElement,
  isUsableTarget,
} from "../focus-scope/focus-scope-dom.ts";
import { capture, documentOrder, surfaceErrors } from "../focus-scope/focus-scope-internal.ts";
import {
  createRedirectEvent,
  eventElement,
  readBoolean,
  readBranches,
  readRoot,
  readTarget,
  validateOptions,
} from "./focus-guards-internal.ts";
import {
  attachFocusGuards,
  detachFocusGuards,
  recomputeFocusGuards,
} from "./focus-guards-stack.ts";
import type { FocusGuardsToken } from "./focus-guards-stack.ts";
import type {
  FocusGuardDirection,
  FocusGuardPosition,
  FocusGuardProps,
  FocusGuardsController,
  FocusGuardsOptions,
} from "./focus-guards-types.ts";

const disposedDiagnostic = "VIZE_UI_FOCUS_GUARDS_DISPOSED";
const rootDiagnostic = "VIZE_UI_FOCUS_GUARDS_ROOT";
const setupDiagnostic = "VIZE_UI_FOCUS_GUARDS_SETUP";

/** Optional invisible sentinel preset; consumers remain free to replace every value. */
export const focusGuardPreset = Object.freeze({
  blockSize: "1px",
  borderWidth: "0",
  inlineSize: "1px",
  insetBlockStart: "0",
  insetInlineStart: "0",
  opacity: "0",
  outline: "none",
  padding: "0",
  pointerEvents: "none",
  position: "fixed",
} satisfies CSSProperties);

/** Create an SSR-safe pair of nested, portal-aware focus sentinels. */
export function createFocusGuards(options: FocusGuardsOptions): FocusGuardsController {
  validateOptions(options);
  const activeState = shallowRef(false);
  const guardingState = shallowRef(false);
  const token: FocusGuardsToken = {
    document: null,
    root: null,
    readEnabled: () => readBoolean(options.enabled, "enabled", true),
    setTopmost: (value) => {
      guardingState.value = value;
    },
  };
  let disposed = false;
  let redirecting = false;
  let tabDirection: FocusGuardDirection | null = null;
  let observer: MutationObserver | null = null;

  const assertAlive = (): void => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const roots = (): readonly Element[] => {
    const root = token.root;
    return root ? [root, ...readBranches(options.branches, root.ownerDocument)] : [];
  };
  const candidates = (): HTMLElement[] =>
    documentOrder(
      roots().flatMap((root) =>
        focusableElements(root, {
          accept: (element) =>
            !element.hasAttribute("data-vize-focus-guard") && options.accept?.(element) !== false,
        }),
      ),
    );
  const fallback = (): HTMLElement | null => {
    const ownedRoots = roots();
    const preferred = readTarget(options.fallbackFocus?.());
    if (preferred && !ownedRoots.some((root) => containsComposed(root, preferred))) {
      throw new Error(`${rootDiagnostic}: fallbackFocus must resolve inside a guarded region`);
    }
    if (preferred?.hasAttribute("data-vize-focus-guard")) return null;
    if (isUsableTarget(preferred)) return preferred;
    const root = token.root as HTMLElement | null;
    return root?.hasAttribute("tabindex") && isUsableTarget(root) ? root : null;
  };
  const owns = (element: Element | null): boolean =>
    element !== null && roots().some((root) => containsComposed(root, element));
  const onKeydown = (event: KeyboardEvent): void => {
    if (event.key === "Tab" && !event.altKey && !event.ctrlKey && !event.metaKey) {
      tabDirection = event.shiftKey ? "backward" : "forward";
    }
  };
  const redirect = (position: FocusGuardPosition, originalEvent: globalThis.FocusEvent): void => {
    if (!guardingState.value || redirecting) return;
    const related = eventElement(originalEvent.relatedTarget);
    const wrapped = owns(related);
    const direction =
      wrapped || related
        ? position === "before"
          ? wrapped
            ? "backward"
            : "forward"
          : wrapped
            ? "forward"
            : "backward"
        : (tabDirection ?? (position === "before" ? "forward" : "backward"));
    tabDirection = null;
    const values = candidates();
    const target = (direction === "forward" ? values[0] : values.at(-1)) ?? fallback();
    const event = createRedirectEvent(
      position,
      direction,
      wrapped ? "wrap" : "enter",
      target,
      related,
      originalEvent,
    );
    const errors: unknown[] = [];
    redirecting = true;
    try {
      capture(errors, () => options.onRedirect?.(event));
      if (!event.defaultPrevented && target) {
        capture(errors, () =>
          focusElement(target, readBoolean(options.preventScroll, "preventScroll", true)),
        );
      }
    } finally {
      redirecting = false;
    }
    surfaceErrors(errors, "Focus guard redirection failed");
  };
  const props = (position: FocusGuardPosition): Readonly<FocusGuardProps> =>
    Object.freeze({
      "data-vize-focus-guard": position,
      get tabindex() {
        return guardingState.value ? 0 : -1;
      },
      onFocus: (event: globalThis.FocusEvent) => redirect(position, event),
    });
  const beforeProps = props("before");
  const afterProps = props("after");

  const detachDom = (): void => {
    observer?.disconnect();
    observer = null;
    token.document?.removeEventListener("keydown", onKeydown, true);
    detachFocusGuards(token);
  };
  const observe = (root: Element): void => {
    const Observer = root.ownerDocument.defaultView?.MutationObserver;
    const documentRoot = root.ownerDocument.documentElement;
    if (!Observer || !documentRoot) return;
    observer = new Observer(() => recomputeFocusGuards(root.ownerDocument));
    observer.observe(documentRoot, { childList: true, subtree: true });
  };
  const refresh = (): void => {
    assertAlive();
    if (!activeState.value) return;
    const nextRoot = readRoot(options.root);
    readBoolean(options.enabled, "enabled", true);
    readBoolean(options.preventScroll, "preventScroll", true);
    readBranches(options.branches, nextRoot?.ownerDocument ?? null);
    if (nextRoot !== token.root) {
      const sameDocument = nextRoot !== null && token.document === nextRoot.ownerDocument;
      if (sameDocument) {
        attachFocusGuards(token, nextRoot);
      } else {
        detachDom();
      }
      if (nextRoot && !sameDocument) {
        attachFocusGuards(token, nextRoot);
        nextRoot.ownerDocument.addEventListener("keydown", onKeydown, true);
        observe(nextRoot);
      }
    } else if (token.document) recomputeFocusGuards(token.document);
  };
  const activate = (): void => {
    assertAlive();
    if (activeState.value) return;
    activeState.value = true;
    try {
      refresh();
    } catch (error) {
      activeState.value = false;
      detachDom();
      throw error;
    }
  };
  const deactivate = (): void => {
    assertAlive();
    if (!activeState.value) return;
    try {
      detachDom();
    } finally {
      activeState.value = false;
      tabDirection = null;
    }
  };
  const stopWatch = watch(
    () => [
      toValue(options.root),
      toValue(options.branches),
      toValue(options.enabled),
      toValue(options.preventScroll),
    ],
    () => {
      if (activeState.value) refresh();
    },
    { flush: "sync" },
  );

  return Object.freeze({
    isActive: shallowReadonly(activeState),
    isGuarding: shallowReadonly(guardingState),
    beforeProps,
    afterProps,
    activate,
    deactivate,
    refresh,
    dispose: () => {
      if (disposed) return;
      try {
        detachDom();
      } finally {
        activeState.value = false;
        disposed = true;
        stopWatch();
      }
    },
  });
}

/** Create, mount-activate, and scope-dispose a focus-guard pair. */
export function useFocusGuards(options: FocusGuardsOptions): FocusGuardsController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createFocusGuards(options);
  if (getCurrentInstance()) onMounted(controller.activate);
  else controller.activate();
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  FocusGuardDirection,
  FocusGuardPosition,
  FocusGuardProps,
  FocusGuardReason,
  FocusGuardRedirectEvent,
  FocusGuardsController,
  FocusGuardsOptions,
  FocusGuardStylePreset,
} from "./focus-guards-types.ts";
