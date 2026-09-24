import type { RichTextPosition, RichTextSelection } from "./rich-text-model.ts";

/** Point in the DOM (a node and an offset within it). */
export interface RichTextDomPoint {
  readonly node: Node;
  readonly offset: number;
}

function isElement(node: Node): node is Element {
  return node.nodeType === 1;
}

function isLeaf(node: Node): boolean {
  return isElement(node) && node.hasAttribute("data-rt-leaf");
}

function isFiller(node: Node): boolean {
  return isElement(node) && node.hasAttribute("data-rt-filler");
}

function sizeOf(node: Node): number {
  if (node.nodeType === 3) return node.textContent?.length ?? 0;
  if (isLeaf(node)) return 1;
  if (isFiller(node) || !isElement(node)) return 0;
  let total = 0;
  for (const child of node.childNodes) total += sizeOf(child);
  return total;
}

function textblockOf(node: Node, root: Element): Element | null {
  let current: Node | null = node;
  while (current && current !== root) {
    if (isElement(current) && current.hasAttribute("data-rt-textblock")) return current;
    current = current.parentNode;
  }
  return null;
}

function pathOf(block: Element): number[] {
  return (block.getAttribute("data-rt-path") ?? "")
    .split(".")
    .filter((part) => part !== "")
    .map(Number);
}

function offsetWithin(block: Element, target: Node, targetOffset: number): number {
  let count = 0;
  let done = false;
  const visit = (node: Node): void => {
    if (done) return;
    if (node === target) {
      if (node.nodeType === 3) count += Math.min(targetOffset, sizeOf(node));
      else if (isLeaf(node)) count += targetOffset > 0 ? 1 : 0;
      else for (const child of [...node.childNodes].slice(0, targetOffset)) count += sizeOf(child);
      done = true;
      return;
    }
    if (node.nodeType === 3 || isLeaf(node) || isFiller(node)) {
      count += sizeOf(node);
      return;
    }
    for (const child of node.childNodes) visit(child);
  };
  visit(block);
  return count;
}

/** Map a DOM point inside the editable root to a model position. */
export function positionFromDom(
  root: Element,
  node: Node,
  offset: number,
): RichTextPosition | null {
  if (!root.contains(node)) return null;
  const block = textblockOf(node, root);
  if (block) return { path: pathOf(block), offset: offsetWithin(block, node, offset) };
  if (!isElement(node)) return null;
  // Element-level point between blocks: snap to the nearest textblock.
  const blocks = [...root.querySelectorAll("[data-rt-textblock]")];
  const after = node.childNodes[offset];
  if (after) {
    const next = blocks.find(
      (candidate) =>
        after === candidate ||
        after.contains(candidate) ||
        (after.compareDocumentPosition(candidate) & 4) !== 0,
    );
    if (next) return { path: pathOf(next), offset: 0 };
  }
  const before = node.childNodes[offset - 1] ?? node;
  const previous = [...blocks]
    .reverse()
    .find(
      (candidate) =>
        before === candidate ||
        before.contains(candidate) ||
        (before.compareDocumentPosition(candidate) & 2) !== 0,
    );
  return previous ? { path: pathOf(previous), offset: sizeOf(previous) } : null;
}

/** Map a model position to a DOM point, preferring text nodes. */
export function domFromPosition(
  root: Element,
  position: RichTextPosition,
): RichTextDomPoint | null {
  const path = position.path.join(".");
  const block = [...root.querySelectorAll("[data-rt-textblock]")].find(
    (candidate) => candidate.getAttribute("data-rt-path") === path,
  );
  if (!block) return null;
  let remaining = position.offset;
  let found: RichTextDomPoint | null = null;
  const visit = (node: Node): void => {
    if (found) return;
    if (node.nodeType === 3) {
      const length = node.textContent?.length ?? 0;
      if (remaining <= length) found = { node, offset: remaining };
      else remaining -= length;
      return;
    }
    if (isLeaf(node) || isFiller(node)) {
      const parent = node.parentNode;
      const index = parent ? [...parent.childNodes].indexOf(node as ChildNode) : 0;
      if (remaining === 0 && parent) found = { node: parent, offset: index };
      else if (isLeaf(node)) {
        remaining -= 1;
        if (remaining === 0 && parent && !node.nextSibling)
          found = { node: parent, offset: index + 1 };
      }
      return;
    }
    for (const child of node.childNodes) visit(child);
  };
  visit(block);
  return found ?? { node: block, offset: remaining > 0 ? block.childNodes.length : 0 };
}

/** Read the document selection when it lies inside `root`. */
export function readDomSelection(root: Element): RichTextSelection | null {
  const selection = root.ownerDocument.getSelection();
  if (!selection?.anchorNode || !selection.focusNode) return null;
  const anchor = positionFromDom(root, selection.anchorNode, selection.anchorOffset);
  const head = positionFromDom(root, selection.focusNode, selection.focusOffset);
  return anchor && head ? { anchor, head } : null;
}

/** Write a model selection into the document selection. */
export function writeDomSelection(root: Element, selection: RichTextSelection): boolean {
  const anchor = domFromPosition(root, selection.anchor);
  const head = domFromPosition(root, selection.head);
  const domSelection = root.ownerDocument.getSelection();
  if (!anchor || !head || !domSelection) return false;
  if (
    domSelection.anchorNode === anchor.node &&
    domSelection.anchorOffset === anchor.offset &&
    domSelection.focusNode === head.node &&
    domSelection.focusOffset === head.offset
  ) {
    return true;
  }
  domSelection.setBaseAndExtent(anchor.node, anchor.offset, head.node, head.offset);
  return true;
}

/** Viewport rectangle of the current DOM selection, when measurable. */
export function selectionRect(root: Element): DOMRect | null {
  const selection = root.ownerDocument.getSelection();
  if (!selection || selection.rangeCount === 0) return null;
  const range = selection.getRangeAt(0);
  if (!root.contains(range.commonAncestorContainer)) return null;
  return typeof range.getBoundingClientRect === "function" ? range.getBoundingClientRect() : null;
}
