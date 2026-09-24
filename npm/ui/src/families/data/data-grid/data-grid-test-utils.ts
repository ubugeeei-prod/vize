import assert from "node:assert/strict";

import { nextTick } from "vue";

/** Flush Vue updates and deferred focus. */
export async function settle(): Promise<void> {
  for (let index = 0; index < 3; index++) await nextTick();
}

/** Dispatch a keydown on the focused element and settle. */
export async function press(
  key: string,
  init: Partial<KeyboardEventInit> = {},
): Promise<KeyboardEvent> {
  const target = document.activeElement ?? document.body;
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  await settle();
  return event;
}

/** Click with modifier keys and settle. */
export async function click(target: Element, init: Partial<MouseEventInit> = {}): Promise<void> {
  target.dispatchEvent(
    new MouseEvent("click", { bubbles: true, cancelable: true, detail: 1, ...init }),
  );
  await settle();
}

/** Every element matching a selector. */
export function all(selector: string, scope: ParentNode = document): HTMLElement[] {
  return [...scope.querySelectorAll(selector)].filter(
    (element): element is HTMLElement => element instanceof HTMLElement,
  );
}

/** The single element matching a selector. */
export function one(selector: string, scope: ParentNode = document): HTMLElement {
  const element = scope.querySelector(selector);
  assert.ok(element instanceof HTMLElement, `expected ${selector}`);
  return element;
}

/** Body cell by row id and column id. */
export function cell(rowId: string, columnId: string): HTMLElement {
  return one(`[role="gridcell"][data-row-id="${rowId}"][data-column-id="${columnId}"]`);
}

/** Column header by column id. */
export function header(columnId: string): HTMLElement {
  return one(`[role="columnheader"][data-column-id="${columnId}"]`);
}

/** Row ids in rendered order. */
export function rowIds(): string[] {
  return all('[role="row"][data-row-id]').map((row) => row.dataset.rowId ?? "");
}

/** Column ids in rendered header order. */
export function columnIds(): string[] {
  return all('[role="columnheader"]').map((element) => element.dataset.columnId ?? "");
}

/** `data-row-id:data-column-id` of the focused cell, or its role. */
export function focused(): string {
  const active = document.activeElement;
  if (!(active instanceof HTMLElement)) return "";
  if (active.getAttribute("role") === "columnheader") return `header:${active.dataset.columnId}`;
  if (active.getAttribute("role") === "gridcell")
    return `${active.dataset.rowId}:${active.dataset.columnId}`;
  return active.tagName.toLowerCase();
}
