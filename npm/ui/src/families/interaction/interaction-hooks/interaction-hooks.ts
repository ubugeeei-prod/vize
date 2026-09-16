import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { createHover } from "../hover/hover.ts";
import { createPress } from "../press/press.ts";
import { createFocusRing, createFocusWithin } from "../../accessibility/focus/focus.ts";
import { createInteractionModalityTracker } from "../../accessibility/interaction-modality/interaction-modality.ts";
import type { HoverOptions } from "../hover/hover.ts";
import type { PressOptions } from "../press/press.ts";
import type { FocusRingOptions, FocusWithinOptions } from "../../accessibility/focus/focus.ts";
import type {
  InteractionModality,
  InteractionModalityOptions,
} from "../../accessibility/interaction-modality/interaction-modality.ts";
import type {
  InteractionFeatureOptions,
  InteractionHooksCancelResult,
  InteractionHooksController,
  InteractionHooksOptions,
  InteractionHooksProps,
  InteractionShortcutBinding,
  InteractionShortcutController,
  InteractionShortcutOptions,
} from "./interaction-hooks-types.ts";

const setupDiagnostic = "VIZE_UI_INTERACTION_HOOKS_SETUP";
const shortcutDiagnostic = "VIZE_UI_INTERACTION_HOOKS_SHORTCUT";

const falseState = shallowReadonly(shallowRef(false));
const nullModalityState = shallowReadonly(shallowRef<InteractionModality | null>(null));

function withDisabled<
  Options extends { readonly isDisabled?: MaybeRefOrGetter<boolean | undefined> },
>(
  feature: Options | undefined,
  isDisabled: MaybeRefOrGetter<boolean | undefined> | undefined,
): Options {
  if (isDisabled === undefined) return (feature ?? {}) as Options;
  return { isDisabled, ...feature } as Options;
}

function enabled<Options>(
  feature: InteractionFeatureOptions<Options>,
): feature is Options | undefined {
  return feature !== false;
}

function enabledWhenRequested<Options>(
  feature: InteractionFeatureOptions<Options>,
): feature is Options {
  return feature !== undefined && feature !== false;
}

function readonlyState(
  controller: { readonly value: boolean } | Readonly<ShallowRef<boolean>> | null,
) {
  return controller ?? falseState;
}

function readonlyModalityState(
  controller: Readonly<ShallowRef<InteractionModality | null>> | null,
) {
  return controller ?? nullModalityState;
}

function mergeProps(
  ...sources: Array<Readonly<Record<string, unknown>> | null | undefined>
): Readonly<Record<string, unknown>> {
  const merged: Record<string, unknown> = {};
  for (const source of sources) {
    if (!source) continue;
    for (const [key, value] of Object.entries(source)) {
      if (value === undefined) continue;
      const current = merged[key];
      if (key.startsWith("on") && typeof current === "function" && typeof value === "function") {
        merged[key] = (event: Event) => {
          (current as (event: Event) => void)(event);
          (value as (event: Event) => void)(event);
        };
        continue;
      }
      merged[key] = value;
    }
  }
  return Object.freeze(merged);
}

interface NormalizedShortcut {
  readonly key: string;
  readonly altKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;
}

interface ShortcutEntry {
  readonly binding: InteractionShortcutBinding;
  readonly normalized: NormalizedShortcut;
}

function readShortcutDisabled(source: InteractionShortcutOptions["isDisabled"]): boolean {
  const value = toMaybeValue(source);
  if (value === undefined) return false;
  if (typeof value !== "boolean") {
    throw new TypeError(`${shortcutDiagnostic}: isDisabled must resolve to a boolean`);
  }
  return value;
}

function toMaybeValue<T>(source: MaybeRefOrGetter<T> | undefined): T | undefined {
  if (typeof source === "function") return (source as () => T)();
  if (source && typeof source === "object" && "value" in source) {
    return (source as { readonly value: T }).value;
  }
  return source;
}

function normalizeKey(key: string): string {
  return key.length === 1 ? key.toLowerCase() : key;
}

function normalizeShortcut(
  shortcut: string,
  platform: InteractionShortcutOptions["platform"],
): NormalizedShortcut {
  const normalized: NormalizedShortcut = {
    key: "",
    altKey: false,
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
  };
  const parts = shortcut
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);
  for (const part of parts) {
    const lower = part.toLowerCase();
    if (lower === "mod") {
      (normalized as { ctrlKey: boolean; metaKey: boolean })[
        platform === "apple" ? "metaKey" : "ctrlKey"
      ] = true;
    } else if (lower === "ctrl" || lower === "control") {
      (normalized as { ctrlKey: boolean }).ctrlKey = true;
    } else if (lower === "alt" || lower === "option") {
      (normalized as { altKey: boolean }).altKey = true;
    } else if (lower === "cmd" || lower === "meta") {
      (normalized as { metaKey: boolean }).metaKey = true;
    } else if (lower === "shift") {
      (normalized as { shiftKey: boolean }).shiftKey = true;
    } else if (normalized.key === "") {
      (normalized as { key: string }).key = normalizeKey(part);
    } else {
      throw new TypeError(`${shortcutDiagnostic}: shortcut has more than one key`);
    }
  }
  if (normalized.key === "") {
    throw new TypeError(`${shortcutDiagnostic}: shortcut must include a key`);
  }
  return Object.freeze(normalized);
}

function shortcutMatches(event: KeyboardEvent, shortcut: NormalizedShortcut): boolean {
  return (
    normalizeKey(event.key) === shortcut.key &&
    event.altKey === shortcut.altKey &&
    event.ctrlKey === shortcut.ctrlKey &&
    event.metaKey === shortcut.metaKey &&
    event.shiftKey === shortcut.shiftKey
  );
}

function createInteractionShortcutController(
  options: InteractionShortcutOptions = {},
): InteractionShortcutController {
  const platform = options.platform ?? "standard";
  const entries: ShortcutEntry[] = [];
  let disposed = false;
  const assertActive = () => {
    if (disposed) throw new Error(`${shortcutDiagnostic}: controller has been disposed`);
  };
  const register = (binding: InteractionShortcutBinding) => {
    assertActive();
    if (typeof binding.handler !== "function") {
      throw new TypeError(`${shortcutDiagnostic}: shortcut handler must be a function`);
    }
    const entry = Object.freeze({
      binding,
      normalized: normalizeShortcut(binding.shortcut, platform),
    });
    entries.push(entry);
    return () => {
      const index = entries.indexOf(entry);
      if (index !== -1) entries.splice(index, 1);
    };
  };
  options.bindings?.forEach(register);
  const input = (event: KeyboardEvent) => {
    assertActive();
    if (readShortcutDisabled(options.isDisabled) || event.isComposing) return;
    for (let index = entries.length - 1; index >= 0; index--) {
      const entry = entries[index];
      if (!entry || (event.repeat && entry.binding.allowRepeat !== true)) continue;
      if (!shortcutMatches(event, entry.normalized)) continue;
      if (entry.binding.preventDefault !== false) event.preventDefault();
      entry.binding.handler(
        Object.freeze({
          shortcut: entry.binding.shortcut,
          description: entry.binding.description ?? null,
          originalEvent: event,
        }),
      );
      return;
    }
  };
  return Object.freeze({
    shortcutProps: Object.freeze({ onKeydown: input }),
    register,
    reset: () => false,
    getBindings: () => Object.freeze(entries.map((entry) => entry.binding)),
    dispose: () => {
      if (disposed) return;
      disposed = true;
      entries.splice(0);
    },
  });
}

function collectErrors(action: () => void, errors: unknown[]): void {
  try {
    action();
  } catch (error) {
    errors.push(error);
  }
}

function surfaceErrors(errors: readonly unknown[]): void {
  if (errors.length === 1) throw errors[0];
  if (errors.length > 1) throw new AggregateError(errors, "Interaction hook cleanup failed");
}

export function createInteractionHooks<const Options extends InteractionHooksOptions = {}>(
  options: Options = {} as Options,
): InteractionHooksController<Options> {
  const modalityTracker = enabledWhenRequested(options.modality)
    ? createInteractionModalityTracker(options.modality as InteractionModalityOptions)
    : null;
  const press = enabled(options.press)
    ? createPress(withDisabled(options.press, options.isDisabled))
    : null;
  const hover = enabled(options.hover)
    ? createHover(withDisabled(options.hover, options.isDisabled))
    : null;
  const focusRing = enabled(options.focusRing)
    ? createFocusRing(withDisabled(options.focusRing, options.isDisabled))
    : null;
  const focusWithin = enabled(options.focusWithin)
    ? createFocusWithin(withDisabled(options.focusWithin, options.isDisabled))
    : null;
  const shortcutController = enabledWhenRequested(options.shortcuts)
    ? createInteractionShortcutController(withDisabled(options.shortcuts, options.isDisabled))
    : null;

  const interactionProps = mergeProps(
    press?.pressProps,
    hover?.hoverProps,
    focusRing?.focusProps,
    focusWithin?.focusProps,
    shortcutController?.shortcutProps,
  ) as Readonly<InteractionHooksProps<Options>>;

  return Object.freeze({
    modalityTracker,
    shortcutController,
    focusRing,
    focusWithin,
    hover,
    press,
    interactionProps,
    currentModality: readonlyModalityState(modalityTracker?.modality ?? null),
    isFocused: readonlyState(focusRing?.isFocused ?? null),
    isFocusVisible: readonlyState(focusRing?.isFocusVisible ?? null),
    isFocusWithin: readonlyState(focusWithin?.isFocused ?? null),
    isHovered: readonlyState(hover?.isHovered ?? null),
    isPressed: readonlyState(press?.isPressed ?? null),
    cancel(): InteractionHooksCancelResult {
      return Object.freeze({
        focusRing: focusRing?.cancel() ?? false,
        focusWithin: focusWithin?.cancel() ?? false,
        hover: hover?.cancel() ?? false,
        press: press?.cancel() ?? false,
        shortcuts: shortcutController?.reset() ?? false,
      });
    },
    dispose(): void {
      const errors: unknown[] = [];
      collectErrors(() => press?.dispose(), errors);
      collectErrors(() => hover?.dispose(), errors);
      collectErrors(() => focusRing?.dispose(), errors);
      collectErrors(() => focusWithin?.dispose(), errors);
      collectErrors(() => shortcutController?.dispose(), errors);
      collectErrors(() => modalityTracker?.dispose(), errors);
      surfaceErrors(errors);
    },
  }) as InteractionHooksController<Options>;
}

export function useInteractionHooks<const Options extends InteractionHooksOptions = {}>(
  options: Options = {} as Options,
): InteractionHooksController<Options> {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createInteractionHooks(options);
  onScopeDispose(controller.dispose);
  return controller;
}

export type {
  InteractionFeatureOptions,
  InteractionHooksCancelResult,
  InteractionHooksController,
  InteractionHooksOptions,
  InteractionHooksProps,
  InteractionShortcutBinding,
  InteractionShortcutController,
  InteractionShortcutMatch,
  InteractionShortcutPlatform,
  InteractionShortcutProps,
} from "./interaction-hooks-types.ts";
export type { FocusRingOptions, FocusWithinOptions, HoverOptions, PressOptions };
export type { InteractionModalityOptions, InteractionShortcutOptions };
