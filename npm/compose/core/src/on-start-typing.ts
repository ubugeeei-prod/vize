import { toValue, watch } from "vue";
import type { MaybeRefOrGetter } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Document-like capability observed by {@link onStartTyping}. */
export interface StartTypingHost extends EventTarget {
  /** Currently focused element. */
  readonly activeElement: Element | null;
}

/** Options for {@link onStartTyping}. */
export interface OnStartTypingOptions {
  /**
   * Reactive document capability for alternate runtimes and tests.
   *
   * @default globalThis.document when available
   */
  readonly host?: MaybeRefOrGetter<StartTypingHost | null | undefined>;
}

/**
 * Whether a keyboard event would type a printable character.
 *
 * Single-character keys (letters, digits, punctuation; not space) without
 * `ctrl`/`meta`/`alt` qualify.
 *
 * @param event Keyboard event.
 * @returns Whether the event starts typing.
 */
export function isTypingKeystroke(event: KeyboardEvent): boolean {
  if (event.ctrlKey || event.metaKey || event.altKey) return false;
  return event.key.length === 1 && event.key !== " ";
}

/**
 * Call a handler when the user starts typing while no editable element has
 * focus — typically to focus a search field.
 *
 * Editable means `input`, `textarea`, `select`, or any element with
 * `isContentEditable`, and elements with `role="textbox"`/`"searchbox"`/`"combobox"`.
 * Nothing is registered during server rendering; the listener is removed with
 * the owning scope or the returned stop function.
 *
 * @param handler Called with the initiating `keydown` event.
 * @param options Document capability.
 * @default options {}
 * @returns Stop function.
 */
export function onStartTyping(
  handler: (event: KeyboardEvent) => void,
  options: OnStartTypingOptions = {},
): () => void {
  const stop = watch(
    () => (options.host === undefined ? browserTypingHost() : toValue(options.host)),
    (host, _previous, onCleanup) => {
      if (!host) return;
      const onKeyDown = (event: Event): void => {
        if (!isKeyboardEvent(event) || event.defaultPrevented) return;
        if (isEditable(host.activeElement) || !isTypingKeystroke(event)) return;
        handler(event);
      };
      host.addEventListener("keydown", onKeyDown, { passive: true });
      onCleanup(() => host.removeEventListener("keydown", onKeyDown));
    },
    { immediate: true },
  );
  const dispose = (): void => stop.stop();
  tryOnScopeDispose(dispose);
  return dispose;
}

function isEditable(element: Element | null): boolean {
  if (!element) return false;
  const tag = element.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
  if ("isContentEditable" in element && element.isContentEditable === true) return true;
  const role = element.getAttribute?.("role");
  return role === "textbox" || role === "searchbox" || role === "combobox";
}

function isKeyboardEvent(event: Event): event is KeyboardEvent {
  return "key" in event && "code" in event;
}

function browserTypingHost(): StartTypingHost | undefined {
  return typeof document !== "undefined" ? document : undefined;
}
