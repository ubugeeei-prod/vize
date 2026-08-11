import type { CollectionItem, CollectionKey } from "./collection-types.ts";

/**
 * Observe DOM ordering and accessible text mutations for registered items.
 *
 * One observer is created per owning document, scoped to the closest common
 * ancestor when every item is connected, so the registry can re-resolve
 * snapshots without polling.
 */
export function observeCollectionMutations<Key extends CollectionKey, Value>(
  items: readonly CollectionItem<Key, Value>[],
  onMutation: () => void,
): MutationObserver[] {
  const elementsByDocument = new Map<Document, Element[]>();
  for (const { element } of items) {
    if (element === null) continue;
    const elements = elementsByDocument.get(element.ownerDocument) ?? [];
    elements.push(element);
    elementsByDocument.set(element.ownerDocument, elements);
  }

  const observers: MutationObserver[] = [];
  for (const [ownerDocument, elements] of elementsByDocument) {
    const MutationObserverConstructor = ownerDocument.defaultView?.MutationObserver;
    if (MutationObserverConstructor === undefined) continue;

    const observer = new MutationObserverConstructor(onMutation);
    const commonAncestor = elements.every((element) => element.isConnected)
      ? findCollectionObservationRoot(elements)
      : null;
    const targets =
      commonAncestor === null
        ? new Set<Node>([
            ...elements.map((element) => element.getRootNode()),
            ownerDocument.documentElement,
          ])
        : new Set<Node>([commonAncestor]);
    for (const target of targets) {
      observer.observe(target, {
        attributeFilter: [
          "alt",
          "aria-hidden",
          "aria-label",
          "aria-labelledby",
          "hidden",
          "inert",
          "title",
          "type",
          "value",
        ],
        attributes: true,
        characterData: true,
        childList: true,
        subtree: true,
      });
    }
    observers.push(observer);
  }
  return observers;
}

function findCollectionObservationRoot(elements: readonly Element[]): Node | null {
  const firstElement = elements[0];
  if (firstElement === undefined) return null;

  let candidate: Node | null = firstElement.parentNode ?? firstElement;
  while (candidate !== null) {
    if (
      elements.every((element) => candidate === element || candidate?.contains(element) === true)
    ) {
      return candidate;
    }
    candidate = candidate.parentNode;
  }
  return null;
}
