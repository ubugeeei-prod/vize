import assert from "node:assert/strict";

import { nextTick } from "vue";
/**
 * Wrap a mount function so every handle is tracked and a failed test cannot
 * leak an open menu into the next one. The harness is injected so this module
 * stays free of test-only imports during type checking.
 */
export function createMenuMounter<Args extends unknown[], Handle extends { unmount(): void }>(
  mount: (...args: Args) => Handle,
): { readonly mountMenu: (...args: Args) => Handle; readonly cleanup: () => void } {
  const mounted = new Set<Handle>();
  return {
    mountMenu: (...args) => {
      const handle = mount(...args);
      mounted.add(handle);
      return {
        ...handle,
        unmount: () => {
          if (!mounted.delete(handle)) return;
          handle.unmount();
        },
      };
    },
    cleanup: () => {
      for (const handle of mounted) handle.unmount();
      mounted.clear();
    },
  };
}

/** Flush Vue updates, the portal move, and deferred controller activation. */
export async function settle(): Promise<void> {
  for (let index = 0; index < 4; index++) await nextTick();
}

/** Wait for real timers such as long-press thresholds. */
export async function wait(milliseconds: number): Promise<void> {
  await new Promise<void>((resolve) => setTimeout(resolve, milliseconds));
  await settle();
}

/** Dispatch a keydown on `target` and report whether it was canceled. */
export function keydown(
  target: Element,
  key: string,
  init: Partial<KeyboardEventInit> = {},
): KeyboardEvent {
  const event = new KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

/** Press a key on the focused element and settle. */
export async function press(key: string, init: Partial<KeyboardEventInit> = {}): Promise<void> {
  const target = document.activeElement;
  assert.ok(target, "a focused element is required to press a key");
  keydown(target, key, init);
  await settle();
}

/** Dispatch a pointer event with mouse defaults. */
export function pointer(
  target: Element,
  type: string,
  init: Partial<PointerEventInit> = {},
): PointerEvent {
  const event = new PointerEvent(type, {
    bubbles: type !== "pointerenter" && type !== "pointerleave",
    cancelable: true,
    button: 0,
    clientX: 0,
    clientY: 0,
    isPrimary: true,
    pointerId: 1,
    pointerType: "mouse",
    ...init,
  });
  target.dispatchEvent(event);
  return event;
}

/** Single element matching a selector anywhere in the document. */
export function one(selector: string): HTMLElement {
  const element = document.querySelector(selector);
  assert.ok(element instanceof HTMLElement, `expected ${selector}`);
  return element;
}

/** Element matching a selector, or `null`. */
export function maybe(selector: string): HTMLElement | null {
  const element = document.querySelector(selector);
  assert.ok(element === null || element instanceof HTMLElement);
  return element;
}

/** Menu item (any item role) with the given text. */
export function item(text: string, scope: ParentNode = document): HTMLElement {
  const match = [
    ...scope.querySelectorAll(
      '[role="menuitem"], [role="menuitemcheckbox"], [role="menuitemradio"]',
    ),
  ].find((element) => element.textContent?.trim() === text);
  assert.ok(match instanceof HTMLElement, `expected menu item ${text}`);
  return match;
}

/** Text of the focused element, trimmed. */
export function focusedText(): string {
  return document.activeElement?.textContent?.trim() ?? "";
}

/** Every open `role="menu"` element in document order. */
export function openMenus(): HTMLElement[] {
  return [...document.querySelectorAll('[role="menu"]')].filter(
    (element): element is HTMLElement =>
      element instanceof HTMLElement && element.getAttribute("data-state") === "open",
  );
}

/** Dispatch a mouse click (`detail: 1`) and settle. */
export async function mouseClick(target: Element): Promise<MouseEvent> {
  const event = new MouseEvent("click", { bubbles: true, cancelable: true, detail: 1 });
  target.dispatchEvent(event);
  await settle();
  return event;
}

/** Give an element a fixed layout box for geometry-dependent logic. */
export function stubRect(
  target: Element,
  rect: { x: number; y: number; width: number; height: number },
): void {
  Object.defineProperty(target, "getBoundingClientRect", {
    configurable: true,
    value: () => new DOMRect(rect.x, rect.y, rect.width, rect.height),
  });
}
