export interface DismissableLayerToken {
  document: Document | null;
  root: Element | null;
  readonly readBranches: () => readonly Element[];
  readonly readEnabled: () => boolean;
  readonly readEscapeKey: () => boolean;
  readonly readOutsideFocus: () => boolean;
  readonly readOutsidePointerDown: () => boolean;
  readonly handleEscapeKeyDown: (event: KeyboardEvent, target: Element | null) => void;
  readonly handleFocusOutside: (event: FocusEvent, target: Element) => void;
  readonly handlePointerDownOutside: (
    event: PointerEvent | MouseEvent | TouchEvent,
    target: Element,
  ) => void;
  readonly setTopLayer: (value: boolean) => void;
}

interface DocumentState {
  readonly document: Document;
  readonly releaseListeners: Array<() => void>;
  readonly tokens: DismissableLayerToken[];
  lastPointerTime: number;
  observer: MutationObserver | null;
  queued: boolean;
}

const documentStates = new WeakMap<Document, DocumentState>();

function composedParent(element: Element): Element | null {
  if ((element as HTMLElement).assignedSlot) return (element as HTMLElement).assignedSlot;
  if (element.parentElement) return element.parentElement;
  return (element.getRootNode() as Partial<ShadowRoot>).host ?? null;
}

function containsComposed(root: Element, element: Element): boolean {
  let current: Element | null = element;
  while (current) {
    if (root === current || root.contains(current)) return true;
    current = composedParent(current);
  }
  return false;
}

function stackFor(document: Document): DocumentState {
  let state = documentStates.get(document);
  if (!state) {
    state = {
      document,
      lastPointerTime: Number.NEGATIVE_INFINITY,
      observer: null,
      queued: false,
      releaseListeners: [],
      tokens: [],
    };
    documentStates.set(document, state);
  }
  return state;
}

function rootsFor(token: DismissableLayerToken): readonly Element[] {
  const root = token.root;
  if (!root?.isConnected || !token.readEnabled()) return [];
  return [...new Set([root, ...token.readBranches()])].filter((branch) => branch.isConnected);
}

function topLayer(state: DocumentState): DismissableLayerToken | null {
  for (let index = state.tokens.length - 1; index >= 0; index--) {
    const token = state.tokens[index];
    if (token && rootsFor(token).length > 0) return token;
  }
  return null;
}

function targetElement(event: Event): Element | null {
  const path = typeof event.composedPath === "function" ? event.composedPath() : [];
  for (const item of path) {
    const element = item as Partial<Element>;
    if (element.nodeType === 1 && typeof element.getRootNode === "function") {
      return element as Element;
    }
  }
  const target = event.target as Partial<Element> | null;
  return target?.nodeType === 1 && typeof target.getRootNode === "function"
    ? (target as Element)
    : null;
}

function eventInside(event: Event, target: Element, roots: readonly Element[]): boolean {
  const path = typeof event.composedPath === "function" ? event.composedPath() : [];
  return roots.some((root) => path.includes(root) || containsComposed(root, target));
}

export function recomputeDismissableLayers(document: Document): void {
  const state = stackFor(document);
  const owner = topLayer(state);
  for (const token of state.tokens) token.setTopLayer(token === owner);
}

function routePointerDown(
  state: DocumentState,
  event: PointerEvent | MouseEvent | TouchEvent,
): void {
  if (event.defaultPrevented) return;
  const owner = topLayer(state);
  if (!owner?.readOutsidePointerDown()) return;
  const target = targetElement(event);
  if (!target || eventInside(event, target, rootsFor(owner))) return;
  owner.handlePointerDownOutside(event, target);
}

function routeFocusIn(state: DocumentState, event: FocusEvent): void {
  const owner = topLayer(state);
  if (!owner?.readOutsideFocus()) return;
  const target = targetElement(event);
  if (!target || eventInside(event, target, rootsFor(owner))) return;
  owner.handleFocusOutside(event, target);
}

function routeEscape(state: DocumentState, event: KeyboardEvent): void {
  if (event.key !== "Escape" || event.defaultPrevented || event.isComposing) return;
  const owner = topLayer(state);
  if (!owner?.readEscapeKey()) return;
  owner.handleEscapeKeyDown(event, targetElement(event));
}

function observe(state: DocumentState): void {
  if (state.observer) return;
  const Observer = state.document.defaultView?.MutationObserver;
  const root = state.document.documentElement;
  if (!Observer || !root) return;
  state.observer = new Observer(() => {
    if (state.queued) return;
    state.queued = true;
    queueMicrotask(() => {
      state.queued = false;
      if (state.tokens.length > 0) recomputeDismissableLayers(state.document);
    });
  });
  state.observer.observe(root, { childList: true, subtree: true });
}

function addDocumentListeners(state: DocumentState): void {
  if (state.releaseListeners.length > 0) return;
  const document = state.document;
  const listen = (
    type: string,
    callback: EventListener,
    options?: AddEventListenerOptions | boolean,
  ) => {
    document.addEventListener(type, callback, options);
    state.releaseListeners.push(() => document.removeEventListener(type, callback, options));
  };
  listen(
    "pointerdown",
    ((event: PointerEvent) => {
      state.lastPointerTime = event.timeStamp;
      routePointerDown(state, event);
    }) as EventListener,
    true,
  );
  listen(
    "mousedown",
    ((event: MouseEvent) => {
      const view = document.defaultView;
      const elapsed = event.timeStamp - state.lastPointerTime;
      if (view && "PointerEvent" in view && elapsed >= 0 && elapsed < 800) return;
      routePointerDown(state, event);
    }) as EventListener,
    true,
  );
  listen(
    "touchstart",
    ((event: TouchEvent) => {
      const view = document.defaultView;
      if (view && "PointerEvent" in view) return;
      routePointerDown(state, event);
    }) as EventListener,
    true,
  );
  listen("focusin", ((event: FocusEvent) => routeFocusIn(state, event)) as EventListener, true);
  listen("keydown", ((event: KeyboardEvent) => routeEscape(state, event)) as EventListener);
}

function removeDocumentListeners(state: DocumentState): void {
  for (const release of state.releaseListeners.splice(0)) release();
}

export function attachDismissableLayer(token: DismissableLayerToken, root: Element): void {
  if (token.document === root.ownerDocument) {
    token.root = root;
    recomputeDismissableLayers(root.ownerDocument);
    return;
  }
  detachDismissableLayer(token);
  const state = stackFor(root.ownerDocument);
  const wasEmpty = state.tokens.length === 0;
  token.document = root.ownerDocument;
  token.root = root;
  state.tokens.push(token);
  if (wasEmpty) {
    addDocumentListeners(state);
    observe(state);
  }
  recomputeDismissableLayers(root.ownerDocument);
}

export function detachDismissableLayer(token: DismissableLayerToken): void {
  const document = token.document;
  if (!document) {
    token.root = null;
    token.setTopLayer(false);
    return;
  }
  const state = stackFor(document);
  const index = state.tokens.indexOf(token);
  if (index >= 0) state.tokens.splice(index, 1);
  token.document = null;
  token.root = null;
  token.setTopLayer(false);
  if (state.tokens.length === 0) {
    removeDocumentListeners(state);
    state.observer?.disconnect();
    state.observer = null;
    documentStates.delete(document);
  } else {
    recomputeDismissableLayers(document);
  }
}

export function refreshDismissableLayer(token: DismissableLayerToken): void {
  if (token.document) recomputeDismissableLayers(token.document);
}
