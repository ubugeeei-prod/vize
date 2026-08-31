import type { CollectionItem, CollectionKey } from "../collection/collection.ts";

import { capture } from "./composite-navigation-internal.ts";

export function eventElement(value: EventTarget | null): Element | null {
  const candidate = value as Partial<Element> | null;
  return candidate?.nodeType === 1 && typeof candidate.getRootNode === "function"
    ? (candidate as Element)
    : null;
}

export function focusItem(
  element: Element | null,
  preventScroll: boolean,
  errors: unknown[],
): void {
  const focusable = element as Partial<HTMLElement> | null;
  if (typeof focusable?.focus !== "function") {
    errors.push(
      new Error("VIZE_UI_COMPOSITE_NAVIGATION_FOCUS: active roving item is not focusable"),
    );
    return;
  }
  capture(errors, () => {
    try {
      focusable.focus?.({ preventScroll });
    } catch {
      focusable.focus?.();
    }
  });
}

export function revealItem<Key extends CollectionKey, Value>(
  item: CollectionItem<Key, Value>,
  custom: ((item: CollectionItem<Key, Value>, event: Event | null) => void) | undefined,
  event: Event | null,
  errors: unknown[],
): void {
  if (custom) {
    capture(errors, () => custom(item, event));
    return;
  }
  const reveal = (item.element as Partial<HTMLElement> | null)?.scrollIntoView;
  if (typeof reveal === "function") {
    capture(errors, () => reveal.call(item.element, { block: "nearest", inline: "nearest" }));
  }
}

function tokenIncludes(value: string | null, token: string): boolean {
  return value?.split(/\s+/u).includes(token) === true;
}

function relatedElement(host: Element, id: string): Element | null {
  const root = host.getRootNode() as { getElementById?: (value: string) => Element | null };
  if (typeof root.getElementById === "function") return root.getElementById(id);
  return host.ownerDocument.getElementById(id);
}

export function validateActiveDescendant<Key extends CollectionKey, Value>(
  host: Element | null,
  item: CollectionItem<Key, Value>,
  id: string,
  errors: unknown[],
): void {
  const element = item.element;
  if (!host || !element) return;
  if (element.id !== id || relatedElement(element, id) !== element) {
    errors.push(
      new Error(
        "VIZE_UI_COMPOSITE_NAVIGATION_ID_RELATIONSHIP: active descendant ID must uniquely resolve to its registered element",
      ),
    );
    return;
  }
  if (host.contains(element) || tokenIncludes(host.getAttribute("aria-owns"), id)) return;
  const role = host.getAttribute("role");
  if (role === "combobox" || role === "searchbox" || role === "textbox") {
    const controlledIds = host.getAttribute("aria-controls")?.split(/\s+/u) ?? [];
    for (const controlledId of controlledIds) {
      const controlled = relatedElement(host, controlledId);
      if (
        controlled?.contains(element) ||
        tokenIncludes(controlled?.getAttribute("aria-owns") ?? null, id)
      ) {
        return;
      }
    }
  }
  errors.push(
    new Error(
      "VIZE_UI_COMPOSITE_NAVIGATION_RELATIONSHIP: active descendant must be contained, aria-owned, or controlled by a supported input role",
    ),
  );
}
