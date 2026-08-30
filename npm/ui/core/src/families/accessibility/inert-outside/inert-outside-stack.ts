import { collectOpenShadowRoots, collectOutside, maskFor } from "./inert-outside-dom.ts";
import type { IsolationMask } from "./inert-outside-dom.ts";
import type { InertOutsideMode } from "./inert-outside-types.ts";

export interface InertOutsideToken {
  document: Document | null;
  root: Element | null;
  readonly readBranches: () => readonly Element[];
  readonly readEnabled: () => boolean;
  readonly readMode: () => InertOutsideMode;
  readonly setAffected: (elements: readonly Element[]) => void;
}

interface AttributeSnapshot {
  readonly ariaHidden: string | null;
  readonly inert: boolean;
}

interface DocumentState {
  readonly document: Document;
  readonly tokens: InertOutsideToken[];
  readonly managed: Map<Element, AttributeSnapshot>;
  observer: MutationObserver | null;
  queued: boolean;
}

const states = new WeakMap<Document, DocumentState>();
const observationOptions = Object.freeze({
  attributes: true,
  attributeFilter: ["aria-hidden", "inert", "name", "slot"],
  childList: true,
  subtree: true,
});

function stateFor(document: Document): DocumentState {
  let state = states.get(document);
  if (!state) {
    state = { document, managed: new Map(), observer: null, queued: false, tokens: [] };
    states.set(document, state);
  }
  return state;
}

function restore(state: DocumentState): void {
  for (const [element, snapshot] of state.managed) {
    if (snapshot.ariaHidden === null) element.removeAttribute("aria-hidden");
    else element.setAttribute("aria-hidden", snapshot.ariaHidden);
    if (snapshot.inert) element.setAttribute("inert", "");
    else element.removeAttribute("inert");
  }
  state.managed.clear();
}

function mergeMask(current: IsolationMask | undefined, next: IsolationMask): IsolationMask {
  return {
    ariaHidden: current?.ariaHidden === true || next.ariaHidden,
    inert: current?.inert === true || next.inert,
  };
}

function refreshObservationRoots(state: DocumentState): void {
  const root = state.document.documentElement;
  if (!state.observer || !root) return;
  state.observer.disconnect();
  state.observer.observe(root, observationOptions);
  for (const shadowRoot of collectOpenShadowRoots(state.document)) {
    state.observer.observe(shadowRoot, observationOptions);
  }
}

function connectedRoots(token: InertOutsideToken): readonly Element[] {
  if (!token.root?.isConnected || !token.readEnabled()) return [];
  return [...new Set([token.root, ...token.readBranches()])].filter((root) => root.isConnected);
}

export function recomputeDocument(state: DocumentState): void {
  const records = state.tokens
    .map((token) => ({ mode: token.readMode(), roots: connectedRoots(token), token }))
    .filter(({ roots }) => roots.length > 0);
  const desired = new Map<Element, IsolationMask>();
  for (let index = 0; index < records.length; index++) {
    const record = records[index];
    if (!record) continue;
    const allowed = records.slice(index).flatMap(({ roots }) => roots);
    const affected = collectOutside(state.document, allowed);
    record.token.setAffected(Object.freeze(affected));
    const mask = maskFor(record.mode);
    for (const element of affected) desired.set(element, mergeMask(desired.get(element), mask));
  }
  for (const token of state.tokens) {
    if (!records.some((record) => record.token === token)) token.setAffected(Object.freeze([]));
  }

  restore(state);
  for (const [element, mask] of desired) {
    state.managed.set(element, {
      ariaHidden: element.getAttribute("aria-hidden"),
      inert: element.hasAttribute("inert"),
    });
    if (mask.ariaHidden) element.setAttribute("aria-hidden", "true");
    if (mask.inert) element.setAttribute("inert", "");
  }
  state.observer?.takeRecords();
  refreshObservationRoots(state);
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
      if (state.tokens.length > 0) recomputeDocument(state);
    });
  });
  refreshObservationRoots(state);
}

export function attachToken(token: InertOutsideToken, root: Element): void {
  if (token.document === root.ownerDocument) {
    token.root = root;
    recomputeDocument(stateFor(root.ownerDocument));
    return;
  }
  detachToken(token);
  token.document = root.ownerDocument;
  token.root = root;
  const state = stateFor(root.ownerDocument);
  state.tokens.push(token);
  observe(state);
  recomputeDocument(state);
}

export function detachToken(token: InertOutsideToken): void {
  const document = token.document;
  if (!document) {
    token.root = null;
    token.setAffected(Object.freeze([]));
    return;
  }
  const state = stateFor(document);
  const index = state.tokens.indexOf(token);
  if (index >= 0) state.tokens.splice(index, 1);
  token.document = null;
  token.root = null;
  token.setAffected(Object.freeze([]));
  if (state.tokens.length === 0) {
    state.observer?.disconnect();
    state.observer = null;
    restore(state);
    states.delete(document);
  } else {
    recomputeDocument(state);
  }
}

export function refreshToken(token: InertOutsideToken): void {
  if (token.document) recomputeDocument(stateFor(token.document));
}
