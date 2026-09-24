import { computed, shallowRef, watch } from "vue";
import type { ComputedRef, MaybeRefOrGetter, WritableComputedRef } from "vue";

import { resolveElement } from "./element-target.ts";
import type { MaybeElementTarget } from "./element-target.ts";
import { useMediaQuery } from "./media-query.ts";
import type { MediaQueryHost } from "./media-query.ts";
import { tryOnScopeDispose } from "./scope.ts";
import type { StorageLike } from "./use-storage.ts";

/** Built-in color modes. */
export type BasicColorMode = "light" | "dark";

/** Selectable mode: a built-in mode, a custom mode, or `"auto"` (follow the system). */
export type ColorModeValue<Custom extends string = never> = BasicColorMode | Custom | "auto";

/**
 * Synchronous key-value persistence used by {@link useColorMode}.
 *
 * The read/write subset of `StorageLike` from `./use-storage`, so
 * `localStorage`, `sessionStorage`, and the same adapters work for both. For
 * hydration-stable SSR pass a cookie-backed adapter that reads the request
 * cookie on the server and `document.cookie` on the client.
 */
export type ColorModeStorage = Pick<StorageLike, "getItem" | "setItem">;

/** Options for {@link useColorMode}. */
export interface UseColorModeOptions<Custom extends string = never> {
  /**
   * Class name (or attribute value) written for each mode. Custom modes must
   * be listed here.
   *
   * @default { light: "light", dark: "dark" }
   */
  readonly modes?: Readonly<Partial<Record<BasicColorMode, string>> & Record<Custom, string>>;

  /**
   * `"class"` toggles class names; any other value is used as an attribute
   * name (e.g. `"data-theme"`).
   *
   * @default "class"
   */
  readonly attribute?: string;

  /**
   * Element that receives the class/attribute.
   *
   * @default document.documentElement when available
   */
  readonly target?: MaybeElementTarget;

  /**
   * Mode used when nothing is stored.
   *
   * @default "auto"
   */
  readonly initialValue?: ColorModeValue<Custom>;

  /**
   * Storage key. `null` disables persistence.
   *
   * @default "vize-color-mode"
   */
  readonly storageKey?: string | null;

  /**
   * Persistence adapter. `null` disables persistence.
   *
   * @default globalThis.window.localStorage when available
   */
  readonly storage?: ColorModeStorage | null;

  /**
   * System preference assumed while no media capability exists (server rendering).
   *
   * @default "light"
   */
  readonly ssrSystem?: BasicColorMode;

  /**
   * Suppress CSS transitions for one frame while switching modes.
   *
   * @default true
   */
  readonly disableTransition?: boolean;

  /**
   * Media capability used to read `prefers-color-scheme`.
   *
   * @default globalThis.window when available
   */
  readonly host?: MaybeRefOrGetter<MediaQueryHost | null | undefined>;
}

/** Reactive color mode returned by {@link useColorMode}. */
export interface ColorModeControls<Custom extends string> {
  /** Selected mode, including `"auto"`. Assigning persists and applies it. */
  readonly mode: WritableComputedRef<ColorModeValue<Custom>>;
  /** System preference from `prefers-color-scheme`. */
  readonly system: ComputedRef<BasicColorMode>;
  /** Resolved mode (`"auto"` replaced by the system preference). */
  readonly state: ComputedRef<BasicColorMode | Custom>;
  /** Value read from storage at creation (or the initial value). */
  readonly stored: ComputedRef<ColorModeValue<Custom>>;
}

/**
 * Reactive color mode with persistence and DOM application.
 *
 * The stored mode is read synchronously during setup through the injected
 * storage, so a cookie adapter yields identical server and client renders.
 * On the client the resolved mode is written to the target's class list (or
 * attribute) after each change, optionally with transitions suppressed.
 * Nothing touches the DOM during server rendering.
 *
 * @param options Modes, target, persistence, SSR fallback, and capabilities.
 * @default options {}
 * @returns Writable mode plus system and resolved state.
 */
export function useColorMode<const Custom extends string = never>(
  options: UseColorModeOptions<Custom> = {},
): ColorModeControls<Custom> {
  const modes: Readonly<Record<string, string>> = {
    light: "light",
    dark: "dark",
    ...options.modes,
  };
  const attribute = options.attribute ?? "class";
  const storageKey = options.storageKey === undefined ? "vize-color-mode" : options.storageKey;
  const storage = options.storage === undefined ? browserStorage() : options.storage;
  const initial = options.initialValue ?? "auto";
  const isMode = (value: string | null): value is ColorModeValue<Custom> =>
    value === "auto" || (value !== null && Object.hasOwn(modes, value));

  const read = (): ColorModeValue<Custom> => {
    if (!storage || storageKey === null) return initial;
    try {
      const value = storage.getItem(storageKey);
      return isMode(value) ? value : initial;
    } catch {
      return initial;
    }
  };
  const storedValue = read();
  const selectedRaw = shallowRef<string>(storedValue);
  const selected = computed<ColorModeValue<Custom>>(() =>
    isMode(selectedRaw.value) ? selectedRaw.value : initial,
  );
  const prefersDark = useMediaQuery(
    "(prefers-color-scheme: dark)",
    options.host === undefined
      ? { ssrValue: options.ssrSystem === "dark" }
      : { ssrValue: options.ssrSystem === "dark", host: options.host },
  );
  const system = computed<BasicColorMode>(() => (prefersDark.value ? "dark" : "light"));
  const state = computed<BasicColorMode | Custom>(() =>
    selected.value === "auto" ? system.value : selected.value,
  );

  const target = (): Element | null =>
    options.target === undefined
      ? typeof document !== "undefined"
        ? document.documentElement
        : null
      : resolveElement(options.target);

  const stop = watch(
    [state, target],
    ([resolved, element]) => {
      if (!element) return;
      const restore =
        options.disableTransition === false ? undefined : suppressTransitions(element);
      const value = modes[resolved] ?? resolved;
      if (attribute === "class") {
        for (const name of Object.values(modes)) {
          for (const token of name.split(/\s+/)) if (token) element.classList.remove(token);
        }
        for (const token of value.split(/\s+/)) if (token) element.classList.add(token);
      } else {
        element.setAttribute(attribute, value);
      }
      restore?.();
    },
    { immediate: true, flush: "post" },
  );
  tryOnScopeDispose(() => stop.stop());

  const mode = computed<ColorModeValue<Custom>>({
    get: () => selected.value,
    set: (next) => {
      selectedRaw.value = next;
      if (!storage || storageKey === null) return;
      try {
        storage.setItem(storageKey, next);
      } catch {
        // Persistence is best-effort (quota, privacy mode); the in-memory mode still applies.
      }
    },
  });

  return { mode, system, state, stored: computed(() => storedValue) };
}

/** Options for {@link useDark}. */
export interface UseDarkOptions extends Omit<UseColorModeOptions, "modes"> {
  /**
   * Class/attribute value applied in dark mode.
   *
   * @default "dark"
   */
  readonly valueDark?: string;

  /**
   * Class/attribute value applied in light mode.
   *
   * @default ""
   */
  readonly valueLight?: string;
}

/**
 * Boolean dark-mode switch built on {@link useColorMode}.
 *
 * Setting the ref to the current system preference stores `"auto"`, so the
 * page keeps following the system until the user picks the opposite mode.
 *
 * @param options Dark/light values plus {@link useColorMode} options.
 * @default options {}
 * @returns Writable dark flag.
 */
export function useDark(options: UseDarkOptions = {}): WritableComputedRef<boolean> {
  const { valueDark = "dark", valueLight = "", ...rest } = options;
  const colorMode = useColorMode({ ...rest, modes: { dark: valueDark, light: valueLight } });
  return computed({
    get: () => colorMode.state.value === "dark",
    set: (dark: boolean) => {
      const preferred: BasicColorMode = dark ? "dark" : "light";
      colorMode.mode.value = preferred === colorMode.system.value ? "auto" : preferred;
    },
  });
}

function suppressTransitions(element: Element): (() => void) | undefined {
  const document = element.ownerDocument;
  const head = document?.head;
  if (!head || typeof document.createElement !== "function") return undefined;
  const style = document.createElement("style");
  style.textContent = "*,*::before,*::after{transition:none!important}";
  head.appendChild(style);
  return () => {
    void document.defaultView?.getComputedStyle(style).opacity;
    style.remove();
  };
}

function browserStorage(): ColorModeStorage | null {
  if (typeof window === "undefined") return null;
  try {
    return window.localStorage;
  } catch {
    return null;
  }
}
