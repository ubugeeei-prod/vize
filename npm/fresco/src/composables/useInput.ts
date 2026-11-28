/**
 * useInput - Input handling composable
 */

import { isRef, onUnmounted, ref, watch, type Ref } from "vue";
import { lastCompositionEvent, lastKeyEvent, lastPasteEvent, type KeyEvent } from "../app.js";
import { hasActivePasteHandlers } from "./usePaste.js";
import { useStreamsContext } from "./useStreams.js";

/** Normalized compatibility flags for a delivered terminal key event. */
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

/** Receive a normalized key name and exact modifier state. */
export interface KeyHandler {
  (key: string, modifiers: { ctrl: boolean; alt: boolean; shift: boolean; meta: boolean }): void;
}

/** Activation and callbacks for key, character, paste, and composition input. */
export interface UseInputOptions {
  /** Activation flag used when isActive is omitted. @default true */
  active?: boolean | Ref<boolean>;
  /** Activation flag that takes precedence over active. @default active ?? true */
  isActive?: boolean | Ref<boolean>;
  /** Receive printable input and normalized compatibility key flags. @default undefined */
  handler?: (input: string, key: Key) => void;
  /** Receive normalized key names with exact modifier state. @default undefined */
  onKey?: KeyHandler;
  /** Receive printable text without command modifiers. @default undefined */
  onChar?: (char: string) => void;
  /** Receive the normalized Enter key. @default undefined */
  onSubmit?: () => void;
  /** Receive the normalized Escape key. @default undefined */
  onEscape?: () => void;
  /** Receive normalized arrow directions. @default undefined */
  onArrow?: (direction: "up" | "down" | "left" | "right") => void;
  /** Receive the start of IME preedit. @default undefined */
  onCompositionStart?: () => void;
  /** Receive an IME preedit update. @default undefined */
  onCompositionUpdate?: (text: string, cursor: number) => void;
  /** Receive committed IME composition text. @default undefined */
  onCompositionEnd?: (text: string) => void;
  /** Whether this handler owns Ctrl+C delivery. @default false */
  captureCtrlC?: boolean;
}

/** Mutable activation and the latest delivered key or printable text. */
export interface UseInputReturn {
  isActive: Ref<boolean>;
  lastKey: Ref<string | null>;
  enable: () => void;
  disable: () => void;
}

/** Receive printable input with normalized compatibility key flags. */
export type InputHandler = (input: string, key: Key) => void;

/** Activation for the callback overload of useInput. */
export interface UseInputHandlerOptions {
  /** Whether the callback overload receives terminal input. @default true */
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

function emptyKey(): Key {
  return {
    upArrow: false,
    downArrow: false,
    leftArrow: false,
    rightArrow: false,
    pageDown: false,
    pageUp: false,
    home: false,
    end: false,
    return: false,
    escape: false,
    ctrl: false,
    shift: false,
    tab: false,
    backspace: false,
    delete: false,
    meta: false,
    super: false,
    hyper: false,
    capsLock: false,
    numLock: false,
  };
}

function inputValue(event: KeyEvent, key: Key): string {
  if (event.char) return event.char;
  if (key.return) return "\r";
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
    // Modified character chords represent commands, not text insertion. AltGr
    // terminals report their committed text without these command modifiers.
    if (!event.ctrl && !event.alt && !event.meta) options.onChar?.(event.char);
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

/** Observe normalized terminal input while active and release input watchers and mode requests when the component unmounts. */
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
  const streams = useStreamsContext();
  let rawModeEnabled = false;

  const syncRawMode = (isEnabled: boolean) => {
    if (rawModeEnabled === isEnabled) return;
    streams.setRawMode(isEnabled);
    rawModeEnabled = isEnabled;
  };

  watch(lastKeyEvent, (event) => {
    if (!event || !isActive.value) return;

    const inkKey = toInkKey(event);
    const input = inputValue(event, inkKey);
    const pressedKey = keyName(event);

    if (input === "c" && inkKey.ctrl && streams.internal_exitOnCtrlC && !options.captureCtrlC)
      return;

    lastKey.value = pressedKey || null;
    options.handler?.(input, inkKey);
    handleStructuredOptions(event, options, lastKey);
  });

  watch(lastPasteEvent, (event) => {
    if (!event || !isActive.value || hasActivePasteHandlers()) return;

    lastKey.value = event.text;
    options.handler?.(event.text, emptyKey());
    options.onChar?.(event.text);
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

  watch(isActive, syncRawMode, { immediate: true });
  onUnmounted(() => syncRawMode(false));

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
