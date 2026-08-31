import type { InertOutsideMode } from "./inert-outside-types.ts";

export interface IsolationMask {
  readonly ariaHidden: boolean;
  readonly inert: boolean;
}

const ignoredNames = new Set(["link", "meta", "noscript", "script", "style", "template"]);

function composedParent(element: Element): Element | null {
  if ((element as HTMLElement).assignedSlot) return (element as HTMLElement).assignedSlot;
  if (element.parentElement) return element.parentElement;
  return (element.getRootNode() as Partial<ShadowRoot>).host ?? null;
}

export function containsComposed(root: Element, element: Element): boolean {
  let current: Element | null = element;
  while (current) {
    if (current === root || root.contains(current)) return true;
    current = composedParent(current);
  }
  return false;
}

function renderedChildren(element: Element): readonly Element[] {
  const slot = element as HTMLSlotElement;
  if (slot.localName === "slot" && typeof slot.assignedElements === "function") {
    const assigned = slot.assignedElements({ flatten: true });
    if (assigned.length > 0) return assigned;
  }
  return Array.from(element.shadowRoot?.children ?? element.children);
}

/** Collect every currently reachable open shadow root for document-level observation. */
export function collectOpenShadowRoots(root: Document | ShadowRoot): ShadowRoot[] {
  const roots: ShadowRoot[] = [];
  for (const element of root.querySelectorAll("*")) {
    const shadowRoot = element.shadowRoot;
    if (!shadowRoot) continue;
    roots.push(shadowRoot, ...collectOpenShadowRoots(shadowRoot));
  }
  return roots;
}

function isInsideAllowed(element: Element, allowed: readonly Element[]): boolean {
  return allowed.some((root) => root === element || containsComposed(root, element));
}

function containsAllowed(element: Element, allowed: readonly Element[]): boolean {
  return allowed.some((root) => root === element || containsComposed(element, root));
}

/** Find the smallest rendered sibling subtrees outside every allowed root. */
export function collectOutside(document: Document, allowed: readonly Element[]): Element[] {
  const body = document.body;
  if (!body || allowed.some((root) => root === body || containsComposed(root, body))) return [];
  const outside: Element[] = [];
  const visit = (container: Element): void => {
    for (const child of renderedChildren(container)) {
      if (ignoredNames.has(child.localName)) continue;
      if (isInsideAllowed(child, allowed)) continue;
      if (containsAllowed(child, allowed)) visit(child);
      else outside.push(child);
    }
  };
  visit(body);
  return outside;
}

export function maskFor(mode: InertOutsideMode): IsolationMask {
  return {
    ariaHidden: mode === "aria-hidden" || mode === "both",
    inert: mode === "inert" || mode === "both",
  };
}
