import { toValue, watch } from "vue";
import type { MaybeRefOrGetter } from "vue";

import { normalizeKeyName } from "./magic-keys.ts";
import type { KeyFilter } from "./magic-keys.ts";
import { tryOnScopeDispose } from "./scope.ts";

/** Keyboard event names accepted by {@link onKeyStroke}. */
export type KeyStrokeEventName = "keydown" | "keyup" | "keypress";

/** Options for {@link onKeyStroke}. */
export interface OnKeyStrokeOptions {
  /**
   * Keyboard event to listen for.
   *
   * @default "keydown"
   */
  readonly eventName?: KeyStrokeEventName;

  /**
   * Event target.
   *
   * @default globalThis.window when available
   */
  readonly target?: MaybeRefOrGetter<EventTarget | null | undefined>;

  /**
   * Register a passive listener (the handler then cannot `preventDefault()`).
   *
   * @default false
   */
  readonly passive?: boolean;

  /**
   * Ignore auto-repeated `keydown` events while a key is held.
   *
   * @default false
   */
  readonly dedupe?: MaybeRefOrGetter<boolean>;
}

/**
 * Run a handler when matching keys are pressed.
 *
 * `key` accepts a name, a list of names (normalized like combos, so `"esc"`
 * matches `Escape` and `"a"` matches `KeyA`), a predicate, or `true` for every
 * key. The listener follows the reactive target and is removed with the
 * owning scope or the returned stop function; nothing is registered during
 * server rendering.
 *
 * @param key Key filter, or `true` for any key.
 * @param handler Called with the matching `KeyboardEvent`.
 * @param options Event name, target, passivity, and dedupe.
 * @default options {}
 * @returns Stop function.
 */
export function onKeyStroke(
  key: KeyFilter | true,
  handler: (event: KeyboardEvent) => void,
  options: OnKeyStrokeOptions = {},
): () => void {
  const eventName = options.eventName ?? "keydown";
  const names =
    key === true || typeof key === "function"
      ? undefined
      : new Set((typeof key === "string" ? [key] : key).map((name) => normalizeKeyName(name)));
  const matches = (event: KeyboardEvent): boolean => {
    if (key === true) return true;
    if (typeof key === "function") return key(event);
    return (
      names !== undefined &&
      (names.has(normalizeKeyName(event.key)) || names.has(normalizeKeyName(event.code)))
    );
  };
  const listener = (event: Event): void => {
    if (!isKeyboardEvent(event)) return;
    if (event.repeat && toValue(options.dedupe)) return;
    if (matches(event)) handler(event);
  };

  const stop = watch(
    () => (options.target === undefined ? browserWindow() : toValue(options.target)),
    (target, _previous, onCleanup) => {
      if (!target) return;
      target.addEventListener(eventName, listener, { passive: options.passive ?? false });
      onCleanup(() => target.removeEventListener(eventName, listener));
    },
    { immediate: true },
  );
  const dispose = (): void => stop.stop();
  tryOnScopeDispose(dispose);
  return dispose;
}

/**
 * {@link onKeyStroke} for `keydown`.
 *
 * @param key Key filter, or `true` for any key.
 * @param handler Called with the matching `KeyboardEvent`.
 * @param options Target, passivity, and dedupe.
 * @default options {}
 * @returns Stop function.
 */
export function onKeyDown(
  key: KeyFilter | true,
  handler: (event: KeyboardEvent) => void,
  options: Omit<OnKeyStrokeOptions, "eventName"> = {},
): () => void {
  return onKeyStroke(key, handler, { ...options, eventName: "keydown" });
}

/**
 * {@link onKeyStroke} for `keyup`.
 *
 * @param key Key filter, or `true` for any key.
 * @param handler Called with the matching `KeyboardEvent`.
 * @param options Target, passivity, and dedupe.
 * @default options {}
 * @returns Stop function.
 */
export function onKeyUp(
  key: KeyFilter | true,
  handler: (event: KeyboardEvent) => void,
  options: Omit<OnKeyStrokeOptions, "eventName"> = {},
): () => void {
  return onKeyStroke(key, handler, { ...options, eventName: "keyup" });
}

function isKeyboardEvent(event: Event): event is KeyboardEvent {
  return "key" in event && "code" in event;
}

function browserWindow(): EventTarget | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
