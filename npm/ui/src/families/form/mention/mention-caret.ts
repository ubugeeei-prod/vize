import type { Rect } from "../../overlays/positioner/positioner.ts";

/**
 * Caret geometry and text access for Mention fields.
 *
 * These helpers touch the DOM and are only called from event handlers and
 * positioner measurements, never during setup or server rendering.
 */

/** Editable surface kinds supported by Mention. */
export type MentionFieldKind = "editable" | "text";

/** Computed-style properties copied onto the textarea mirror. */
const mirroredProperties = [
  "boxSizing",
  "width",
  "height",
  "overflowX",
  "overflowY",
  "borderTopWidth",
  "borderRightWidth",
  "borderBottomWidth",
  "borderLeftWidth",
  "borderStyle",
  "paddingTop",
  "paddingRight",
  "paddingBottom",
  "paddingLeft",
  "fontStyle",
  "fontVariant",
  "fontWeight",
  "fontStretch",
  "fontSize",
  "fontSizeAdjust",
  "lineHeight",
  "fontFamily",
  "textAlign",
  "textTransform",
  "textIndent",
  "textDecoration",
  "letterSpacing",
  "wordSpacing",
  "tabSize",
  "direction",
] as const;

function toRect(rect: DOMRect | Rect): Rect {
  return { height: rect.height, width: rect.width, x: rect.x, y: rect.y };
}

/**
 * Measure the viewport rect of the caret at `index` inside a textarea or
 * input with a hidden mirror element that copies its text layout.
 */
export function measureTextFieldCaret(
  element: HTMLTextAreaElement | HTMLInputElement,
  index: number,
): Rect {
  const document = element.ownerDocument;
  const view = document.defaultView;
  const fieldRect = element.getBoundingClientRect();
  if (view === null) return toRect(fieldRect);
  const style = view.getComputedStyle(element);
  const mirror = document.createElement("div");
  const mirrorStyle = mirror.style;
  for (const property of mirroredProperties) mirrorStyle[property] = style[property];
  mirrorStyle.position = "fixed";
  mirrorStyle.visibility = "hidden";
  mirrorStyle.pointerEvents = "none";
  mirrorStyle.left = `${fieldRect.x}px`;
  mirrorStyle.top = `${fieldRect.y}px`;
  mirrorStyle.whiteSpace = element instanceof HTMLInputElement ? "pre" : "pre-wrap";
  mirrorStyle.overflowWrap = "break-word";
  mirror.textContent = element.value.slice(0, index);
  const marker = document.createElement("span");
  marker.textContent = element.value.slice(index) || ".";
  mirror.append(marker);
  document.body.append(mirror);
  try {
    const rect = marker.getBoundingClientRect();
    const lineHeight = Number.parseFloat(style.lineHeight) || rect.height || 0;
    return {
      height: lineHeight,
      width: 1,
      x: rect.x - element.scrollLeft,
      y: rect.y - element.scrollTop,
    };
  } finally {
    mirror.remove();
  }
}

/** Resolve a global text offset to a text node position inside `root`. */
export function locateTextOffset(
  root: Node,
  offset: number,
): { readonly node: Node; readonly offset: number } {
  const walker = root.ownerDocument?.createTreeWalker(root, 4);
  let remaining = offset;
  let last: Text | null = null;
  while (walker !== undefined) {
    const next = walker.nextNode();
    if (next === null) break;
    if (!(next instanceof Text)) continue;
    last = next;
    if (remaining <= next.data.length) return { node: next, offset: remaining };
    remaining -= next.data.length;
  }
  if (last !== null) return { node: last, offset: last.data.length };
  return { node: root, offset: 0 };
}

/** Measure the caret rect at `index` inside a contenteditable element. */
export function measureEditableCaret(element: HTMLElement, index: number): Rect {
  const document = element.ownerDocument;
  const position = locateTextOffset(element, index);
  const range = document.createRange();
  range.setStart(position.node, position.offset);
  range.collapse(true);
  const rect = range.getBoundingClientRect();
  if (rect.width === 0 && rect.height === 0) {
    const first = range.getClientRects()[0];
    return toRect(first ?? element.getBoundingClientRect());
  }
  return toRect(rect);
}

/** Read the plain text of a field. */
export function readFieldText(element: HTMLElement): string {
  if (element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement) {
    return element.value;
  }
  return element.textContent ?? "";
}

/** Read the collapsed caret offset of a field, or `null` for a range selection. */
export function readFieldCaret(element: HTMLElement): number | null {
  if (element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement) {
    const { selectionStart, selectionEnd } = element;
    if (selectionStart === null || selectionStart !== selectionEnd) return null;
    return selectionStart;
  }
  const selection = element.ownerDocument.getSelection();
  if (selection === null || selection.rangeCount === 0 || !selection.isCollapsed) return null;
  const focus = selection.focusNode;
  if (focus === null || !element.contains(focus)) return null;
  const range = element.ownerDocument.createRange();
  range.selectNodeContents(element);
  range.setEnd(focus, selection.focusOffset);
  return range.toString().length;
}

/** Replace `[start, end)` of a contenteditable's text and place the caret after the insertion. */
export function replaceEditableText(
  element: HTMLElement,
  start: number,
  end: number,
  inserted: string,
): void {
  const document = element.ownerDocument;
  const from = locateTextOffset(element, start);
  const to = locateTextOffset(element, end);
  const range = document.createRange();
  range.setStart(from.node, from.offset);
  range.setEnd(to.node, to.offset);
  range.deleteContents();
  const node = document.createTextNode(inserted);
  range.insertNode(node);
  element.normalize();
  const caret = locateTextOffset(element, start + inserted.length);
  const selection = document.getSelection();
  if (selection !== null) {
    const collapsed = document.createRange();
    collapsed.setStart(caret.node, caret.offset);
    collapsed.collapse(true);
    selection.removeAllRanges();
    selection.addRange(collapsed);
  }
}

/** Measure the caret rect at `index` for either field kind. */
export function measureFieldCaret(element: HTMLElement, index: number): Rect {
  if (element instanceof HTMLTextAreaElement || element instanceof HTMLInputElement) {
    return measureTextFieldCaret(element, index);
  }
  return measureEditableCaret(element, index);
}
