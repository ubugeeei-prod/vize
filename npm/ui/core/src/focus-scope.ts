import { shallowReadonly, shallowRef, toValue, watch } from "vue";

import {
  containsComposed,
  createAutoFocusEvent,
  deepActiveElement,
  focusableElements,
  focusElement,
  isUsableTarget,
} from "./focus-scope-dom.ts";
import {
  capture,
  documentOrder,
  readBoolean,
  readRoot,
  readTarget,
  resolveMoveOptions,
  surfaceErrors,
  validateOptions,
} from "./focus-scope-internal.ts";
import { createFocusScopeManager } from "./focus-scope-manager.ts";
import {
  attachScope,
  containmentOwner,
  detachScope,
  parentScope,
  rootsOwnedBy,
} from "./focus-scope-stack.ts";
import type { FocusScopeToken } from "./focus-scope-stack.ts";
import type {
  FocusScopeController,
  FocusScopeMoveOptions,
  FocusScopeOptions,
} from "./focus-scope-types.ts";

const disposedDiagnostic = "VIZE_UI_FOCUS_SCOPE_DISPOSED";
const rootDiagnostic = "VIZE_UI_FOCUS_SCOPE_ROOT";

/** Create an SSR-safe focus scope with containment, traversal, and restoration. */
export function createFocusScope(options: FocusScopeOptions): FocusScopeController {
  validateOptions(options);
  const activeState = shallowRef(false);
  const token: FocusScopeToken = {
    document: null,
    root: null,
    lastFocused: null,
    readContain: () => readBoolean(options.contain, "contain"),
  };
  let disposed = false;
  let observer: MutationObserver | null = null;
  let capturedTarget: HTMLElement | null = null;
  let restoreCandidates: HTMLElement[] = [];
  let restoreIndex = -1;
  let restorationCaptured = false;
  let entryHandled = false;
  let guarding = false;

  const assertAlive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the controller has been disposed`);
  };
  const root = (): Element | null => token.root ?? readRoot(options.root);
  const accepted = (element: HTMLElement, local?: (element: HTMLElement) => boolean): boolean =>
    options.accept?.(element) !== false && local?.(element) !== false;
  const list = (scopeRoot: Element, values?: FocusScopeMoveOptions): HTMLElement[] => {
    const resolved = resolveMoveOptions(values);
    return focusableElements(scopeRoot, {
      includeProgrammatic: resolved.includeProgrammatic,
      accept: (element) => accepted(element, resolved.accept),
    });
  };
  const focusFallback = (scopeRoot: Element): HTMLElement | null => {
    const candidate = readTarget(options.fallbackFocus?.(), "fallbackFocus");
    if (candidate && !containsComposed(scopeRoot, candidate)) {
      throw new Error(`${rootDiagnostic}: fallbackFocus must resolve inside the scope`);
    }
    if (isUsableTarget(candidate)) return candidate;
    const htmlRoot = scopeRoot as HTMLElement;
    return htmlRoot.namespaceURI === "http://www.w3.org/1999/xhtml" &&
      typeof htmlRoot.focus === "function" &&
      htmlRoot.hasAttribute("tabindex") &&
      isUsableTarget(htmlRoot)
      ? htmlRoot
      : null;
  };
  const manager = createFocusScopeManager({ assertAlive, list, root });

  const ownedCandidates = (): HTMLElement[] =>
    documentOrder(rootsOwnedBy(token).flatMap((ownedRoot) => list(ownedRoot)));
  const recover = (): void => {
    if (guarding || token.document === null || containmentOwner(token.document) !== token) return;
    const roots = rootsOwnedBy(token);
    const active = deepActiveElement(token.document);
    if (isUsableTarget(active) && roots.some((ownedRoot) => containsComposed(ownedRoot, active))) {
      return;
    }
    const candidates = ownedCandidates();
    const target =
      (token.lastFocused &&
      roots.some((ownedRoot) => containsComposed(ownedRoot, token.lastFocused)) &&
      isUsableTarget(token.lastFocused)
        ? token.lastFocused
        : candidates[0]) ?? (token.root ? focusFallback(token.root) : null);
    if (!target) return;
    guarding = true;
    try {
      focusElement(target);
    } finally {
      guarding = false;
    }
  };
  const onFocusin = (event: FocusEvent): void => {
    if (token.document === null || containmentOwner(token.document) !== token) return;
    const target = (event.composedPath?.()[0] ?? event.target) as HTMLElement | null;
    if (
      isUsableTarget(target) &&
      rootsOwnedBy(token).some((ownedRoot) => containsComposed(ownedRoot, target))
    ) {
      token.lastFocused = target;
      return;
    }
    recover();
  };
  const onKeydown = (event: KeyboardEvent): void => {
    if (
      event.key !== "Tab" ||
      event.altKey ||
      event.ctrlKey ||
      event.metaKey ||
      event.defaultPrevented ||
      token.document === null ||
      containmentOwner(token.document) !== token
    )
      return;
    const candidates = ownedCandidates();
    const current = deepActiveElement(token.document);
    const index = current ? candidates.indexOf(current) : -1;
    const target = event.shiftKey
      ? index <= 0
        ? candidates.at(-1)
        : undefined
      : index < 0 || index === candidates.length - 1
        ? candidates[0]
        : undefined;
    if (target) {
      event.preventDefault();
      focusElement(target);
    } else if (candidates.length === 0 && token.root) {
      event.preventDefault();
      const fallback = focusFallback(token.root);
      if (fallback) focusElement(fallback);
    }
  };

  const detachDom = (): void => {
    observer?.disconnect();
    observer = null;
    token.document?.removeEventListener("focusin", onFocusin, true);
    token.document?.removeEventListener("keydown", onKeydown, true);
    detachScope(token);
  };
  const observe = (scopeRoot: Element): void => {
    observer?.disconnect();
    observer = null;
    const Observer = scopeRoot.ownerDocument.defaultView?.MutationObserver;
    if (!Observer) return;
    observer = new Observer(recover);
    observer.observe(scopeRoot, {
      subtree: true,
      childList: true,
      attributes: true,
      attributeFilter: [
        "aria-hidden",
        "class",
        "contenteditable",
        "controls",
        "disabled",
        "hidden",
        "href",
        "inert",
        "open",
        "style",
        "tabindex",
        "type",
      ],
    });
    const documentRoot = scopeRoot.ownerDocument.documentElement;
    if (documentRoot && documentRoot !== scopeRoot) {
      observer.observe(documentRoot, { childList: true, subtree: true });
    }
  };
  const captureRestoration = (scopeRoot: Element): void => {
    if (restorationCaptured) return;
    restorationCaptured = true;
    capturedTarget = deepActiveElement(scopeRoot.ownerDocument);
    if (!readBoolean(options.restoreFocus, "restoreFocus")) return;
    const body = scopeRoot.ownerDocument.body;
    restoreCandidates = body ? focusableElements(body) : [];
    restoreIndex = capturedTarget ? restoreCandidates.indexOf(capturedTarget) : -1;
  };
  const enter = (scopeRoot: Element, errors: unknown[]): void => {
    if (entryHandled || !readBoolean(options.autoFocus, "autoFocus")) return;
    entryHandled = true;
    let target = readTarget(options.initialFocus?.(), "initialFocus");
    if (target && !containsComposed(scopeRoot, target)) {
      throw new Error(`${rootDiagnostic}: initialFocus must resolve inside the scope`);
    }
    if (!isUsableTarget(target)) target = null;
    target =
      target ?? list(scopeRoot, { includeProgrammatic: true })[0] ?? focusFallback(scopeRoot);
    const event = createAutoFocusEvent("mount", target);
    capture(errors, () => options.onMountAutoFocus?.(event));
    if (!event.defaultPrevented && target) capture(errors, () => focusElement(target));
  };
  const refresh = (): void => {
    assertAlive();
    if (!activeState.value) return;
    const nextRoot = readRoot(options.root);
    if (nextRoot !== token.root) {
      const sameDocument = nextRoot !== null && token.document === nextRoot.ownerDocument;
      if (!sameDocument) detachDom();
      if (nextRoot) {
        captureRestoration(nextRoot);
        attachScope(token, nextRoot);
        if (!sameDocument) {
          nextRoot.ownerDocument.addEventListener("focusin", onFocusin, true);
          nextRoot.ownerDocument.addEventListener("keydown", onKeydown, true);
        }
        observe(nextRoot);
        const errors: unknown[] = [];
        capture(errors, () => enter(nextRoot, errors));
        surfaceErrors(errors, "Focus scope entry failed");
      }
    }
    recover();
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
    const errors: unknown[] = [];
    const scopeRoot = token.root;
    const document = token.document;
    const active = document ? deepActiveElement(document) : null;
    const closingRoots = rootsOwnedBy(token);
    let shouldRestore = false;
    capture(errors, () => {
      const contain = readBoolean(options.contain, "contain");
      const restoreFocus = readBoolean(options.restoreFocus, "restoreFocus");
      shouldRestore =
        restoreFocus &&
        (!scopeRoot ||
          contain ||
          containsComposed(scopeRoot, active) ||
          (!scopeRoot.isConnected && active === document?.body));
    });
    const parent = parentScope(token);
    detachDom();
    activeState.value = false;
    if (shouldRestore) {
      let target: HTMLElement | null = null;
      capture(errors, () => {
        target = readTarget(options.restoreTarget?.(), "restoreTarget") ?? capturedTarget;
      });
      if (!isUsableTarget(target)) {
        const usableOutside = (candidate: HTMLElement): boolean =>
          isUsableTarget(candidate) &&
          !closingRoots.some((closingRoot) => containsComposed(closingRoot, candidate));
        const after = restoreCandidates.slice(restoreIndex + 1).find(usableOutside);
        const before = restoreCandidates
          .slice(0, Math.max(0, restoreIndex))
          .reverse()
          .find(usableOutside);
        const parentTarget = parent?.lastFocused ?? null;
        target = after ?? before ?? (isUsableTarget(parentTarget) ? parentTarget : null);
      }
      const event = createAutoFocusEvent("unmount", target);
      capture(errors, () => options.onUnmountAutoFocus?.(event));
      const restoreDestination = target;
      if (!event.defaultPrevented && isUsableTarget(restoreDestination)) {
        capture(errors, () => focusElement(restoreDestination));
      }
    }
    capturedTarget = null;
    restoreCandidates = [];
    restoreIndex = -1;
    restorationCaptured = false;
    entryHandled = false;
    token.lastFocused = null;
    surfaceErrors(errors, "Focus scope restoration failed");
  };
  const stopRootWatch = watch(
    () => toValue(options.root),
    () => {
      if (activeState.value) refresh();
    },
    { flush: "sync" },
  );

  return Object.freeze({
    ...manager,
    isActive: shallowReadonly(activeState),
    activate,
    deactivate,
    refresh,
    dispose: () => {
      if (disposed) return;
      try {
        deactivate();
      } finally {
        disposed = true;
        stopRootWatch();
        detachDom();
      }
    },
  });
}

export { useFocusScope } from "./use-focus-scope.ts";

export type {
  FocusScopeAutoFocusEvent,
  FocusScopeController,
  FocusScopeManager,
  FocusScopeMoveOptions,
  FocusScopeOptions,
} from "./focus-scope-types.ts";
