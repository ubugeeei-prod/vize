import {
  applyDocumentLock,
  captureDocumentLock,
  measureScrollbarGap,
  resolveScrollLockStrategy,
  restoreDocumentLock,
  restoreDocumentScroll,
} from "./scroll-lock-dom.ts";
import type { DocumentLockSnapshot } from "./scroll-lock-dom.ts";
import type { ScrollLockStrategy } from "./scroll-lock-types.ts";

type ResolvedStrategy = Exclude<ScrollLockStrategy, "auto">;

export interface ScrollLockToken {
  document: Document | null;
  readonly readEnabled: () => boolean;
  readonly readPreserveGap: () => boolean;
  readonly readRestoreScroll: () => boolean;
  readonly readStrategy: () => ScrollLockStrategy;
  readonly setState: (locked: boolean, gap: number, strategy: ResolvedStrategy | null) => void;
}

interface DocumentLockState {
  readonly document: Document;
  readonly tokens: ScrollLockToken[];
  snapshot: DocumentLockSnapshot | null;
  restoreOnRelease: boolean;
}

const states = new WeakMap<Document, DocumentLockState>();

function stateFor(document: Document): DocumentLockState {
  let state = states.get(document);
  if (!state) {
    state = { document, restoreOnRelease: true, snapshot: null, tokens: [] };
    states.set(document, state);
  }
  return state;
}

function restoreAppliedState(state: DocumentLockState, restoreScroll: boolean): void {
  const snapshot = state.snapshot;
  if (!snapshot) return;
  restoreDocumentLock(snapshot);
  if (restoreScroll) restoreDocumentScroll(snapshot);
}

function effectiveStrategy(
  document: Document,
  tokens: readonly ScrollLockToken[],
): ResolvedStrategy {
  const strategies = tokens.map((token) =>
    resolveScrollLockStrategy(document, token.readStrategy()),
  );
  return strategies.includes("fixed") ? "fixed" : "overflow";
}

export function recomputeScrollLock(state: DocumentLockState): void {
  const enabled = state.tokens.filter((token) => token.readEnabled());
  if (enabled.length === 0) {
    restoreAppliedState(state, state.restoreOnRelease);
    state.snapshot = null;
    for (const token of state.tokens) token.setState(false, 0, null);
    return;
  }

  const previous = state.snapshot;
  if (previous) restoreAppliedState(state, true);
  const next = captureDocumentLock(state.document);
  if (!next) {
    state.snapshot = null;
    for (const token of state.tokens) token.setState(false, 0, null);
    return;
  }
  state.snapshot = previous
    ? { ...next, scrollX: previous.scrollX, scrollY: previous.scrollY }
    : next;
  state.restoreOnRelease = enabled[0]?.readRestoreScroll() ?? true;

  const strategy = effectiveStrategy(state.document, enabled);
  const gap = measureScrollbarGap(state.document);
  const preserveGap = enabled.some((token) => token.readPreserveGap());
  applyDocumentLock(state.snapshot, strategy, gap, preserveGap);
  for (const token of state.tokens) {
    const locked = enabled.includes(token);
    token.setState(locked, locked ? gap : 0, locked ? strategy : null);
  }
}

export function attachScrollLock(token: ScrollLockToken, document: Document): void {
  if (token.document === document) {
    recomputeScrollLock(stateFor(document));
    return;
  }
  detachScrollLock(token);
  token.document = document;
  const state = stateFor(document);
  state.tokens.push(token);
  recomputeScrollLock(state);
}

export function detachScrollLock(token: ScrollLockToken): void {
  const document = token.document;
  if (!document) {
    token.setState(false, 0, null);
    return;
  }
  const state = stateFor(document);
  const index = state.tokens.indexOf(token);
  if (index >= 0) state.tokens.splice(index, 1);
  token.document = null;
  token.setState(false, 0, null);
  if (state.tokens.length === 0) {
    restoreAppliedState(state, state.restoreOnRelease);
    state.snapshot = null;
    states.delete(document);
  } else recomputeScrollLock(state);
}

export function refreshScrollLock(token: ScrollLockToken): void {
  if (token.document) recomputeScrollLock(stateFor(token.document));
}
