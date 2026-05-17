/**
 * useInput - Input handling composable
 */

import { isRef, ref, watch, type Ref } from "@vue/runtime-core";
import { lastCompositionEvent, lastKeyEvent, type KeyEvent } from "../app.js";

export interface Key {
  upArrow: boolean;
  downArrow: boolean;
  leftArrow: boolean;
  rightArrow: boolean;
  pageDown: boolean;
  pageUp: boolean;
  home: boolean;
  end: boolean;
  return: boolean;
  escape: boolean;
  ctrl: boolean;
  shift: boolean;
  tab: boolean;
  backspace: boolean;
  delete: boolean;
  meta: boolean;
  super: boolean;
  hyper: boolean;
  capsLock: boolean;
  numLock: boolean;
  eventType?: "press" | "repeat" | "release";
}

export interface KeyHandler {
  (key: string, modifiers: { ctrl: boolean; alt: boolean; shift: boolean; meta: boolean }): void;
}

export interface UseInputOptions {
  /** Whether to capture input (boolean or Ref<boolean>) */
  active?: boolean | Ref<boolean>;
  /** Whether to capture input (alias for active, boolean or Ref<boolean>) */
  isActive?: boolean | Ref<boolean>;
  /** Ink-compatible input callback */
  handler?: (input: string, key: Key) => void;
  /** Called on key press */
  onKey?: KeyHandler;
  /** Called on character input */
  onChar?: (char: string) => void;
  /** Called on Enter */
  onSubmit?: () => void;
  /** Called on Escape */
  onEscape?: () => void;
  /** Called on arrow keys */
  onArrow?: (direction: "up" | "down" | "left" | "right") => void;
  /** Called when an IME composition starts */
  onCompositionStart?: () => void;
  /** Called when IME preedit text changes */
  onCompositionUpdate?: (text: string, cursor: number) => void;
  /** Called when IME commits text */
  onCompositionEnd?: (text: string) => void;
}

export interface UseInputReturn {
  isActive: Ref<boolean>;
  lastKey: Ref<string | null>;
  enable: () => void;
  disable: () => void;
}

export type InputHandler = (input: string, key: Key) => void;

export interface UseInputHandlerOptions {
  isActive?: boolean | Ref<boolean>;
}

function toRef(value: boolean | Ref<boolean>): Ref<boolean> {
  return isRef(value) ? value : ref(value);
}

function keyName(event: KeyEvent): string {
  return event.char ?? event.key ?? "";
}

function toInkKey(event: KeyEvent): Key {
  const key = event.key;

  return {
    upArrow: key === "up",
    downArrow: key === "down",
    leftArrow: key === "left",
    rightArrow: key === "right",
    pageDown: key === "pagedown" || key === "pageDown",
    pageUp: key === "pageup" || key === "pageUp",
    home: key === "home",
    end: key === "end",
    return: key === "enter" || key === "return",
    escape: key === "escape" || key === "esc",
    ctrl: event.ctrl,
    shift: event.shift,
    tab: key === "tab" || key === "backtab",
    backspace: key === "backspace",
    delete: key === "delete",
    meta: event.meta,
    super: event.super,
    hyper: event.hyper,
    capsLock: event.capsLock,
    numLock: event.numLock,
    eventType: event.eventType,
  };
}

function inputValue(event: KeyEvent, key: Key): string {
  if (event.char) return event.char;
  if (key.return) return "\r";
  if (key.tab) return "\t";
  return "";
}

function handleStructuredOptions(
  event: KeyEvent,
  options: UseInputOptions,
  lastKey: Ref<string | null>,
) {
  const modifiers = {
    ctrl: event.ctrl,
    alt: event.alt,
    shift: event.shift,
    meta: event.meta,
  };
  const pressedKey = keyName(event);
  const inkKey = toInkKey(event);

  if (event.char) {
    lastKey.value = event.char;
    options.onChar?.(event.char);
    options.onKey?.(event.char, modifiers);
    return;
  }

  if (pressedKey) {
    lastKey.value = pressedKey;
    options.onKey?.(pressedKey, modifiers);
  }

  if (inkKey.return) options.onSubmit?.();
  if (inkKey.escape) options.onEscape?.();
  if (inkKey.upArrow) options.onArrow?.("up");
  if (inkKey.downArrow) options.onArrow?.("down");
  if (inkKey.leftArrow) options.onArrow?.("left");
  if (inkKey.rightArrow) options.onArrow?.("right");
}

export function useInput(handler: InputHandler, options?: UseInputHandlerOptions): UseInputReturn;
export function useInput(options?: UseInputOptions): UseInputReturn;
export function useInput(
  handlerOrOptions: InputHandler | UseInputOptions = {},
  handlerOptions: UseInputHandlerOptions = {},
): UseInputReturn {
  const options =
    typeof handlerOrOptions === "function"
      ? { handler: handlerOrOptions, isActive: handlerOptions.isActive }
      : handlerOrOptions;

  const activeSource = options.isActive ?? options.active ?? true;
  const isActive = toRef(activeSource);
  const lastKey = ref<string | null>(null);

  watch(lastKeyEvent, (event) => {
    if (!event || !isActive.value) return;

    const inkKey = toInkKey(event);
    const input = inputValue(event, inkKey);
    const pressedKey = keyName(event);

    lastKey.value = pressedKey || null;
    options.handler?.(input, inkKey);
    handleStructuredOptions(event, options, lastKey);
  });

  watch(lastCompositionEvent, (event) => {
    if (!event || !isActive.value) return;

    if (event.type === "compositionstart") {
      options.onCompositionStart?.();
    } else if (event.type === "compositionupdate") {
      options.onCompositionUpdate?.(event.text, event.cursor);
    } else {
      options.onCompositionEnd?.(event.text);
    }
  });

  const enable = () => {
    isActive.value = true;
  };

  const disable = () => {
    isActive.value = false;
  };

  return {
    isActive,
    lastKey,
    enable,
    disable,
  };
}

/**
 * Shorthand for handling specific key combinations
 */
export function useKeyPress(
  key: string,
  handler: () => void,
  options: { ctrl?: boolean; alt?: boolean; shift?: boolean; meta?: boolean } = {},
) {
  const { ctrl = false, alt = false, shift = false, meta = false } = options;

  useInput({
    onKey: (pressedKey, modifiers) => {
      const matches =
        pressedKey.toLowerCase() === key.toLowerCase() &&
        modifiers.ctrl === ctrl &&
        modifiers.alt === alt &&
        modifiers.shift === shift &&
        modifiers.meta === meta;

      if (matches) {
        handler();
      }
    },
  });
}
