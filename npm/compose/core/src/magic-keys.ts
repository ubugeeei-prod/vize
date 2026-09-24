import { computed, reactive, readonly, ref, toValue, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, Ref } from "vue";

import { tryOnScopeDispose } from "./scope.ts";

/** Modifier key names recognized in combos (after alias resolution). */
export type ModifierKeyName = "control" | "shift" | "alt" | "meta";

/**
 * Validate a key combo string at the type level.
 *
 * Combos join lowercase-insensitive key names with `+` or `_`
 * (`"ctrl+s"`, `"shift_arrowup"`). Empty segments such as `"ctrl+"` or
 * `"+s"` resolve to `never`, turning typos into compile errors. Non-literal
 * `string` values are accepted unchanged.
 */
export type KeyCombo<Combo extends string> = string extends Combo
  ? string
  : Combo extends ""
    ? never
    : Combo extends `${infer Head}${"+" | "_"}${infer Tail}`
      ? Head extends ""
        ? never
        : Tail extends ""
          ? never
          : KeyCombo<Tail> extends never
            ? never
            : Combo
      : Combo;

/** Default aliases resolved before matching combos. */
export const DEFAULT_KEY_ALIASES = {
  ctrl: "control",
  command: "meta",
  cmd: "meta",
  option: "alt",
  esc: "escape",
  up: "arrowup",
  down: "arrowdown",
  left: "arrowleft",
  right: "arrowright",
  " ": "space",
  spacebar: "space",
} as const satisfies Readonly<Record<string, string>>;

/** Options shared by the keyboard composables. */
export interface UseKeyboardTargetOptions {
  /**
   * Event target that receives `keydown`/`keyup`.
   *
   * @default globalThis.window when available
   */
  readonly target?: MaybeRefOrGetter<EventTarget | null | undefined>;

  /**
   * Register passive listeners.
   *
   * @default true
   */
  readonly passive?: boolean;
}

/** Options for {@link useMagicKeys}. */
export interface UseMagicKeysOptions extends UseKeyboardTargetOptions {
  /**
   * Extra aliases merged over {@link DEFAULT_KEY_ALIASES}. Keys and values are
   * matched case-insensitively.
   *
   * @default {}
   */
  readonly aliasMap?: Readonly<Record<string, string>>;

  /**
   * Called for every `keydown`/`keyup` after the pressed set has been
   * updated. Useful for `preventDefault()` with `passive: false`.
   *
   * @default undefined
   */
  readonly onEventFired?: (event: KeyboardEvent) => void;
}

/** Reactive keyboard state returned by {@link useMagicKeys}. */
export interface MagicKeys {
  /** Normalized names (`event.key` and `event.code`, lowercase) currently pressed. */
  readonly current: ReadonlySet<string>;

  /**
   * Proxy of combo refs created lazily on property access, e.g.
   * `const { shift, ctrl_s } = keys.combos`.
   */
  readonly combos: Readonly<Record<string, ComputedRef<boolean>>>;

  /**
   * Create (or reuse) a ref that is `true` while every key of the combo is
   * pressed.
   *
   * @param combo Key combo such as `"ctrl+shift+p"`.
   * @returns Readonly computed ref.
   */
  readonly isPressed: <const Combo extends string>(
    combo: Combo & KeyCombo<Combo>,
  ) => ComputedRef<boolean>;

  /** Forget every pressed key (for example after a modal steals focus). */
  readonly reset: () => void;
}

const modifierNames: ReadonlySet<string> = new Set<ModifierKeyName>([
  "control",
  "shift",
  "alt",
  "meta",
]);

/**
 * Normalize a key name the way combos are matched.
 *
 * @param name Raw `KeyboardEvent.key`/`code` or combo segment.
 * @param aliases Alias table (keys lowercased).
 * @default aliases DEFAULT_KEY_ALIASES
 * @returns Lowercase, alias-resolved name.
 */
export function normalizeKeyName(
  name: string,
  aliases: Readonly<Record<string, string>> = DEFAULT_KEY_ALIASES,
): string {
  const lower = name.toLowerCase();
  return (aliases[lower] ?? lower).toLowerCase();
}

/**
 * Track pressed keys and expose reactive key combos.
 *
 * Both `event.key` and `event.code` are recorded (lowercased), so `"a"` and
 * `"keya"` both match. Because some platforms (notably macOS with `meta`)
 * swallow `keyup` for other keys while a modifier is held, releasing a
 * modifier clears every non-modifier key; losing window focus clears
 * everything. Nothing is pressed during server rendering, and listeners are
 * removed with the owning reactive scope.
 *
 * @param options Target, passivity, aliases, and event hook.
 * @default options {}
 * @returns Pressed set, lazily created combo refs, and controls.
 */
export function useMagicKeys(options: UseMagicKeysOptions = {}): MagicKeys {
  const aliases: Record<string, string> = { ...DEFAULT_KEY_ALIASES };
  for (const [alias, key] of Object.entries(options.aliasMap ?? {})) {
    aliases[alias.toLowerCase()] = key.toLowerCase();
  }
  const current = reactive(new Set<string>());
  const cache = new Map<string, ComputedRef<boolean>>();

  const namesOf = (event: KeyboardEvent): string[] => {
    const names = new Set<string>();
    if (event.key) names.add(normalizeKeyName(event.key, aliases));
    if (event.code) names.add(normalizeKeyName(event.code, aliases));
    return [...names];
  };
  const reset = (): void => current.clear();
  const onKey = (event: Event): void => {
    if (!isKeyboardEvent(event)) return;
    const names = namesOf(event);
    if (event.type === "keydown") {
      for (const name of names) current.add(name);
    } else {
      for (const name of names) current.delete(name);
      if (names.some((name) => modifierNames.has(name))) {
        for (const name of current) if (!modifierNames.has(name)) current.delete(name);
      }
    }
    options.onEventFired?.(event);
  };

  const stop = watch(
    () => (options.target === undefined ? browserWindow() : toValue(options.target)),
    (target, _previous, onCleanup) => {
      if (!target) return;
      const listenerOptions: AddEventListenerOptions = { passive: options.passive ?? true };
      target.addEventListener("keydown", onKey, listenerOptions);
      target.addEventListener("keyup", onKey, listenerOptions);
      target.addEventListener("blur", reset, { passive: true });
      onCleanup(() => {
        target.removeEventListener("keydown", onKey);
        target.removeEventListener("keyup", onKey);
        target.removeEventListener("blur", reset);
        reset();
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  const comboRef = (combo: string): ComputedRef<boolean> => {
    const key = combo.toLowerCase();
    const cached = cache.get(key);
    if (cached) return cached;
    const parts = key
      .split(/[+_]/)
      .filter((part) => part !== "")
      .map((part) => normalizeKeyName(part, aliases));
    const pressed = computed(() => parts.length > 0 && parts.every((part) => current.has(part)));
    cache.set(key, pressed);
    return pressed;
  };

  const combos = new Proxy<Record<string, ComputedRef<boolean>>>(
    {},
    {
      get: (_target, property) => (typeof property === "string" ? comboRef(property) : undefined),
      has: (_target, property) => typeof property === "string",
    },
  );

  return {
    current: readonly(current),
    combos,
    isPressed: (combo) => comboRef(combo),
    reset,
  };
}

/** Key filter accepted by {@link useKeyPressed}. */
export type KeyFilter = string | readonly string[] | ((event: KeyboardEvent) => boolean);

/**
 * Track whether any key matching a filter is held down.
 *
 * String filters are normalized like combos (`"esc"` matches `Escape`) and
 * compared against both `event.key` and `event.code`. The state resets on
 * window blur and during server rendering is always `false`.
 *
 * @param filter Key name(s) or predicate.
 * @param options Target and passivity.
 * @default options {}
 * @returns Readonly ref that is `true` while a matching key is held.
 */
export function useKeyPressed(
  filter: KeyFilter,
  options: UseKeyboardTargetOptions = {},
): Readonly<Ref<boolean>> {
  const held = new Set<string>();
  const pressed = ref(false);
  const names =
    typeof filter === "function"
      ? undefined
      : new Set(
          (typeof filter === "string" ? [filter] : filter).map((name) => normalizeKeyName(name)),
        );
  const matches = (event: KeyboardEvent): boolean =>
    names === undefined
      ? typeof filter === "function" && filter(event)
      : names.has(normalizeKeyName(event.key)) || names.has(normalizeKeyName(event.code));
  const identity = (event: KeyboardEvent): string => event.code || event.key;

  const onDown = (event: Event): void => {
    if (!isKeyboardEvent(event) || !matches(event)) return;
    held.add(identity(event));
    pressed.value = true;
  };
  const onUp = (event: Event): void => {
    if (!isKeyboardEvent(event)) return;
    held.delete(identity(event));
    pressed.value = held.size > 0;
  };
  const reset = (): void => {
    held.clear();
    pressed.value = false;
  };

  const stop = watch(
    () => (options.target === undefined ? browserWindow() : toValue(options.target)),
    (target, _previous, onCleanup) => {
      if (!target) return;
      const listenerOptions: AddEventListenerOptions = { passive: options.passive ?? true };
      target.addEventListener("keydown", onDown, listenerOptions);
      target.addEventListener("keyup", onUp, listenerOptions);
      target.addEventListener("blur", reset, { passive: true });
      onCleanup(() => {
        target.removeEventListener("keydown", onDown);
        target.removeEventListener("keyup", onUp);
        target.removeEventListener("blur", reset);
        reset();
      });
    },
    { immediate: true },
  );
  tryOnScopeDispose(() => stop.stop());

  return readonly(pressed);
}

function isKeyboardEvent(event: Event): event is KeyboardEvent {
  return "key" in event && "code" in event;
}

function browserWindow(): EventTarget | undefined {
  return typeof window !== "undefined" ? window : undefined;
}
