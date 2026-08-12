import {
  computed,
  getCurrentScope,
  isRef,
  onScopeDispose,
  shallowReadonly,
  shallowRef,
  toValue,
  watch,
} from "vue";

import {
  publishDocumentModality,
  subscribeToDocumentModality,
} from "./interaction-modality-hub.ts";
import type { DocumentModalityUpdate } from "./interaction-modality-hub.ts";
import type {
  InteractionModality,
  InteractionModalityChange,
  InteractionModalityOptions,
  InteractionModalityTracker,
} from "./interaction-modality-types.ts";

const disposedDiagnostic = "VIZE_UI_INTERACTION_MODALITY_DISPOSED";
const invalidDocumentDiagnostic = "VIZE_UI_INTERACTION_MODALITY_DOCUMENT";
const setupDiagnostic = "VIZE_UI_INTERACTION_MODALITY_SETUP";
const invalidValueDiagnostic = "VIZE_UI_INTERACTION_MODALITY_VALUE";
const modalities = new Set<InteractionModality>(["keyboard", "pointer", "touch", "virtual"]);

function defaultDocument(): Document | null {
  return typeof globalThis.document === "undefined" ? null : globalThis.document;
}

/** Accept cross-realm documents without relying on an unreliable instanceof check. */
function toDocument(value: Document | null | undefined): Document | null {
  if (value == null) return null;
  if (
    value.nodeType !== 9 ||
    typeof value.addEventListener !== "function" ||
    typeof value.removeEventListener !== "function"
  ) {
    throw new TypeError(`${invalidDocumentDiagnostic}: expected a Document, null, or undefined`);
  }
  return value;
}

/** Validate JavaScript callers without widening the closed TypeScript union. */
function toModality(value: InteractionModality | null): InteractionModality | null {
  if (value === null || modalities.has(value)) return value;
  throw new TypeError(
    `${invalidValueDiagnostic}: expected keyboard, pointer, touch, virtual, or null`,
  );
}

/**
 * Create an SSR-safe, document-scoped interaction-modality observer.
 *
 * Trackers in one document share native capture listeners and state. Separate
 * documents, including iframes, remain isolated. Call {@link InteractionModalityTracker.dispose}
 * when using this factory outside a Vue effect scope.
 */
export function createInteractionModalityTracker(
  options: InteractionModalityOptions = {},
): InteractionModalityTracker {
  const ownerDocument = shallowRef<Document | null>(null);
  const modality = shallowRef<InteractionModality | null>(
    toModality(options.initialModality ?? null),
  );
  let releaseDocument: (() => void) | null = null;
  let disposed = false;

  const assertActive = () => {
    if (disposed) throw new Error(`${disposedDiagnostic}: the tracker has been disposed`);
  };
  const apply = (update: DocumentModalityUpdate) => {
    if (modality.value === update.modality) return false;
    const previousModality = modality.value;
    modality.value = update.modality;
    const change: InteractionModalityChange = Object.freeze({
      modality: update.modality,
      previousModality,
      reason: update.reason,
      originalEvent: update.originalEvent,
      document: ownerDocument.value,
    });
    options.onChange?.(change);
    return true;
  };
  const releaseCurrentDocument = () => {
    const changed = ownerDocument.value !== null;
    releaseDocument?.();
    releaseDocument = null;
    ownerDocument.value = null;
    return changed;
  };
  const attach = (nextDocument: Document | null) => {
    assertActive();
    const resolved = toDocument(nextDocument);
    if (ownerDocument.value === resolved) return false;
    releaseCurrentDocument();
    if (!resolved) return true;

    ownerDocument.value = resolved;
    try {
      const subscription = subscribeToDocumentModality(resolved, modality.value, apply);
      releaseDocument = subscription.release;
      if (subscription.current !== modality.value) {
        apply({ modality: subscription.current, reason: "document", originalEvent: null });
      }
    } catch (error) {
      releaseCurrentDocument();
      throw error;
    }
    return true;
  };
  const detach = () => {
    assertActive();
    return releaseCurrentDocument();
  };
  const setModality = (nextModality: InteractionModality | null) => {
    assertActive();
    const update: DocumentModalityUpdate = {
      modality: toModality(nextModality),
      reason: "manual",
      originalEvent: null,
    };
    if (ownerDocument.value) {
      return publishDocumentModality(ownerDocument.value, update);
    }
    return apply(update);
  };

  let stopDocumentWatch: () => void = () => undefined;
  if (options.document === undefined) {
    attach(defaultDocument());
  } else if (isRef(options.document) || typeof options.document === "function") {
    stopDocumentWatch = watch(
      () => toValue(options.document),
      (nextDocument) => attach(toDocument(nextDocument)),
      { flush: "sync", immediate: true },
    );
  } else {
    attach(toDocument(options.document));
  }
  const dispose = () => {
    if (disposed) return;
    stopDocumentWatch();
    releaseCurrentDocument();
    disposed = true;
  };

  return Object.freeze({
    document: shallowReadonly(ownerDocument),
    modality: shallowReadonly(modality),
    isFocusVisible: computed(() => modality.value === "keyboard" || modality.value === "virtual"),
    attach,
    detach,
    setModality,
    dispose,
  });
}

/**
 * Create a tracker owned by the current Vue effect scope.
 *
 * The tracker is disposed automatically when its component or effect scope is
 * destroyed, preventing document-listener leaks.
 */
export function useInteractionModality(
  options: InteractionModalityOptions = {},
): InteractionModalityTracker {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const tracker = createInteractionModalityTracker(options);
  onScopeDispose(tracker.dispose);
  return tracker;
}

/**
 * Determine whether a focused element should expose its focus indicator.
 *
 * Native `:focus-visible` semantics take precedence. The modality fallback is
 * used only when the selector is unsupported, preserving text-input and
 * platform heuristics implemented by the browser.
 */
export function isElementFocusVisible(
  element: Element | null | undefined,
  modality: InteractionModality | null,
): boolean {
  if (!element) return false;
  const root = element.getRootNode();
  const activeElement =
    root && "activeElement" in root
      ? (root as Document | ShadowRoot).activeElement
      : element.ownerDocument.activeElement;
  if (activeElement !== element) return false;

  try {
    return element.matches(":focus-visible");
  } catch {
    return modality === "keyboard" || modality === "virtual";
  }
}

export type {
  InteractionModality,
  InteractionModalityChange,
  InteractionModalityChangeReason,
  InteractionModalityOptions,
  InteractionModalityTracker,
} from "./interaction-modality-types.ts";
